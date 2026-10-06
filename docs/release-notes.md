hidweave 0.1.0 compares HID report-descriptor contracts and explains how the same
saved report changes meaning across firmware versions.

- Safe Rust library and CLI with no third-party production dependencies.
- Variable/array/padding normalization, field matching and source-byte diagnostics.
- Contract v1 JSON, explicit unsupported semantics and bounded input expansion.
- Offline demo: `hidweave demo`.

Download the archive for your platform and check it against `SHA256SUMS`.
Linux binaries are built on Ubuntu 24.04; older glibc distributions are not claimed.
macOS binaries target Apple Silicon; Windows binaries target x86-64.
Source installation is documented in the README.

Equality is limited to the supported report contract. It does not certify host,
device, boot-protocol or transport compatibility. See `docs/contract.md` and
`docs/threat-model.md` for the supported subset and limits.
