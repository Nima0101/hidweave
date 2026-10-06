//! Minimal downstream library usage; all inputs are synthetic.
use hidweave::{ReportKind, compare, decode, parse, parse_hex};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let old = parse(&parse_hex(include_str!("axis-old.hex"))?)?;
    let new = parse(&parse_hex(include_str!("axis-swapped.hex"))?)?;
    assert_eq!(compare(&old, &new).len(), 2);
    let values = decode(&new, ReportKind::Input, &[1, 2])?;
    assert_eq!(values[1].usage, Some(0x0001_0030));
    assert_eq!(values[1].value, 2);
    println!("X now reads the second byte: {}", values[1].value);
    Ok(())
}
