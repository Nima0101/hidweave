# Contract v1

The crate follows semantic versioning. Breaking API changes before 1.0 increment
the minor version. JSON and comparison meaning are independently identified by
`contract_version: 1`; changing equality semantics requires a contract version
change. Additive diagnostic fields may be introduced within v1. Consumers should
ignore unknown object keys, but treat unknown contract versions as unsupported.

## Input profile

HID report descriptor bytes follow USB HID 1.11 short-item syntax. One descriptor
represents one interface. Little-endian integer payloads are 0, 1, 2 or 4 bytes.
All fields must be inside collections. Collections require one explicit usage;
report data require explicit logical bounds and at least one usage. Reserved
collection types, global/local/Main tags and flag bits are rejected. Explicit
Push/Pop and collection stacks must be balanced. Trailing unused globals are
permitted; dangling locals are errors.

Supported globals: Usage Page, logical/physical minima and maxima, Unit and Unit
Exponent, Report Size/Count/ID, Push and Pop. Unit exponent accepts signed nibbles
or a sign-extended integer in -8..7. Unit values retain their HID nibbles rather
than being converted to SI values. Supported locals: Usage and same-page ordered
Usage Minimum/Maximum ranges. Ranges expand in encounter order. A Usage between
an unfinished minimum/maximum pair is rejected. Local state resets after every
Main item; global state persists. Local string, designator and delimiter items,
long items and buffered-byte data are unsupported, even if apparently irrelevant.

Data slots are 1..32 bits. Constant padding may be wider. Minima are signed;
maxima use signed interpretation when the active corresponding minimum is
negative, otherwise unsigned. Logical bounds must fit the field width. If either
physical bound is unspecified, or both are zero, both effective physical bounds
default to the logical pair. Physical scaling is retained but not applied during
decode. Main flags are compared verbatim for data. Constant fields are treated as
padding; their usage, flags and global interpretation do not participate.

Any Report ID declaration anywhere requires all reports to have nonzero IDs.
ID values are one byte. Offsets accumulate independently for `(kind, id)`, with
kind ordered Input, Output, Feature. Payload coordinates exclude the ID prefix.
Wire length is ceiling(payload bits / 8), plus one for numbered reports.

## Normalization and equality

Variable fields expand to one value per usage; when fewer usages than values are
declared, the final usage repeats. Excess variable usages are rejected. Array
groups preserve slot width/count and their entire ordered usage list. For an
in-range selector `v`, usage-list index is `v - logical_min`; a missing entry is
`unmapped`, never guessed. Array grouping remains significant: v1 is deliberately
conservative and does not prove all possible array equivalences.

Adjacent padding runs in a report merge. Their original grouping and interpretation
do not matter. All data semantics, collection route, bit width/count and position
do matter. Collections are identified by type, fully qualified usage, and
zero-based sibling ordinal. Identical sibling collections are therefore distinct.
Empty collection existence is not separately compared; adding one before a data
collection can change its route ordinal and conservatively cause a difference.

Reports match by kind/ID. Fields match first by collection route and variable
usage (arrays by route, padding by offset), then occurrence order. Remaining
fields at the same bit offset match to explain meaning changes. Remaining unmatched
fields are additions/removals. This deterministic matching is an explanation
heuristic, not a claim about the author's intended rename. Equality still requires
all matched normalized fields and payload lengths to agree; matching cannot hide
a moved or reinterpreted field. Source offsets do not affect equality.

## Errors and resource bounds

| Resource | Limit |
|---|---:|
| Binary descriptor | 65,535 bytes |
| Hex text | 1 MiB |
| Collection/global stack depth | 32 each |
| Expanded local usage list | 4,096 |
| Total retained array usage entries | 65,536 |
| Normalized fields | 4,096 |
| Total decoded data slots across layout | 4,096 |
| Payload bits per report | 65,536 |
| Report Count per item | 4,096 |

Budgets are checked before expansion/allocation. Memory and work are bounded by
these limits, not just file length. A library parse error has `kind`, `code`,
`offset` and `message`. Categories distinguish malformed, unsupported and limit
failures. Report framing errors are separate. The library never recovers by
skipping an unknown construct and returning a partial layout.

## Report evidence

`decode` takes a kind and one complete report, including ID only if numbered.
It requires exact byte length; final unused bits in the rounded byte are ignored.
It returns raw logical values, usage and state for each data slot. States are
`value`, `null` (outside bounds with Null State flag), `out_of_range`, and `unmapped`.
These are evidence, not reconstructed keyboard events or physical unit conversions.
`decode` returns success for a correctly framed report containing invalid values;
inspect each value's state. Padding is omitted.

## JSON and exit status

`inspect --json` emits `{contract_version, reports}`; each report has kind, id,
payload_bits, wire_bytes and fields. Fields contain bit_offset, bit_size, count,
source_offset, meaning and semantics. Meaning is padding, variable with `usage`,
or array with `usages`. Usages are unsigned integers `(page << 16) | usage_id`.
Physical/logical integers are at most 32-bit in magnitude and JSON-safe.

`compare --json` emits `{contract_version, equal, changes, evidence}`. Each change
has report kind/id, change name, changed properties, old/new payload lengths,
and old/new field snapshots or null. `evidence` is null unless a report is supplied;
then it contains old/new objects, each with `values` or `error`. No raw input bytes
or filesystem paths are emitted. Array and collection ordering is significant.

`decode --json` emits `{contract_version, decoded}`. Decode errors use the same
structured error shape as comparison evidence. CLI argument/read/descriptor errors
are plain text on stderr, even with `--json`; stdout is empty and exit status is 2.

- 0: equal comparison, inspection, successfully framed decode, help or demo.
- 1: any descriptor contract change, including added reports/fields.
- 2: invalid command, file, descriptor, resource limit, unsupported semantics or
  standalone report framing error.

In comparison, evidence failures are included in the report and do not override
the equality result or exit status. The demo exits 0 because it intentionally
demonstrates a detected regression. Outputs are deterministic for fixed inputs
and a fixed contract version; no clock, locale or host HID database is consulted.
