//! CLI exit and demonstration tests.
use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hidweave"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn demonstration_and_ci_exit_contract() {
    let demo = run(&["demo"]);
    assert!(demo.status.success());
    let text = String::from_utf8(demo.stdout).unwrap();
    assert!(text.contains("contract v1: equal"));
    assert!(text.contains("X (0001:0030) = 2"));
    assert_eq!(
        run(&[
            "compare",
            "examples/axis-old.hex",
            "examples/axis-equivalent.hex",
            "--hex"
        ])
        .status
        .code(),
        Some(0)
    );
    let diff = run(&[
        "compare",
        "examples/axis-old.hex",
        "examples/axis-swapped.hex",
        "--hex",
        "--json",
        "--report",
        "examples/axis-report.hex",
        "--kind",
        "input",
    ]);
    assert_eq!(diff.status.code(), Some(1));
    assert!(
        String::from_utf8(diff.stdout)
            .unwrap()
            .contains("\"equal\":false")
    );
    assert_eq!(
        run(&[
            "compare",
            "examples/delimiter.hex",
            "examples/delimiter.hex",
            "--hex"
        ])
        .status
        .code(),
        Some(2)
    );
}
#[test]
fn cli_arguments_and_input_fail_closed() {
    for args in [
        vec!["decode"],
        vec!["inspect", "examples/axis-old.hex", "--hex", "--hex"],
        vec![
            "compare",
            "examples/axis-old.hex",
            "examples/axis-old.hex",
            "--hex",
            "--report",
            "examples/axis-report.hex",
        ],
        vec!["inspect", "examples/absent"],
        vec!["inspect", "examples"],
        vec!["inspect", "examples/axis-old.hex"],
    ] {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty());
    }
}
#[test]
fn decode_and_help() {
    assert!(run(&["--help"]).status.success());
    assert!(run(&["--version"]).status.success());
    let out = run(&[
        "decode",
        "examples/axis-old.hex",
        "examples/axis-report.hex",
        "--kind",
        "input",
        "--hex",
        "--json",
    ]);
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("\"usage\":65584")
    );
}
