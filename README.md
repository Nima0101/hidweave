# hidweave

**Catch HID report-contract changes before firmware and host software disagree.**

hidweave compares two HID report descriptors, identifies changes to field locations
and declared meaning, and decodes the same saved report under both versions.
It is an offline Rust library and CLI for firmware maintainers, device SDK authors,
and applications that depend on fixed HID report layouts.

A two-byte report can stay two bytes long while X and Y silently trade places:

```text
contract v1: changed
input id=0 FieldChanged: X (0001:0030) bit 0 -> X (0001:0030) bit 8
input id=0 FieldChanged: Y (0001:0031) bit 8 -> Y (0001:0031) bit 0

same report, old interpretation: X = 1, Y = 2
same report, new interpretation: Y = 1, X = 2
```

The bundled example is synthetic. Real report-layout regressions happen:
[EdgeTX #6320](https://github.com/EdgeTX/edgetx/issues/6320) documents a firmware
update that changed a joystick report and disrupted console use. That issue was
fixed upstream; hidweave does not predict console compatibility.

## Five-minute quickstart

For a [release binary](https://github.com/Nima0101/hidweave/releases), extract the
archive and run `./hidweave demo` (PowerShell: `.\hidweave.exe demo`). Verify the
archive against the release's `SHA256SUMS`. Source build and test instructions:

Install [Rust](https://www.rust-lang.org/tools/install) (1.85 or newer), then:

```sh
git clone https://github.com/Nima0101/hidweave.git
cd hidweave
cargo build --release --locked --offline
cargo run --release --locked --offline -- demo
cargo test --locked --offline
```

The demo first compares differently encoded but equivalent descriptors. It then
introduces an X/Y usage swap and shows both interpretations of `01 02`.
No hardware, device permissions, downloaded corpus, or runtime dependencies.

Try the actual CI command (the intentional regression exits **1**):

```sh
cargo run --release --locked --offline -- compare examples/axis-old.hex examples/axis-swapped.hex --hex --report examples/axis-report.hex --kind input
```

Compare a harmless refactoring (exits **0**):

```sh
cargo run --release --locked --offline -- compare examples/axis-old.hex examples/axis-equivalent.hex --hex
```

Unsupported semantics fail explicitly (exits **2**, even compared with itself):

```sh
cargo run --release --locked --offline -- compare examples/delimiter.hex examples/delimiter.hex --hex
```

## Use with your firmware

Save the report descriptor emitted by each firmware build as a raw binary file.
Use **one interface's report descriptor**, not the USB device/configuration descriptor.

```sh
cargo install --path . --locked --offline
hidweave compare old.bin new.bin --json
hidweave inspect new.bin --json
hidweave decode new.bin report.bin --kind input --json
```

`--hex` applies to **all input files** and accepts whitespace-separated byte pairs
and `#` comments. C arrays and hexdump address columns are not accepted. A saved
wire report includes its ID byte only when the descriptor uses report IDs;
remove any API-specific leading zero for an unnumbered report. Payload bit offsets
exclude the ID byte. Diagnostics also identify the descriptor's Main-item byte.

Exit status is `0` for an equal contract/success, `1` for any contract change,
and `2` for an invalid, unsupported, over-limit or unreadable input. A comparison's
optional report evidence can contain a decode error without changing the
descriptor comparison status. See the [versioned contract](docs/contract.md).

## What it compares

- Report kind and ID, payload bits, and wire length.
- Variable usage, bit position, width, signed range, flags, units and physical bounds.
- Array selector maps and slot counts; arrays remain arrays, not pretend variables.
- Collection routes and repeated usage occurrences.
- Equivalent global factoring, variable grouping, usage-range encodings and padding runs.

Matching follows collection/usage/occurrence before falling back to bit position,
so an inserted axis does not turn every later field into a spurious rename.
Source offsets are diagnostic provenance, not part of equality.

**Equal means equal under contract v1.** HID is self-describing, and many hosts
adapt to a changed descriptor. A difference is a review signal for a pinned
contract, not proof that every host breaks. Equality does not establish device
behavior, OS quirks, USB endpoint validity, boot-protocol compliance, or transport
compatibility. Empty collection topology is outside the report contract, though
collection sibling positions remain part of data-field routes.

The profile supports bounded HID 1.11 short items, global Push/Pop, variable and
array fields up to 32 bits, numbered and unnumbered reports, and constant padding.
Long items, delimiters, designators, string associations, buffered bytes and
reserved semantics are rejected. Limits and normalization details are explicit in
[the contract](docs/contract.md); this is not a complete HID conformance checker.

## Library

```rust
use hidweave::{compare, decode, parse, ReportKind};

fn check(old_bytes: &[u8], new_bytes: &[u8], report: &[u8])
    -> Result<(), hidweave::Error>
{
    let old = parse(old_bytes)?;
    let new = parse(new_bytes)?;
    let changes = compare(&old, &new);
    let before = decode(&old, ReportKind::Input, report)?;
    let after = decode(&new, ReportKind::Input, report)?;
    println!("{} changes; before={before:?}, after={after:?}", changes.len());
    Ok(())
}
```

For a Git dependency, pin a release tag. The library has no third-party dependencies
and forbids unsafe Rust. See [examples/consumer.rs](examples/consumer.rs).

## Alternatives and development

[hid-tools](https://gitlab.freedesktop.org/libevdev/hid-tools) is a broader
decode/capture/replay workflow; [hidrd](https://github.com/DIGImend/hidrd) converts
descriptor formats; [Waratah](https://github.com/microsoft/hidtools) and
[hid-rp](https://github.com/IntergatedCircuits/hid-rp) help author descriptors.
[hidreport](https://docs.rs/hidreport/) and [hidparser](https://docs.rs/hidparser/)
already offer Rust parsing. hidweave adds an explicit artifact-pair comparison
contract and same-byte interpretation evidence; it complements those tools.

Builds, tests, lint, packaged-library use and CLI installation are verified on
Ubuntu 24.04 x86_64, macOS 14 arm64 and Windows Server 2022 x86_64 in CI.
Local verification also covers macOS 26.6.2 arm64. See [verification](docs/verification.md)
for scope and limitations.

Read the [architecture](docs/architecture.md), [design rationale](docs/design-rationale.md),
[engineering deep dive](docs/engineering-deep-dive.md), [threat model](docs/threat-model.md),
[verification](docs/verification.md), [benchmarks](benchmarks/README.md), and
[contributing guide](CONTRIBUTING.md).
Security reports: [SECURITY.md](SECURITY.md). License: [MIT](LICENSE).
