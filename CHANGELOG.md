# Changelog

## Unreleased

- Add `gate` with deterministic schema-v1 review JSON and explicit `strict`/`add-reports` policies. Existing HID normalization and comparison semantics remain unchanged.
- Add an executable CI runner and composite GitHub Action, with positive and negative tests for equivalence, semantic changes, unsupported input, report additions/removals and stable output.
- Verify packaging from an exact source snapshot so pre-commit gates can validate unpublished edits without bypassing Cargo's package build.

## 0.1.0 — 2026-10-07

- Offline HID report-descriptor inspection and contract comparison.
- Same-report decoding under old/new descriptors, including arrays and null states.
- Normalization of equivalent variable grouping, usage ranges and padding.
- Bounded safe Rust parser, explicit unsupported semantics, contract v1 JSON.
- Synthetic demo, deterministic property tests, fuzz target, consumer example and CI.
