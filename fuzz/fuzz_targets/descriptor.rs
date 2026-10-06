#![no_main]
use hidweave::{compare, decode, parse, parse_hex, MAX_FIELDS};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) { let _ = parse_hex(text); }
    if let Ok(layout) = parse(data) {
        assert!(compare(&layout, &layout).is_empty());
        for report in layout.reports().values() {
            let mut wire = vec![0; report.wire_bytes()];
            if report.key.id != 0 { wire[0] = report.key.id; }
            let values = decode(&layout, report.key.kind, &wire).unwrap();
            assert!(values.len() <= MAX_FIELDS);
            // Exercise arbitrary payload bits as well as all zeros.
            let skip = usize::from(report.key.id != 0);
            for (i, b) in wire.iter_mut().enumerate().skip(skip) { *b = data[i % data.len()]; }
            assert!(decode(&layout, report.key.kind, &wire).is_ok());
            for pair in report.fields.windows(2) {
                assert_eq!(pair[0].bit_offset + pair[0].bit_size * pair[0].count, pair[1].bit_offset);
            }
        }
    }
});
