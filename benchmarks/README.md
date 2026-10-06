# Small-corpus experiment

Run `cargo bench --bench contracts --locked --offline` from the source repository.
The harness uses `std::hint::black_box` and 100,000 iterations per operation:

- Parse the original synthetic two-axis descriptor (21 bytes, two fields).
- Compare it with the same-size X/Y-swapped descriptor (two changed fields).
- Decode the two-byte report `01 02`.

Initial local run, 2026-10-06: Apple M2, 8 GB RAM, macOS 26.6.2 arm64;
Homebrew rustc 1.98.1, Cargo 1.98.1; Cargo release profile, overflow checks enabled.

| Operation | Mean elapsed ns/op |
|---|---:|
| Parse | 4,559 |
| Compare | 6,927 |
| Decode | 681 |

These are one run's elapsed-time averages, not latency percentiles. Other tool
builds were active, so scheduling contention affects the result. No warmup,
confidence interval, allocator profiling or comparison with other tools is claimed.
The tiny corpus demonstrates reproducibility of the measurement, not performance
on large real-world descriptors. Full filesystem/CLI startup cost is excluded.

Correctness experiments are more important for this tool: the same-size axis swap
must produce a difference while equivalent factoring must not. See
[verification](../docs/verification.md) for exhaustive bitfield checks and a
bounded independent-parser experiment.
