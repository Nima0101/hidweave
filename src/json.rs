//! Dependency-free JSON renderers for contract version 1.
//! Numeric usages are fully qualified `(page << 16) | id` values.
use crate::*;
use std::fmt::Write;

/// Escape a string as a JSON string literal, including terminal control characters.
pub fn string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c <= '\u{1f}' => {
                write!(out, "\\u{:04x}", c as u32).expect("String write");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn key(key: ReportKey) -> String {
    format!(
        "\"kind\":{},\"id\":{}",
        string(&key.kind.to_string()),
        key.id
    )
}

/// Render a normalized field with source attribution.
pub fn field(field: &Field) -> String {
    let meaning = match &field.meaning {
        Meaning::Padding => "{\"type\":\"padding\"}".into(),
        Meaning::Variable(usage) => format!("{{\"type\":\"variable\",\"usage\":{usage}}}"),
        Meaning::Array(usages) => format!(
            "{{\"type\":\"array\",\"usages\":[{}]}}",
            usages
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
    };
    let semantics = if let Some(s) = &field.semantics {
        let route = s
            .collections
            .iter()
            .map(|c| {
                format!(
                    "{{\"kind\":{},\"usage\":{},\"sibling\":{}}}",
                    c.kind, c.usage, c.sibling
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"flags\":{},\"logical_min\":{},\"logical_max\":{},\"physical_min\":{},\"physical_max\":{},\"unit\":{},\"unit_exponent\":{},\"collections\":[{}]}}",
            s.flags,
            s.logical_min,
            s.logical_max,
            s.physical_min,
            s.physical_max,
            s.unit,
            s.unit_exponent,
            route
        )
    } else {
        "null".into()
    };
    format!(
        "{{\"bit_offset\":{},\"bit_size\":{},\"count\":{},\"source_offset\":{},\"meaning\":{},\"semantics\":{}}}",
        field.bit_offset, field.bit_size, field.count, field.source_offset, meaning, semantics
    )
}

/// Render an inspection report. Reports and fields have deterministic ordering.
pub fn layout(layout: &Layout) -> String {
    let reports = layout
        .reports()
        .values()
        .map(|r| {
            let fields = r.fields.iter().map(field).collect::<Vec<_>>().join(",");
            format!(
                "{{{},\"payload_bits\":{},\"wire_bytes\":{},\"fields\":[{}]}}",
                key(r.key),
                r.payload_bits,
                r.wire_bytes(),
                fields
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"contract_version\":{CONTRACT_VERSION},\"reports\":[{reports}]}}")
}

/// Render structured parser/report errors.
pub fn error(error: &Error) -> String {
    let kind = match error.kind {
        ErrorKind::Malformed => "malformed",
        ErrorKind::Unsupported => "unsupported",
        ErrorKind::Limit => "limit",
        ErrorKind::Report => "report",
    };
    format!(
        "{{\"kind\":{},\"code\":{},\"offset\":{},\"message\":{}}}",
        string(kind),
        string(error.code),
        error.offset,
        string(error.message)
    )
}

/// Render one report interpretation, including report-shape errors.
pub fn decoded(result: &Result<Vec<DecodedValue>, Error>) -> String {
    match result {
        Err(e) => format!("{{\"error\":{}}}", error(e)),
        Ok(values) => {
            let values = values.iter().map(|v| {
                let state = match v.state { ValueState::Value => "value", ValueState::Null => "null", ValueState::OutOfRange => "out_of_range", ValueState::Unmapped => "unmapped" };
                format!("{{{},\"field\":{},\"slot\":{},\"bit_offset\":{},\"value\":{},\"usage\":{},\"state\":{}}}",key(v.report),v.field,v.slot,v.bit_offset,v.value,v.usage.map_or("null".into(),|u| u.to_string()),string(state))
            }).collect::<Vec<_>>().join(",");
            format!("{{\"values\":[{values}]}}")
        }
    }
}

/// Render differences with old/new field snapshots. Optional evidence represents
/// the same raw report decoded under each layout; input bytes are not echoed.
pub fn comparison(old: &Layout, new: &Layout, evidence: Option<(ReportKind, &[u8])>) -> String {
    let changes = compare(old, new);
    let rendered = changes.iter().map(|c| {
        let kind = match c.kind { ChangeKind::ReportAdded => "report_added", ChangeKind::ReportRemoved => "report_removed", ChangeKind::ReportLength => "report_length", ChangeKind::FieldAdded => "field_added", ChangeKind::FieldRemoved => "field_removed", ChangeKind::FieldChanged => "field_changed" };
        let before = c.old_field.map_or("null".into(),|i| field(&old.reports()[&c.report].fields[i]));
        let after = c.new_field.map_or("null".into(),|i| field(&new.reports()[&c.report].fields[i]));
        let props = c.properties.iter().map(|p|string(p)).collect::<Vec<_>>().join(",");
        let old_bits = old.reports().get(&c.report).map_or("null".into(),|r|r.payload_bits.to_string());
        let new_bits = new.reports().get(&c.report).map_or("null".into(),|r|r.payload_bits.to_string());
        format!("{{{},\"change\":{},\"properties\":[{}],\"old_payload_bits\":{},\"new_payload_bits\":{},\"old\":{},\"new\":{}}}",key(c.report),string(kind),props,old_bits,new_bits,before,after)
    }).collect::<Vec<_>>().join(",");
    let evidence = evidence.map_or("null".into(), |(kind, bytes)| {
        format!(
            "{{\"old\":{},\"new\":{}}}",
            decoded(&decode(old, kind, bytes)),
            decoded(&decode(new, kind, bytes))
        )
    });
    format!(
        "{{\"contract_version\":{CONTRACT_VERSION},\"equal\":{},\"changes\":[{}],\"evidence\":{}}}",
        changes.is_empty(),
        rendered,
        evidence
    )
}
