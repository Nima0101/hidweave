# Firmware contract gate v1

`hidweave gate BASELINE CANDIDATE [--hex] [--policy strict|add-reports]`
produces deterministic JSON matching [gate.schema.json](gate.schema.json).
Exit 0 means policy permits the change; 1 denies it; 2 indicates invalid or
unsupported input. Invocation syntax errors exit 2 on stderr. No policy makes
unsupported descriptors pass. Existing compare/inspect/decode semantics are unchanged.

`strict` permits only equal normalized report contracts. `add-reports` additionally
permits whole new report keys; it still denies removal, changes to existing fields,
and length changes. Adding fields to an existing report is not a report addition.
Allowed differences remain visible in the artifact. Neither policy proves host or
device compatibility. Schema and semantic contract versions are independent;
consumers must reject unknown versions and check exit status.

The JSON artifact is the review output (no SARIF claim). Findings identify report
kind/ID, change category, changed properties and policy disposition. Input errors
are a separate status, never represented by an empty passing comparison.

## Concrete CI integration

Use an immutable revision of this repository with the composite action at
`Nima0101/hidweave/integrations/action@<reviewed-commit-sha>`. The runner needs Rust
1.85+, Python 3 and Bash. It builds the pinned checkout offline without production
dependencies. Supply `baseline`, `candidate`, `artifact`, `policy`, and optionally
`hex: 'true'`. The step's exit status is the gate result. Preserve the artifact
with your CI artifact uploader even when the gate fails. Pin that uploader too.

For any other CI provider, the exact runner is executable directly:

```sh
cargo build --locked --offline
python3 integrations/ci_gate.py target/debug/hidweave examples/axis-old.hex examples/axis-equivalent.hex review.json --hex
```

The baseline, policy, tool pin and workflow are trusted review inputs. Store them
on a protected branch; do not let a candidate change its own baseline unnoticed.
The action runs no firmware code, accesses no device and requires no secret.
Paths are passed through environment variables and process argument lists, never
interpolated as executable shell text. Input directories and artifact destination
are caller-controlled, not a filesystem sandbox.

`integrations/scripts/verify-local.sh` runs the actual CI runner through harmless
refactors, semantic changes, added/removed reports and unsupported descriptors,
checking byte-identical repeated output and propagated failure status. Hosted
execution of the action and other OS support require separate CI evidence.
