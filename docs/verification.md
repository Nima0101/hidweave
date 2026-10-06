# Verification and its limits

## Deterministic coverage

`cargo test --locked --offline` runs unit/integration, CLI and documentation tests.
The parser tests exercise truncation, bad stacks, unsupported semantics, ID framing,
logical signedness, arrays, physical defaults and expansion limits. A pairwise
property checks that empty differences agree with an independent direct comparison
of normalized ordered fields, including repeated usages and arrays.

The bit extraction oracle exhaustively enumerates all values for widths 1..8,
signed and unsigned, at each bit offset 0..7. Its whole-integer shift/mask method
differs from the production decoder's per-bit extraction. Full 32-bit boundaries
are tested separately. This is finite coverage, not an all-input proof.

A deterministic 30,000-case mutation smoke test starts from original axis and
keyboard descriptors. The separate libFuzzer target adds coverage-guided parsing,
self-comparison and report decoding under AddressSanitizer. Neither establishes
full HID conformance. CI runs a short fuzz campaign; contributors can run longer ones.

## Independent parser experiment

`scripts/differential.py` uses the separately installed `hid-parser==0.1.0` as an
independent implementation. The recorded experiment compares 112 variable-field
layouts (width 1..7, offset 0..7, signed/unsigned bounds) and six byte-aligned
unsigned reports. Those shared-subset results agree.

Two observed differences are pinned explicitly in the script rather than hidden:

| Case | hidweave | hid-parser 0.1.0 |
|---|---:|---:|
| First 1-bit field at bit 0, report `03` | 1 | 0 |
| Signed 8-bit field, logical -127..127, report byte `ff` | -1 | 255 |

The complete synthetic descriptors are in the script. HID 1.11 section 5.8
specifies least-significant-bit ordering and two's-complement signed values;
hidweave's independent exhaustive extraction tests check those rules. The Python
parser is therefore an oracle only for the stated shared subset, not a universal
authority. This experiment does not characterize current unreleased upstream
behavior or other parsers, and no upstream bug report is implied.

## Reproduction

See [CONTRIBUTING.md](../CONTRIBUTING.md) for test, sanitizer, independent parser,
package and fuzz commands. The release build has integer overflow checks enabled.
The library and CLI use no third-party production dependencies; fuzz and independent
oracle tools are isolated test dependencies. CLI JSON is parsed and checked using
Python's independent standard JSON parser in `scripts/check_json.py`.

Platform claims are limited to recorded local or completed CI runs. Release
archives carry platform labels; a successful cross-compilation alone does not
establish runtime support. No hardware, real device, console or OS HID-driver
compatibility has been tested or claimed.
