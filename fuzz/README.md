# Coverage-guided parser fuzzing

The fuzz-only workspace uses libFuzzer and is not a production dependency.
Install a nightly Rust toolchain and `cargo-fuzz`, then from the repository root:

```sh
python3 scripts/seed_fuzz.py
cargo +nightly fuzz run descriptor -- -max_total_time=30 -max_len=65535 -rss_limit_mb=1024
```

The target exercises binary and hex parsing, self-comparison, report construction,
arbitrary report decoding, and contiguous field invariants. AddressSanitizer is
enabled by cargo-fuzz. Keep minimized regressions in deterministic tests; do not
commit private device captures. The generated corpus and artifacts are ignored.
Long campaigns should use an external process budget and retain their logs.

The 1 GiB process cap includes the fuzzer corpus, sanitizer metadata and retained
freed allocations, not just parser state. A Linux smoke run exceeded a 512 MiB
cap with about 25 MiB of live allocations and 244 MiB in ASan quarantine. The
larger cap preserves default sanitizer detection settings; parser input and
expansion limits are unchanged. This is a test-process budget, not a measured
upper bound on the library's memory use.
