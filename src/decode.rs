use crate::*;

/// Meaning of a decoded raw integer within its declared bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueState {
    /// In logical range; an array selector also has a mapped usage.
    Value,
    /// Outside logical range with the Null State flag set.
    Null,
    /// Outside logical range without Null State.
    OutOfRange,
    /// In range, but the array has no usage for this selector.
    Unmapped,
}

/// One decoded variable or array slot; constant padding is omitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedValue {
    /// Report selected from the wire prefix and caller-supplied kind.
    pub report: ReportKey,
    /// Field index in the report.
    pub field: usize,
    /// Slot index (zero for a variable).
    pub slot: u32,
    /// Bit offset in payload, excluding report ID.
    pub bit_offset: u32,
    /// Logical signed/unsigned integer, before physical unit scaling.
    pub value: i64,
    /// Variable usage or selected array usage, if mapped.
    pub usage: Option<u32>,
    /// Whether the value is valid, null, invalid, or unmapped.
    pub state: ValueState,
}

/// Decode exactly one wire report. A numbered report includes its ID byte;
/// an unnumbered report does not include an API-specific leading zero.
/// Length must match exactly. Invalid logical values are reported, not coerced.
pub fn decode(layout: &Layout, kind: ReportKind, bytes: &[u8]) -> Result<Vec<DecodedValue>, Error> {
    let err = |code, message| Error::new(ErrorKind::Report, 0, code, message);
    let numbered = layout.numbered();
    let id = if numbered {
        *bytes
            .first()
            .ok_or_else(|| err("report_id_missing", "numbered report has no ID byte"))?
    } else {
        0
    };
    let key = ReportKey { kind, id };
    let report = layout
        .reports
        .get(&key)
        .ok_or_else(|| err("report_unknown", "report kind/ID is not defined"))?;
    if bytes.len() != report.wire_bytes() {
        return Err(err(
            "report_length",
            "wire report length does not match descriptor",
        ));
    }
    let payload = &bytes[usize::from(numbered)..];
    let mut values = Vec::new();
    for (index, field) in report.fields.iter().enumerate() {
        let Some(semantics) = &field.semantics else {
            continue;
        };
        for slot in 0..field.count {
            let bit_offset = field.bit_offset + slot * field.bit_size;
            let mut raw = 0_u32;
            for bit in 0..field.bit_size {
                let source = bit_offset + bit;
                raw |= (((payload[(source / 8) as usize] >> (source % 8)) & 1) as u32) << bit;
            }
            let value = if semantics.logical_min < 0 {
                let shift = 64 - field.bit_size;
                ((raw as i64) << shift) >> shift
            } else {
                raw as i64
            };
            let in_range = (semantics.logical_min..=semantics.logical_max).contains(&value);
            let usage = match &field.meaning {
                Meaning::Variable(usage) => Some(*usage),
                Meaning::Array(usages) if in_range => usages
                    .get((value - semantics.logical_min) as usize)
                    .copied(),
                _ => None,
            };
            let state = if !in_range {
                if semantics.flags & 0x40 != 0 {
                    ValueState::Null
                } else {
                    ValueState::OutOfRange
                }
            } else if usage.is_none() {
                ValueState::Unmapped
            } else {
                ValueState::Value
            };
            values.push(DecodedValue {
                report: key,
                field: index,
                slot,
                bit_offset,
                value,
                usage,
                state,
            });
        }
    }
    Ok(values)
}
