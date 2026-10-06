use crate::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Default)]
struct Number {
    raw: u32,
    bytes: usize,
}
impl Number {
    fn signed(self) -> i64 {
        let shift = 64 - self.bytes * 8;
        if self.bytes == 0 {
            0
        } else {
            ((self.raw as i64) << shift) >> shift
        }
    }
    fn maximum(self, minimum: i64) -> i64 {
        if minimum < 0 {
            self.signed()
        } else {
            self.raw as i64
        }
    }
}

#[derive(Clone, Default)]
struct Globals {
    page: u32,
    min: Option<i64>,
    max: Option<Number>,
    physical_min: Option<i64>,
    physical_max: Option<Number>,
    unit: u32,
    exponent: i8,
    size: u32,
    count: u32,
    id: u8,
}

#[derive(Default)]
struct Locals {
    usages: Vec<u32>,
    pending_min: Option<u32>,
}

fn fail(offset: usize, code: &'static str, message: &'static str) -> Error {
    Error::new(ErrorKind::Malformed, offset, code, message)
}
fn unsupported(offset: usize, code: &'static str, message: &'static str) -> Error {
    Error::new(ErrorKind::Unsupported, offset, code, message)
}
fn limit(offset: usize, code: &'static str, message: &'static str) -> Error {
    Error::new(ErrorKind::Limit, offset, code, message)
}

/// Decode whitespace-separated hexadecimal byte pairs, with optional `#` comments.
/// C arrays, `0x` prefixes and offset columns are deliberately not accepted.
/// Offsets in errors refer to input text bytes. Input must be at most 1 MiB.
pub fn parse_hex(text: &str) -> Result<Vec<u8>, Error> {
    if text.len() > 1_048_576 {
        return Err(limit(0, "hex_limit", "hex input exceeds 1 MiB"));
    }
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'#' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'#' {
            i += 1;
        }
        if i - start != 2 {
            return Err(fail(
                start,
                "hex_token",
                "expected a two-digit hexadecimal byte",
            ));
        }
        let digit = |b: u8| -> Option<u8> {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            }
        };
        let hi = digit(bytes[start])
            .ok_or_else(|| fail(start, "hex_digit", "invalid hexadecimal digit"))?;
        let lo = digit(bytes[start + 1])
            .ok_or_else(|| fail(start + 1, "hex_digit", "invalid hexadecimal digit"))?;
        if out.len() == MAX_DESCRIPTOR_BYTES {
            return Err(limit(
                start,
                "descriptor_limit",
                "decoded bytes exceed descriptor limit",
            ));
        }
        out.push(hi * 16 + lo);
    }
    Ok(out)
}

/// Parse a raw HID report descriptor into contract version 1.
/// See `docs/contract.md` for supported semantics and explicit resource limits.
pub fn parse(bytes: &[u8]) -> Result<Layout, Error> {
    if bytes.len() > MAX_DESCRIPTOR_BYTES {
        return Err(limit(
            0,
            "descriptor_limit",
            "descriptor exceeds 65535 bytes",
        ));
    }
    let mut globals = Globals::default();
    let mut stack = Vec::new();
    let mut locals = Locals::default();
    let mut collections = Vec::<Collection>::new();
    let mut siblings = vec![0_u32];
    let mut reports = BTreeMap::<ReportKey, Report>::new();
    let mut fields = 0_usize;
    let mut slots = 0_usize;
    let mut retained_usages = 0_usize;
    let mut seen_report_id = false;
    let mut i = 0;
    while i < bytes.len() {
        let source = i;
        let prefix = bytes[i];
        i += 1;
        if prefix == 0xfe {
            return Err(unsupported(
                source,
                "long_item",
                "long items are outside contract version 1",
            ));
        }
        let size = match prefix & 3 {
            3 => 4,
            n => n as usize,
        };
        let payload = bytes
            .get(i..i + size)
            .ok_or_else(|| fail(source, "truncated_item", "item payload is truncated"))?;
        i += size;
        let mut raw = 0;
        for (index, b) in payload.iter().enumerate() {
            raw |= (*b as u32) << (8 * index);
        }
        let number = Number { raw, bytes: size };
        let tag = prefix >> 4;
        let ty = (prefix >> 2) & 3;
        match ty {
            1 => {
                if matches!(tag, 10 | 11) {
                    if size != 0 {
                        return Err(fail(
                            source,
                            "stack_size",
                            "Push and Pop must have no payload",
                        ));
                    }
                } else if size == 0 {
                    return Err(fail(
                        source,
                        "global_size",
                        "global value requires a payload",
                    ));
                }
                match tag {
                    0 => {
                        if raw > 0xffff {
                            return Err(fail(source, "usage_page", "usage page exceeds 16 bits"));
                        }
                        globals.page = raw;
                    }
                    1 => globals.min = Some(number.signed()),
                    2 => globals.max = Some(number),
                    3 => globals.physical_min = Some(number.signed()),
                    4 => globals.physical_max = Some(number),
                    5 => {
                        // HID exponent is a signed nibble; accept signed integer encodings too.
                        let exponent = if raw <= 15 {
                            ((raw as i8) << 4) >> 4
                        } else {
                            let signed = number.signed();
                            if !(-8..=7).contains(&signed) {
                                return Err(unsupported(
                                    source,
                                    "unit_exponent",
                                    "unit exponent must be in -8..7",
                                ));
                            }
                            signed as i8
                        };
                        globals.exponent = exponent;
                    }
                    6 => {
                        if raw >> 28 != 0 || matches!(raw & 15, 5..=14) {
                            return Err(unsupported(
                                source,
                                "unit_reserved",
                                "reserved unit encoding",
                            ));
                        }
                        globals.unit = raw;
                    }
                    7 => globals.size = raw,
                    8 => {
                        if size != 1 || raw == 0 {
                            return Err(fail(
                                source,
                                "report_id",
                                "report ID must be one nonzero byte",
                            ));
                        }
                        globals.id = raw as u8;
                        seen_report_id = true;
                    }
                    9 => globals.count = raw,
                    10 => {
                        if stack.len() == MAX_DEPTH {
                            return Err(limit(
                                source,
                                "global_depth",
                                "global stack exceeds 32 entries",
                            ));
                        }
                        stack.push(globals.clone());
                    }
                    11 => {
                        globals = stack
                            .pop()
                            .ok_or_else(|| fail(source, "global_underflow", "Pop without Push"))?
                    }
                    _ => return Err(unsupported(source, "global_tag", "unknown global item")),
                }
            }
            2 => {
                if size == 0 {
                    return Err(fail(source, "local_size", "local value requires a payload"));
                }
                let usage = if size == 4 {
                    raw
                } else {
                    (globals.page << 16) | raw
                };
                match tag {
                    0 => {
                        if locals.pending_min.is_some() {
                            return Err(unsupported(
                                source,
                                "usage_order",
                                "Usage inside an unfinished range is not supported",
                            ));
                        }
                        if locals.usages.len() == MAX_USAGES {
                            return Err(limit(source, "usage_limit", "too many local usages"));
                        }
                        locals.usages.push(usage);
                    }
                    1 => {
                        if locals.pending_min.replace(usage).is_some() {
                            return Err(fail(source, "usage_range", "nested usage minima"));
                        }
                    }
                    2 => {
                        let min = locals.pending_min.take().ok_or_else(|| {
                            fail(source, "usage_range", "Usage Maximum without Minimum")
                        })?;
                        if min > usage || min >> 16 != usage >> 16 {
                            return Err(fail(
                                source,
                                "usage_range",
                                "usage range must be ordered on one page",
                            ));
                        }
                        let count = (usage - min) as usize + 1;
                        if count > MAX_USAGES - locals.usages.len() {
                            return Err(limit(
                                source,
                                "usage_limit",
                                "expanded usage list exceeds 4096",
                            ));
                        }
                        locals.usages.extend(min..=usage);
                    }
                    _ => {
                        return Err(unsupported(
                            source,
                            "local_tag",
                            "designators, strings, delimiters and unknown local items are not supported",
                        ));
                    }
                }
            }
            0 => {
                if locals.pending_min.is_some() {
                    return Err(fail(
                        source,
                        "usage_range",
                        "unfinished usage range before Main item",
                    ));
                }
                match tag {
                    10 => {
                        if size != 1 {
                            return Err(fail(
                                source,
                                "collection_size",
                                "Collection requires one byte",
                            ));
                        }
                        if (7..128).contains(&raw) {
                            return Err(unsupported(
                                source,
                                "collection_reserved",
                                "reserved collection type",
                            ));
                        }
                        if collections.len() == MAX_DEPTH {
                            return Err(limit(
                                source,
                                "collection_depth",
                                "collection nesting exceeds 32",
                            ));
                        }
                        if locals.usages.len() != 1 {
                            return Err(unsupported(
                                source,
                                "collection_usage",
                                "Collection requires exactly one usage in this profile",
                            ));
                        }
                        let sibling = siblings.last_mut().expect("root sibling counter");
                        collections.push(Collection {
                            kind: raw as u8,
                            usage: locals.usages[0],
                            sibling: *sibling,
                        });
                        *sibling += 1;
                        siblings.push(0);
                    }
                    12 => {
                        if size != 0 {
                            return Err(fail(
                                source,
                                "collection_size",
                                "End Collection must have no payload",
                            ));
                        }
                        if !locals.usages.is_empty() {
                            return Err(unsupported(
                                source,
                                "end_collection_usage",
                                "local usages on End Collection are not supported",
                            ));
                        }
                        collections.pop().ok_or_else(|| {
                            fail(
                                source,
                                "collection_underflow",
                                "End Collection without Collection",
                            )
                        })?;
                        siblings.pop();
                    }
                    8 | 9 | 11 => {
                        if size == 0 {
                            return Err(fail(
                                source,
                                "main_size",
                                "report Main item requires flags",
                            ));
                        }
                        if raw & !0x1ff != 0 || (tag == 8 && raw & 0x80 != 0) {
                            return Err(unsupported(
                                source,
                                "main_flags",
                                "reserved Main item flags",
                            ));
                        }
                        if raw & 0x100 != 0 {
                            return Err(unsupported(
                                source,
                                "buffered_bytes",
                                "buffered-byte reports are outside this profile",
                            ));
                        }
                        if collections.is_empty() {
                            return Err(fail(
                                source,
                                "missing_collection",
                                "report field must be inside a collection",
                            ));
                        }
                        if globals.size == 0 || globals.count == 0 {
                            return Err(fail(
                                source,
                                "report_dimensions",
                                "report size and count must be nonzero",
                            ));
                        }
                        if globals.count > MAX_FIELDS as u32 {
                            return Err(limit(source, "count_limit", "report count exceeds 4096"));
                        }
                        let bits = globals
                            .size
                            .checked_mul(globals.count)
                            .filter(|n| *n <= MAX_REPORT_BITS)
                            .ok_or_else(|| {
                                limit(source, "report_limit", "report item exceeds bit limit")
                            })?;
                        let constant = raw & 1 != 0;
                        if !constant && globals.size > 32 {
                            return Err(unsupported(
                                source,
                                "field_width",
                                "data fields wider than 32 bits are not supported",
                            ));
                        }
                        let kind = match tag {
                            8 => ReportKind::Input,
                            9 => ReportKind::Output,
                            _ => ReportKind::Feature,
                        };
                        let key = ReportKey {
                            kind,
                            id: globals.id,
                        };
                        let report = reports.entry(key).or_insert(Report {
                            key,
                            payload_bits: 0,
                            fields: Vec::new(),
                        });
                        let end = report
                            .payload_bits
                            .checked_add(bits)
                            .filter(|n| *n <= MAX_REPORT_BITS)
                            .ok_or_else(|| {
                                limit(source, "report_limit", "report payload exceeds 65536 bits")
                            })?;
                        let start = report.payload_bits;
                        report.payload_bits = end;
                        if constant {
                            if let Some(last) = report
                                .fields
                                .last_mut()
                                .filter(|f| f.meaning == Meaning::Padding)
                            {
                                last.bit_size += bits;
                            } else {
                                if fields == MAX_FIELDS {
                                    return Err(limit(
                                        source,
                                        "field_limit",
                                        "normalized fields exceed 4096",
                                    ));
                                }
                                report.fields.push(Field {
                                    bit_offset: start,
                                    bit_size: bits,
                                    count: 1,
                                    meaning: Meaning::Padding,
                                    semantics: None,
                                    source_offset: source,
                                });
                                fields += 1;
                            }
                        } else {
                            let min = globals.min.ok_or_else(|| {
                                fail(source, "logical_bounds", "data requires Logical Minimum")
                            })?;
                            let max = globals
                                .max
                                .ok_or_else(|| {
                                    fail(source, "logical_bounds", "data requires Logical Maximum")
                                })?
                                .maximum(min);
                            if min > max {
                                return Err(fail(
                                    source,
                                    "logical_bounds",
                                    "logical range is reversed",
                                ));
                            }
                            let (low, high) = if min < 0 {
                                (
                                    -(1_i64 << (globals.size - 1)),
                                    (1_i64 << (globals.size - 1)) - 1,
                                )
                            } else {
                                (0, (1_i64 << globals.size) - 1)
                            };
                            if min < low || max > high {
                                return Err(fail(
                                    source,
                                    "logical_width",
                                    "logical range does not fit field width",
                                ));
                            }
                            if locals.usages.is_empty() {
                                return Err(unsupported(
                                    source,
                                    "missing_usage",
                                    "data field requires a usage",
                                ));
                            }
                            let (pmin, pmax) = match (globals.physical_min, globals.physical_max) {
                                (Some(pmin), Some(pmax)) => (pmin, pmax.maximum(pmin)),
                                _ => (min, max),
                            };
                            let (pmin, pmax) = if pmin == 0 && pmax == 0 {
                                (min, max)
                            } else {
                                (pmin, pmax)
                            };
                            if pmin > pmax {
                                return Err(fail(
                                    source,
                                    "physical_bounds",
                                    "physical range is reversed",
                                ));
                            }
                            let semantics = Semantics {
                                flags: raw,
                                logical_min: min,
                                logical_max: max,
                                physical_min: pmin,
                                physical_max: pmax,
                                unit: globals.unit,
                                unit_exponent: globals.exponent,
                                collections: collections.clone(),
                            };
                            let variable = raw & 2 != 0;
                            if globals.count as usize > MAX_FIELDS - slots {
                                return Err(limit(
                                    source,
                                    "slot_limit",
                                    "total data slots exceed 4096",
                                ));
                            }
                            slots += globals.count as usize;
                            if !variable {
                                if locals.usages.len() > MAX_TOTAL_USAGES - retained_usages {
                                    return Err(limit(
                                        source,
                                        "total_usage_limit",
                                        "retained array usages exceed 65536",
                                    ));
                                }
                                retained_usages += locals.usages.len();
                            }
                            let added = if variable { globals.count as usize } else { 1 };
                            if added > MAX_FIELDS - fields {
                                return Err(limit(
                                    source,
                                    "field_limit",
                                    "normalized fields exceed 4096",
                                ));
                            }
                            if variable && locals.usages.len() > globals.count as usize {
                                return Err(unsupported(
                                    source,
                                    "extra_usages",
                                    "excess variable usages are outside this profile",
                                ));
                            }
                            for n in 0..added {
                                let meaning = if variable {
                                    Meaning::Variable(locals.usages[n.min(locals.usages.len() - 1)])
                                } else {
                                    Meaning::Array(locals.usages.clone())
                                };
                                report.fields.push(Field {
                                    bit_offset: start + n as u32 * globals.size,
                                    bit_size: globals.size,
                                    count: if variable { 1 } else { globals.count },
                                    meaning,
                                    semantics: Some(semantics.clone()),
                                    source_offset: source,
                                });
                            }
                            fields += added;
                        }
                    }
                    _ => return Err(unsupported(source, "main_tag", "unknown Main item")),
                }
                locals = Locals::default();
            }
            _ => return Err(unsupported(source, "reserved_item", "reserved item type")),
        }
    }
    if !collections.is_empty() {
        return Err(fail(i, "unclosed_collection", "collection is not closed"));
    }
    if !stack.is_empty() {
        return Err(fail(i, "unclosed_push", "Push is not balanced by Pop"));
    }
    if !locals.usages.is_empty() || locals.pending_min.is_some() {
        return Err(fail(i, "dangling_local", "local items have no Main item"));
    }
    if reports.is_empty() {
        return Err(fail(i, "empty_layout", "descriptor defines no reports"));
    }
    if seen_report_id && reports.keys().any(|k| k.id == 0) {
        return Err(fail(
            i,
            "mixed_ids",
            "every report must be numbered if any Report ID item occurs",
        ));
    }
    Ok(Layout { reports })
}
