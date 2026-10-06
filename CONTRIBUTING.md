# Contributing

Start with a small reproducible descriptor pair and the expected contract change.
The project is scoped to offline report-contract analysis. A good first contribution
is a synthetic regression test, a clarified diagnostic, or a documented public
descriptor fixture with provenance and redistribution permission.

## Development

Rust 1.85+; no third-party production dependencies. Standard verification:

```sh
cargo fmt --all -- --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --locked --offline
cargo run --locked --offline --example consumer
python3 scripts/check_json.py
```

`check_json.py` uses Python 3.9+ standard library and builds the CLI if needed.
Run the bounded mutation/property suite with `cargo test`; it is deterministic.
For coverage-guided fuzzing, see [fuzz/README.md](fuzz/README.md).

AddressSanitizer on Linux/macOS with a nightly toolchain that includes its runtime:

```sh
rustup toolchain install nightly --profile minimal
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu --lib --tests
```

On Apple Silicon, use target `aarch64-apple-darwin`. Keep sanitizer output in a
separate target directory if switching configurations frequently. Rust std is not
rebuilt by this command; it instruments project code, not every standard-library
operation. LeakSanitizer availability depends on the host platform.

Optional independent parser experiment:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install hid-parser==0.1.0
.venv/bin/python scripts/differential.py
```

On Windows use `.venv/Scripts/python.exe`. This oracle is test-only and covers
its documented shared subset; differences between parsers need investigation,
not automatic acceptance of either side. The standard build remains offline.

Package consumer check: `python3 scripts/check_consumer.py` packages the crate and
builds an external project against the packaged source. CI also installs the CLI
into a temporary prefix. Run this from a clean Git checkout.

## Changes

For a new semantic feature, cite the specification, define resource bounds and
failure behavior, and test equivalent and changed descriptor pairs. Add negative
tests for malformed/unsupported forms. Do not silently ignore unknown semantics.
Keep source offsets out of equality. Update `docs/contract.md` for observable changes.

Public API changes follow semantic versioning. Comparison/JSON meaning has its own
contract version. No adoption or platform claims without verification. Please do
not commit confidential captures, absolute developer paths or generated fuzz data.
By submitting a contribution you agree it may be distributed under the MIT license.
