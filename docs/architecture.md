# Architecture

```text
bounded regular-file read / caller-provided bytes
                 |
          strict hex (optional)
                 |
   HID item parser: globals + locals + collection stacks
                 |
       immutable validated Layout
           /          |          \
       inspect      compare      decode saved report
           \          |          /
           versioned JSON / text CLI
```

The library has no filesystem or network calls. CLI I/O is a separate boundary.
The parser uses slices and checked arithmetic; expansion budgets cover usages,
fields, slots, report bits and nesting. No recursion is needed. A malformed or
unsupported item terminates interpretation before a Layout can escape.

Layouts own their data. Fields and reports are exposed through immutable
references; callers cannot construct a Layout with invalid decode invariants.
No shared mutable state or caches exist. Layouts can be shared across threads
under ordinary Rust ownership rules. Comparison returns indices into its source
layouts, avoiding duplication of large array maps. JSON snapshots are generated
only at the presentation boundary.

Comparison uses ordered maps for stable report ordering and field identity queues
for deterministic occurrence matching. Decode reads at most 32 bits per data
slot. Parsing is linear in bytes plus bounded expanded output; matching costs
roughly O(fields log fields × collection depth), plus comparison of retained
array maps. No operation depends on attacker-specified allocation sizes without
a prior bound.

Failure model: malformed input, unsupported semantics, resource exhaustion by
declared dimensions, mismatched report framing, unreadable files, and output I/O
errors. OS allocation failure is not recoverable. CLI refuses nonregular inputs
as observed by metadata and bounds reads even if a regular file grows. It does
not provide a filesystem sandbox against concurrent hostile path replacement.

The core source modules separate model, parsing, comparison, decoding, JSON and
CLI. The versioned contract is independent of presentation and does not depend on
an OS HID stack, vendor usage database or external executable.
