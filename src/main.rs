//! Command-line interface for hidweave.
use hidweave::*;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::ExitCode,
};

const HELP: &str = "hidweave 0.1.0 — compare HID report contracts\n\nUsage:\n  hidweave demo\n  hidweave inspect DESCRIPTOR [--hex] [--json]\n  hidweave compare OLD NEW [--hex] [--json] [--report REPORT --kind input|output|feature]\n  hidweave gate OLD NEW [--hex] [--policy strict|add-reports]\n  hidweave decode DESCRIPTOR REPORT --kind input|output|feature [--hex] [--json]\n\nInputs are raw binary unless --hex is set (then all inputs are whitespace-separated\nhex byte pairs with optional # comments). Report bytes include ID only if numbered.\nOffsets exclude the report-ID byte. No device access.\n\nExit: 0 equal/success; 1 contract changed; 2 input/unsupported/resource/report error.\nCompare evidence errors do not replace the descriptor comparison exit status.\n";

fn read(path: &str, hex: bool) -> Result<Vec<u8>, String> {
    let path = Path::new(path);
    // Bound reads even if the file grows after metadata. Refuse non-regular files.
    let meta = path
        .metadata()
        .map_err(|_| "cannot inspect input file".to_string())?;
    if !meta.is_file() {
        return Err("input must be a regular file".into());
    }
    let max = if hex { 1_048_576 } else { MAX_DESCRIPTOR_BYTES };
    if meta.len() > max as u64 {
        return Err("input exceeds file-size limit".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "cannot open input file")?
        .take((max + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read input file")?;
    if bytes.len() > max {
        return Err("input exceeds file-size limit".into());
    }
    if hex {
        parse_hex(std::str::from_utf8(&bytes).map_err(|_| "hex input is not UTF-8")?)
            .map_err(|e| e.to_string())
    } else {
        Ok(bytes)
    }
}
fn usage_name(usage: u32) -> String {
    let name = match usage {
        0x10030 => "X",
        0x10031 => "Y",
        0x10032 => "Z",
        _ => "",
    };
    if name.is_empty() {
        format!("{:04x}:{:04x}", usage >> 16, usage & 0xffff)
    } else {
        format!("{name} ({:04x}:{:04x})", usage >> 16, usage & 0xffff)
    }
}
fn field_name(field: &Field) -> String {
    match &field.meaning {
        Meaning::Padding => "padding".into(),
        Meaning::Variable(u) => usage_name(*u),
        Meaning::Array(_) => "array selectors".into(),
    }
}
fn show_values(result: Result<Vec<DecodedValue>, Error>) -> String {
    match result {
        Err(e) => format!("  {e}\n"),
        Ok(values) => values
            .iter()
            .map(|v| {
                format!(
                    "  {} id={} bit={} field={} slot={} {} = {} [{:?}]\n",
                    v.report.kind,
                    v.report.id,
                    v.bit_offset,
                    v.field,
                    v.slot,
                    v.usage.map_or("unmapped".into(), usage_name),
                    v.value,
                    v.state
                )
            })
            .collect(),
    }
}
fn show_compare(old: &Layout, new: &Layout, evidence: Option<(ReportKind, &[u8])>) -> String {
    let changes = compare(old, new);
    let mut out = format!(
        "contract v{CONTRACT_VERSION}: {}\n",
        if changes.is_empty() {
            "equal"
        } else {
            "changed"
        }
    );
    for c in changes {
        out.push_str(&format!(
            "{} id={} {:?}",
            c.report.kind, c.report.id, c.kind
        ));
        match (c.old_field, c.new_field) {
            (Some(a), Some(b)) => {
                let a = &old.reports()[&c.report].fields[a];
                let b = &new.reports()[&c.report].fields[b];
                out.push_str(&format!(
                    ": {} bit {} -> {} bit {}; {} (descriptor bytes {} -> {})",
                    field_name(a),
                    a.bit_offset,
                    field_name(b),
                    b.bit_offset,
                    c.properties.join(", "),
                    a.source_offset,
                    b.source_offset
                ));
            }
            (Some(a), None) => out.push_str(&format!(
                ": {}",
                field_name(&old.reports()[&c.report].fields[a])
            )),
            (None, Some(b)) => out.push_str(&format!(
                ": {}",
                field_name(&new.reports()[&c.report].fields[b])
            )),
            _ => {
                let length = |l: &Layout| {
                    l.reports().get(&c.report).map_or("absent".into(), |r| {
                        format!("{} bits / {} wire bytes", r.payload_bits, r.wire_bytes())
                    })
                };
                out.push_str(&format!(": {} -> {}", length(old), length(new)));
            }
        }
        out.push('\n');
    }
    if let Some((kind, bytes)) = evidence {
        out.push_str("same report, old interpretation:\n");
        out.push_str(&show_values(decode(old, kind, bytes)));
        out.push_str("same report, new interpretation:\n");
        out.push_str(&show_values(decode(new, kind, bytes)));
    }
    out
}
fn demo() -> Result<(u8, String), String> {
    let load = |s| {
        parse_hex(s)
            .and_then(|b| parse(&b))
            .map_err(|e| e.to_string())
    };
    let old = load(include_str!("../examples/axis-old.hex"))?;
    let equivalent = load(include_str!("../examples/axis-equivalent.hex"))?;
    let swapped = load(include_str!("../examples/axis-swapped.hex"))?;
    let mut out = String::from(
        "Synthetic two-axis firmware regression\n\n1. Different encoding, same contract:\n",
    );
    out.push_str(&show_compare(&old, &equivalent, None));
    out.push_str("\n2. Same two-byte length, swapped axis meanings:\n");
    out.push_str(&show_compare(
        &old,
        &swapped,
        Some((ReportKind::Input, &[1, 2])),
    ));
    out.push_str("\nA fixed-layout consumer now attributes each value to the wrong axis.\nThis is a contract change, not a prediction of every host's behavior.\n");
    Ok((0, out))
}
fn run(args: Vec<String>) -> Result<(u8, String), String> {
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        return Ok((0, HELP.into()));
    }
    if args == ["--version"] {
        return Ok((0, format!("hidweave {}\n", env!("CARGO_PKG_VERSION"))));
    }
    if args == ["demo"] {
        return demo();
    }
    let command = args[0].as_str();
    let mut positional = Vec::new();
    let mut hex = false;
    let mut as_json = false;
    let mut policy = None;
    let mut report = None;
    let mut kind = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--hex" if !hex => hex = true,
            "--json" if !as_json => as_json = true,
            "--policy" if policy.is_none() && command == "gate" => {
                i += 1;
                policy = Some(match args.get(i).map(String::as_str) {
                    Some("strict") => gate::Policy::Strict,
                    Some("add-reports") => gate::Policy::AddReports,
                    _ => return Err("--policy must be strict or add-reports".into()),
                });
            }
            "--report" if report.is_none() => {
                i += 1;
                report = Some(args.get(i).ok_or("--report needs a file")?.as_str());
            }
            "--kind" if kind.is_none() => {
                i += 1;
                kind = Some(match args.get(i).map(String::as_str) {
                    Some("input") => ReportKind::Input,
                    Some("output") => ReportKind::Output,
                    Some("feature") => ReportKind::Feature,
                    _ => return Err("--kind must be input, output, or feature".into()),
                });
            }
            value if value.starts_with('-') => {
                return Err("unknown or repeated option; see --help".into());
            }
            value => positional.push(value),
        }
        i += 1;
    }
    let load = |path| read(path, hex).and_then(|b| parse(&b).map_err(|e| e.to_string()));
    match command {
        "gate" if positional.len() == 2 && report.is_none() && kind.is_none() && !as_json => {
            let policy = policy.unwrap_or(gate::Policy::Strict);
            match load(positional[0]).and_then(|old| load(positional[1]).map(|new| (old, new))) {
                Ok((old, new)) => {
                    let (allowed, artifact) = gate::evaluate(&old, &new, policy);
                    Ok((u8::from(!allowed), artifact + "\n"))
                }
                Err(message) => Ok((2, gate::invalid(policy, &message) + "\n")),
            }
        }
        "inspect" if positional.len() == 1 && report.is_none() && kind.is_none() => {
            let l = load(positional[0])?;
            if as_json {
                return Ok((0, json::layout(&l) + "\n"));
            }
            let mut out =
                format!("contract v{CONTRACT_VERSION}; payload bit offsets exclude report ID\n");
            for r in l.reports().values() {
                out.push_str(&format!(
                    "{} id={}: {} payload bits, {} wire bytes\n",
                    r.key.kind,
                    r.key.id,
                    r.payload_bits,
                    r.wire_bytes()
                ));
                for f in &r.fields {
                    out.push_str(&format!(
                        "  bit {}: {} x {} {} (descriptor byte {})\n",
                        f.bit_offset,
                        f.bit_size,
                        f.count,
                        field_name(f),
                        f.source_offset
                    ));
                }
            }
            Ok((0, out))
        }
        "compare" if positional.len() == 2 && report.is_some() == kind.is_some() => {
            let old = load(positional[0])?;
            let new = load(positional[1])?;
            let bytes = report.map(|p| read(p, hex)).transpose()?;
            let evidence = kind.zip(bytes.as_deref());
            let code = u8::from(!compare(&old, &new).is_empty());
            Ok((
                code,
                if as_json {
                    json::comparison(&old, &new, evidence) + "\n"
                } else {
                    show_compare(&old, &new, evidence)
                },
            ))
        }
        "decode" if positional.len() == 2 && report.is_none() && kind.is_some() => {
            let l = load(positional[0])?;
            let bytes = read(positional[1], hex)?;
            let result = decode(&l, kind.expect("checked kind"), &bytes);
            let code = if result.is_err() { 2 } else { 0 };
            Ok((
                code,
                if as_json {
                    format!(
                        "{{\"contract_version\":{CONTRACT_VERSION},\"decoded\":{}}}\n",
                        json::decoded(&result)
                    )
                } else {
                    show_values(result)
                },
            ))
        }
        _ => Err("invalid command or arguments; see --help".into()),
    }
}
fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok((code, out)) => match io::stdout().lock().write_all(out.as_bytes()) {
            Ok(()) => ExitCode::from(code),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
            Err(_) => ExitCode::from(2),
        },
        Err(message) => {
            let _ = writeln!(io::stderr().lock(), "hidweave: {message}");
            ExitCode::from(2)
        }
    }
}
