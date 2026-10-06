//! Contract regression and property tests.
use hidweave::*;

fn layout(text: &str) -> Layout {
    parse(&parse_hex(text).unwrap()).unwrap()
}
fn base() -> Vec<u8> {
    parse_hex(include_str!("../examples/axis-old.hex")).unwrap()
}
fn error(text: &str) -> Error {
    parse(&parse_hex(text).unwrap()).unwrap_err()
}
fn data(body: &str) -> Layout {
    layout(&format!("05 01 09 04 a1 01 {body} c0"))
}
fn first(l: &Layout) -> &Field {
    &l.reports().values().next().unwrap().fields[0]
}

#[test]
fn equivalent_factoring() {
    let old = parse(&base()).unwrap();
    let new = layout(include_str!("../examples/axis-equivalent.hex"));
    assert!(compare(&old, &new).is_empty());
    assert_ne!(first(&old).source_offset, first(&new).source_offset);
}
#[test]
fn axis_swap_matches_by_usage_and_exposes_meaning() {
    let a = parse(&base()).unwrap();
    let b = layout(include_str!("../examples/axis-swapped.hex"));
    let changes = compare(&a, &b);
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].old_field, Some(0));
    assert_eq!(changes[0].new_field, Some(1));
    assert_eq!(changes[0].properties, vec!["bit_offset"]);
    let a = decode(&a, ReportKind::Input, &[1, 2]).unwrap();
    let b = decode(&b, ReportKind::Input, &[1, 2]).unwrap();
    assert_eq!((a[0].usage, a[0].value), (Some(0x10030), 1));
    assert_eq!((b[1].usage, b[1].value), (Some(0x10030), 2));
}
#[test]
fn unsigned_maximum_and_signed_negative() {
    let unsigned = data("15 00 25 ff 75 08 95 01 09 30 81 02");
    assert_eq!(
        first(&unsigned).semantics.as_ref().unwrap().logical_max,
        255
    );
    assert_eq!(
        decode(&unsigned, ReportKind::Input, &[255]).unwrap()[0].value,
        255
    );
    let signed = parse(&base()).unwrap();
    assert_eq!(
        decode(&signed, ReportKind::Input, &[255, 128]).unwrap()[0].value,
        -1
    );
    assert_eq!(
        decode(&signed, ReportKind::Input, &[255, 128]).unwrap()[1].state,
        ValueState::OutOfRange
    );
}
#[test]
fn maximum_interpreted_using_active_minimum_not_item_order() {
    let a = data("25 ff 15 00 75 08 95 01 09 30 81 02");
    let b = data("15 00 25 ff 75 08 95 01 09 30 81 02");
    assert!(compare(&a, &b).is_empty());
}
#[test]
fn independent_kinds_and_ids() {
    let l = data("85 01 15 00 25 ff 75 08 95 01 09 30 81 02 09 31 91 02 85 02 09 32 b1 02");
    assert_eq!(l.reports().len(), 3);
    for r in l.reports().values() {
        assert_eq!(r.fields[0].bit_offset, 0);
        assert_eq!(r.wire_bytes(), 2);
    }
    assert_eq!(
        decode(&l, ReportKind::Output, &[1, 42]).unwrap()[0].usage,
        Some(0x10031)
    );
    assert_eq!(
        decode(&l, ReportKind::Feature, &[2, 42]).unwrap()[0].usage,
        Some(0x10032)
    );
    assert!(decode(&l, ReportKind::Input, &[2, 42]).is_err());
}
#[test]
fn repeated_id_appends_same_report() {
    let l = data("85 01 15 00 25 ff 75 08 95 01 09 30 81 02 85 02 09 31 81 02 85 01 09 32 81 02");
    let values = decode(&l, ReportKind::Input, &[1, 4, 5]).unwrap();
    assert_eq!(values[1].bit_offset, 8);
    assert_eq!(values[1].usage, Some(0x10032));
}
#[test]
fn push_pop_restores_every_global_but_not_locals() {
    let a = data(
        "15 00 25 0f 75 04 95 01 09 30 81 02 a4 05 09 15 ff 25 00 75 01 09 01 81 02 b4 09 31 81 02",
    );
    let r = a.reports().values().next().unwrap();
    assert_eq!(r.payload_bits, 9);
    assert_eq!(r.fields[2].meaning, Meaning::Variable(0x10031));
    assert_eq!(r.fields[2].semantics.as_ref().unwrap().logical_max, 15);
}
#[test]
fn local_state_resets_after_every_main_item() {
    assert_eq!(
        error("05 01 09 04 a1 01 15 00 25 01 75 01 95 01 09 30 81 02 81 02 c0").code,
        "missing_usage"
    );
    assert_eq!(
        error("05 01 09 04 a1 01 15 00 25 01 75 01 95 01 81 02 c0").code,
        "missing_usage"
    );
}
#[test]
fn usage_range_and_repeat_last_normalize() {
    let a = data("15 00 25 01 75 01 95 03 19 30 29 31 81 02");
    let b = data("15 00 25 01 75 01 95 03 09 30 09 31 09 31 81 02");
    assert!(compare(&a, &b).is_empty());
}
#[test]
fn extended_usage_uses_its_own_page() {
    let a = data("15 00 25 ff 75 08 95 01 0b 30 00 09 00 81 02");
    assert_eq!(first(&a).meaning, Meaning::Variable(0x90030));
}
#[test]
fn padding_factoring_and_irrelevant_state_normalize() {
    let a = data("75 08 95 01 81 01");
    let b = data("15 81 25 7f 75 04 95 01 81 03 81 01");
    assert!(compare(&a, &b).is_empty());
}
#[test]
fn partial_physical_bounds_both_default_to_logical() {
    let a = data("15 00 25 64 75 08 95 01 09 30 81 02");
    for physical in ["45 c8", "35 20", "35 00 45 00"] {
        let b = data(&format!("15 00 25 64 {physical} 75 08 95 01 09 30 81 02"));
        assert!(compare(&a, &b).is_empty(), "{physical}");
    }
}
#[test]
fn physical_units_are_part_of_contract() {
    let a = data("15 00 25 64 35 00 45 c8 65 11 55 0e 75 08 95 01 09 30 81 02");
    let s = first(&a).semantics.as_ref().unwrap();
    assert_eq!(s.physical_max, 200);
    assert_eq!(s.unit_exponent, -2);
    let b = data("15 00 25 64 35 00 45 c8 65 11 55 fe 75 08 95 01 09 30 81 02");
    assert!(compare(&a, &b).is_empty());
    let c = data("15 00 25 64 35 00 45 c8 65 11 55 0f 75 08 95 01 09 30 81 02");
    assert!(compare(&a, &c)[0].properties.contains(&"unit"));
}
#[test]
fn arrays_map_relative_to_logical_minimum_and_preserve_nulls() {
    let l = layout(include_str!("../examples/keyboard.hex"));
    let values = decode(&l, ReportKind::Input, &[3, 1, 6]).unwrap();
    assert_eq!(values[0].usage, Some(0x70004));
    assert_eq!(values[1].usage, Some(0x70009));
    let values = decode(&l, ReportKind::Input, &[3, 0, 7]).unwrap();
    assert!(
        values
            .iter()
            .all(|v| v.state == ValueState::Null && v.usage.is_none())
    );
}
#[test]
fn unmapped_array_selector_is_not_guessed() {
    let l = data("15 00 25 03 75 02 95 01 09 30 81 00");
    assert_eq!(
        decode(&l, ReportKind::Input, &[2]).unwrap()[0].state,
        ValueState::Unmapped
    );
}
#[test]
fn array_map_changes_are_visible() {
    let a = data("15 00 25 01 75 01 95 02 09 30 09 31 81 00");
    let b = data("15 00 25 01 75 01 95 02 09 31 09 30 81 00");
    assert_eq!(compare(&a, &b)[0].properties, vec!["meaning"]);
}
#[test]
fn collection_routes_and_repeated_usages_are_not_conflated() {
    let a = data("15 00 25 ff 75 08 95 01 09 01 a1 00 09 30 81 02 c0 09 02 a1 00 09 30 81 02 c0");
    let b = data("15 00 25 ff 75 08 95 01 09 02 a1 00 09 30 81 02 c0 09 01 a1 00 09 30 81 02 c0");
    let c = compare(&a, &b);
    assert_eq!(c.len(), 2);
    assert!(c[0].properties.contains(&"collections"));
}
#[test]
fn added_report_and_changed_length_are_separate_diagnostics() {
    let a = data("85 01 15 00 25 ff 75 08 95 01 09 30 81 02");
    let b = data("85 01 15 00 25 ff 75 08 95 01 09 30 81 02 09 31 81 02 85 02 09 32 81 02");
    let c = compare(&a, &b);
    assert!(c.iter().any(|c| c.kind == ChangeKind::ReportLength));
    assert!(c.iter().any(|c| c.kind == ChangeKind::ReportAdded));
}
#[test]
fn insertion_matches_existing_fields_by_identity() {
    let a = parse(&base()).unwrap();
    let b = data("15 81 25 7f 75 08 95 03 09 32 09 30 09 31 81 02");
    let c = compare(&a, &b);
    let moved: Vec<_> = c
        .iter()
        .filter(|c| c.kind == ChangeKind::FieldChanged)
        .collect();
    assert_eq!(moved.len(), 2);
    assert!(moved.iter().all(|c| c.properties == ["bit_offset"]));
}
#[test]
fn rejects_mixed_ids_even_unused_or_popped() {
    for suffix in ["85 01", "a4 85 01 b4"] {
        assert_eq!(
            error(&format!(
                "05 01 09 04 a1 01 15 00 25 01 75 01 95 01 09 30 81 02 {suffix} c0"
            ))
            .code,
            "mixed_ids"
        );
    }
}
#[test]
fn malformed_and_unsupported_paths() {
    for (s, code) in [
        ("", "empty_layout"),
        ("05", "truncated_item"),
        ("b4", "global_underflow"),
        ("c0", "collection_underflow"),
        ("fe", "long_item"),
        ("0c", "reserved_item"),
        ("a9 01", "local_tag"),
        ("85 00", "report_id"),
        ("86 01 00", "report_id"),
        ("19 01 29 00", "usage_range"),
        ("29 01", "usage_range"),
        ("09 01", "dangling_local"),
        ("a4", "unclosed_push"),
        ("05 01 09 01 a1 01", "unclosed_collection"),
        (
            "05 01 09 01 a1 01 75 08 95 01 82 00 01 c0",
            "buffered_bytes",
        ),
        (
            "05 01 09 01 a1 01 15 00 25 ff 75 04 95 01 09 30 81 02 c0",
            "logical_width",
        ),
        (
            "05 01 09 01 a1 01 15 01 25 00 75 08 95 01 09 30 81 02 c0",
            "logical_bounds",
        ),
    ] {
        assert_eq!(error(s).code, code, "{s}");
    }
}
#[test]
fn exact_wire_framing_required() {
    let l = parse(&base()).unwrap();
    for b in [&[][..], &[1][..], &[0, 1, 2][..]] {
        assert_eq!(
            decode(&l, ReportKind::Input, b).unwrap_err().code,
            "report_length"
        );
    }
    let l = layout(include_str!("../examples/keyboard.hex"));
    assert_eq!(
        decode(&l, ReportKind::Input, &[]).unwrap_err().code,
        "report_id_missing"
    );
}
#[test]
fn strict_hex_grammar_and_limits() {
    assert_eq!(parse_hex("01 02# note\n ff\t00").unwrap(), [1, 2, 255, 0]);
    for text in ["0x01", "0102", "0", "gg", "é", "01, 02"] {
        assert!(parse_hex(text).is_err(), "{text}");
    }
    assert_eq!(
        parse(&vec![0; MAX_DESCRIPTOR_BYTES + 1]).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        parse_hex(&" ".repeat(1_048_577)).unwrap_err().kind,
        ErrorKind::Limit
    );
}
#[test]
fn resource_limits_precede_expansion() {
    assert_eq!(error("05 01 19 00 2a ff ff").code, "usage_limit");
    assert_eq!(
        error("05 01 09 04 a1 01 77 ff ff ff ff 95 02 81 01 c0").code,
        "report_limit"
    );
    assert_eq!(parse(&[0xa4; 33]).unwrap_err().code, "global_depth");
    let mut bytes = Vec::new();
    for _ in 0..33 {
        bytes.extend([0x09, 1, 0xa1, 1]);
    }
    assert_eq!(parse(&bytes).unwrap_err().code, "collection_depth");
    let mut bytes =
        parse_hex("05 01 09 04 a1 01 15 00 25 01 75 01 96 00 10 09 30 81 00 09 30 81 00 c0")
            .unwrap();
    assert_eq!(parse(&bytes).unwrap_err().code, "slot_limit");
    bytes.clear();
}

// Independent extraction oracle: assemble an entire payload as u64 then mask/shift,
// rather than the production decoder's individual-bit reads.
#[test]
fn exhaustive_small_bitfield_oracle() {
    for padding in 0..8_u32 {
        for width in 1..=8_u32 {
            for signed in [false, true] {
                let mut bytes = parse_hex("05 01 09 04 a1 01").unwrap();
                if padding > 0 {
                    bytes.extend([0x75, padding as u8, 0x95, 1, 0x81, 1]);
                }
                if signed {
                    bytes.extend([
                        0x15,
                        (-(1_i16 << (width - 1))) as u8,
                        0x25,
                        ((1_u16 << (width - 1)) - 1) as u8,
                    ]);
                } else {
                    bytes.extend([0x15, 0, 0x25, ((1_u16 << width) - 1) as u8]);
                }
                bytes.extend([0x75, width as u8, 0x95, 1, 0x09, 0x30, 0x81, 2, 0xc0]);
                let l = parse(&bytes).unwrap();
                for raw in 0..(1_u64 << width) {
                    let packed = (raw << padding) | ((1_u64 << padding) - 1);
                    let wire = packed.to_le_bytes();
                    let count = (padding + width).div_ceil(8) as usize;
                    let value = decode(&l, ReportKind::Input, &wire[..count]).unwrap()[0].value;
                    let expected = if signed && raw & (1 << (width - 1)) != 0 {
                        raw as i64 - (1_i64 << width)
                    } else {
                        raw as i64
                    };
                    assert_eq!(
                        value, expected,
                        "padding={padding}, width={width}, raw={raw}"
                    );
                }
            }
        }
    }
}
#[test]
fn full_32_bit_extremes() {
    let a = data("15 00 27 ff ff ff ff 75 20 95 01 09 30 81 02");
    assert_eq!(
        decode(&a, ReportKind::Input, &[255; 4]).unwrap()[0].value,
        u32::MAX as i64
    );
    let b = data("17 00 00 00 80 27 ff ff ff 7f 75 20 95 01 09 30 81 02");
    assert_eq!(
        decode(&b, ReportKind::Input, &[0, 0, 0, 128]).unwrap()[0].value,
        i32::MIN as i64
    );
}
#[test]
fn all_prefix_truncations_rejected() {
    let b = base();
    for n in 0..b.len() {
        assert!(parse(&b[..n]).is_err(), "prefix {n}");
    }
}
#[test]
fn deterministic_mutation_smoke() {
    let seeds = [
        base(),
        parse_hex(include_str!("../examples/keyboard.hex")).unwrap(),
    ];
    let mut state = 0x123456789abcdef_u64;
    for n in 0..30_000 {
        let mut bytes = seeds[n % seeds.len()].clone();
        for _ in 0..1 + n % 5 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let i = state as usize % bytes.len();
            bytes[i] = (state >> 32) as u8;
        }
        if let Ok(l) = parse(&bytes) {
            assert!(compare(&l, &l).is_empty());
            for r in l.reports().values() {
                let mut wire = vec![0; r.wire_bytes()];
                if r.key.id != 0 {
                    wire[0] = r.key.id;
                }
                let decoded = decode(&l, r.key.kind, &wire).unwrap();
                assert!(decoded.len() <= MAX_FIELDS);
                assert!(
                    r.fields
                        .windows(2)
                        .all(|pair| pair[0].bit_offset + pair[0].bit_size * pair[0].count
                            == pair[1].bit_offset)
                );
            }
        }
    }
}

#[test]
fn rounded_wire_length_only_reported_when_changed() {
    let a = data("15 00 25 01 75 01 95 01 09 30 81 02");
    let b = data("15 00 25 01 75 02 95 01 09 30 81 02");
    let changes = compare(&a, &b);
    let length = changes
        .iter()
        .find(|c| c.kind == ChangeKind::ReportLength)
        .unwrap();
    assert_eq!(length.properties, ["payload_bits"]);
}

#[test]
fn negative_minimum_array_mapping() {
    let l = data("15 fe 25 01 75 03 95 01 19 30 29 33 81 40");
    for (wire, usage, state) in [
        (6, Some(0x10030), ValueState::Value),
        (7, Some(0x10031), ValueState::Value),
        (0, Some(0x10032), ValueState::Value),
        (1, Some(0x10033), ValueState::Value),
        (2, None, ValueState::Null),
    ] {
        let v = decode(&l, ReportKind::Input, &[wire]).unwrap();
        assert_eq!((v[0].usage, v[0].state), (usage, state));
    }
}

#[test]
fn aggregate_usage_budget_exact_boundary() {
    let mut bytes = parse_hex("05 01 09 04 a1 01 15 00 26 ff 0f 75 10 95 01").unwrap();
    let item = [0x19, 0, 0x2a, 0xff, 0x0f, 0x81, 0];
    for _ in 0..16 {
        bytes.extend(item);
    }
    let mut good = bytes.clone();
    good.push(0xc0);
    assert!(parse(&good).is_ok());
    bytes.extend(item);
    bytes.push(0xc0);
    assert_eq!(parse(&bytes).unwrap_err().code, "total_usage_limit");
}

#[test]
fn report_bit_boundary_and_total_slots() {
    let exact = data("76 00 01 96 00 01 81 01");
    assert_eq!(
        exact.reports().values().next().unwrap().payload_bits,
        MAX_REPORT_BITS
    );
    assert_eq!(
        error("05 01 09 04 a1 01 76 00 01 96 00 01 81 01 75 01 95 01 81 01 c0").code,
        "report_limit"
    );
    let exact = data("15 00 25 01 75 01 96 00 10 09 30 81 02");
    assert_eq!(
        exact.reports().values().next().unwrap().fields.len(),
        MAX_FIELDS
    );
    assert_eq!(
        decode(&exact, ReportKind::Input, &vec![0; 512])
            .unwrap()
            .len(),
        MAX_FIELDS
    );
}

#[test]
fn matching_equality_iff_normalized_ordered_contract_equal() {
    fn oracle(a: &Layout, b: &Layout) -> bool {
        if a.reports().len() != b.reports().len() {
            return false;
        }
        for (key, a) in a.reports() {
            let Some(b) = b.reports().get(key) else {
                return false;
            };
            if a.payload_bits != b.payload_bits || a.fields.len() != b.fields.len() {
                return false;
            }
            for (a, b) in a.fields.iter().zip(&b.fields) {
                let mut a = a.clone();
                let mut b = b.clone();
                a.source_offset = 0;
                b.source_offset = 0;
                if a != b {
                    return false;
                }
            }
        }
        true
    }
    let sequences: &[&[u8]] = &[
        &[0x30],
        &[0x31],
        &[0x30, 0x31],
        &[0x31, 0x30],
        &[0x30, 0x30],
        &[0x30, 0x31, 0x30],
        &[0x32, 0x30, 0x31],
        &[0x30, 0x30, 0x31],
    ];
    let mut layouts = Vec::new();
    for sequence in sequences {
        for width in [1, 2, 8] {
            for array in [false, true] {
                let mut bytes = parse_hex("05 01 09 04 a1 01 15 00 25 01").unwrap();
                bytes.extend([0x75, width, 0x95, sequence.len() as u8]);
                for u in *sequence {
                    bytes.extend([0x09, *u]);
                }
                bytes.extend([0x81, if array { 0 } else { 2 }, 0xc0]);
                layouts.push(parse(&bytes).unwrap());
            }
        }
    }
    for a in &layouts {
        for b in &layouts {
            assert_eq!(compare(a, b).is_empty(), oracle(a, b));
        }
    }
}

#[test]
fn evidence_error_does_not_hide_descriptor_difference() {
    let a = parse(&base()).unwrap();
    let b = layout(include_str!("../examples/keyboard.hex"));
    assert!(!compare(&a, &b).is_empty());
    let report = json::comparison(&a, &b, Some((ReportKind::Input, &[1, 2])));
    assert!(report.contains("\"equal\":false"));
    assert!(report.contains("report_unknown"));
}
