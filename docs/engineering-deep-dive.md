# Engineering deep dive

## The invariant

Every decoded slot must use the global state active at its Main item, the local
usages attached to that item, and the correct `(report kind, report ID)` payload
coordinate system. Unknown semantics cannot reach an “equal” result.

HID is a small stateful language. Moving a Usage Page changes later short usages;
Push/Pop restores globals but not locals; every Main item clears locals. The
same Report ID can reappear, and Input/Output/Feature offsets accumulate separately.
Maximum signedness depends on the active minimum. Physical bounds have paired
defaults. These interactions are why comparing bytes or lengths is insufficient.

## Equality versus explanatory matching

The parser normalizes equivalent representation choices while retaining meaning.
Variable groups expand, contiguous padding merges, and ranges become ordered
usages. Source offsets survive as provenance but are excluded from equality.

Matching is not equality. Matching helps explain a moved X axis as a move. It uses
route/usage/occurrence, then offset for unmatched fields. A match still compares
all contract properties. Duplicate usages cannot be magically assigned semantic
identity; occurrence order is the explicit tie-breaker. An insertion may be
ambiguous, but cannot conceal changed bit positions.

Arrays illustrate the limit of normalization. A selector is an index relative to
the logical minimum, not necessarily the numeric usage itself. Slot order in a
keyboard array may be semantically unimportant to an application, but the report
contract retains group boundaries and ordering. No general theorem of application
equivalence is claimed.

## Verification strategy

Deterministic tests cover equivalent encodings, same-size meaning changes, report
IDs and type isolation, arrays/null values, signed and unsigned limits, physical
defaults, nested state, unaligned extraction and malformed inputs. The independent
small-field oracle assembles a whole integer and shifts/masks it; production decode
reads individual bits. Exhaustive enumeration checks every value for widths 1..8,
signed and unsigned, at every starting bit offset 0..7.

A fixed-seed mutation smoke test exercises 30,000 corrupted descriptors. Successful
parses must compare equal to themselves and decode within bounds. This is bounded
parser hardening, not proof of correctness or a substitute for coverage-guided
fuzzing. Additional fuzz instructions and verification commands live in
[CONTRIBUTING.md](../CONTRIBUTING.md).

## Growth without losing focus

Useful contributions include independently sourced public descriptor fixtures,
supported local string/designator semantics, better evidence renderers, optional
usage labels, and integration with descriptor-producing builds. Each new semantic
feature needs a reference, budget, failure behavior and paired positive/negative
tests. Live device control and automatic firmware repair would change the threat
model and are outside the current roadmap.
