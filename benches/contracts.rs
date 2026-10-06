//! Reproducible small-corpus microbenchmark.
use hidweave::{ReportKind, compare, decode, parse, parse_hex};
use std::{hint::black_box, time::Instant};
fn main() {
    let bytes = parse_hex(include_str!("../examples/axis-old.hex")).unwrap();
    let a = parse(&bytes).unwrap();
    let b = parse(&parse_hex(include_str!("../examples/axis-swapped.hex")).unwrap()).unwrap();
    let iterations = 100_000;
    for (name, op) in [("parse", 0), ("compare", 1), ("decode", 2)] {
        let start = Instant::now();
        for _ in 0..iterations {
            match op {
                0 => {
                    black_box(parse(black_box(&bytes)).unwrap());
                }
                1 => {
                    black_box(compare(black_box(&a), black_box(&b)));
                }
                _ => {
                    black_box(
                        decode(black_box(&a), ReportKind::Input, black_box(&[1, 2])).unwrap(),
                    );
                }
            }
        }
        println!(
            "{name}: {iterations} iterations, {} ns/op",
            start.elapsed().as_nanos() / iterations
        );
    }
}
