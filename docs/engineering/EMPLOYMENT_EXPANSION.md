# Employment-signal expansion

hidweave remains a precise HID report-contract analyzer. The expansion productizes it for firmware/device CI without changing its core semantic contract.

## Target evidence
Add stable machine-readable review output, a compatibility policy layer and a concrete reusable CI integration. SARIF is preferred when it maps cleanly to findings; otherwise use a versioned artifact with an explicit schema. Policy must distinguish allowed representation-only changes from semantic contract changes without hiding unsupported semantics.

## Done means
A firmware repository can pin hidweave, compare a baseline descriptor with a candidate, obtain deterministic review output, and fail CI on disallowed semantic changes. Tests prove equivalent descriptors pass, semantic changes fail, unsupported inputs fail explicitly, and the CI integration itself is exercised. Existing parser hardening and differential checks remain intact.
