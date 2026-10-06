# Threat model

## Trust boundary

Descriptor files and saved reports may be malicious. The attacker controls bytes,
declared counts, ranges, nesting and file size. The caller controls file selection
and the execution environment. hidweave does not enumerate hardware, open a HID
device, capture input, send reports, execute input as code, or make network calls.
No administrator permissions or credentials are needed.

## Defenses

- Safe Rust only, enforced with `unsafe_code = forbid`.
- Explicit input and aggregate expansion limits; checked report-size arithmetic.
- Unknown, reserved and unsupported semantics fail before comparison.
- Immutable validated layouts prevent library consumers from bypassing parser
  invariants before decoding.
- Exact report framing; malformed logical values remain visible as value states.
- Errors omit raw bytes and filesystem paths. No telemetry or persistent logs.
- JSON escapes strings; input does not supply arbitrary terminal labels. Numeric
  usages are rendered by the tool. Reports can still contain sensitive values:
  redirecting output is an intentional export by the user.
- No third-party production dependencies or build scripts. CI scans dependencies,
  source/workflows, and tests parsing with adversarial inputs.

## Residual risks and exclusions

The main risk is **false reassurance**, not device compromise. A parser bug could
miss or misinterpret a change. Equality covers only the documented report contract;
real firmware can emit bytes that contradict its descriptor. Host drivers, quirks,
USB configuration/endpoint descriptors, BLE transport, boot protocols and vendor
application semantics are outside scope.

Limits bound work per invocation, not the number of concurrent invocations. A
service embedding the library must provide its own process/memory/request budget.
The CLI's regular-file check is not race-proof against hostile path replacement,
and it follows explicitly selected symlinks. Use trusted input directories or an
external sandbox where that matters. No recursive traversal or archive extraction
is performed. Out-of-memory and OS faults remain outside recoverable error handling.

Saved input reports may expose keystrokes or other sensitive values. Use synthetic
or deliberately selected reports and review diagnostics before sharing. hidweave
does not sanitize captures and does not claim to do so.

Report vulnerabilities through [SECURITY.md](../SECURITY.md).
