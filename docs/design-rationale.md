# Design rationale

## A contract review tool

HID already has capable generators, parsers and replay tools. The missing workflow
we target is a small release gate over two actual descriptor artifacts, followed
by a concrete explanation using one report. Using the emitted descriptor catches
changes introduced by generators or conditional firmware builds without requiring
a new authoring language.

The result says **changed**, not **incompatible**. Self-describing host software
may correctly adapt to a new report. Fixed-layout software may not. The tool does
not know which host assumptions are intentional, so additions also require review.
Nor is equality a device conformance certificate.

## Why Rust and a small independent parser

The core problem is bounded interpretation of untrusted binary bytecode with
state and bit extraction. Safe Rust expresses ownership and integer bounds well,
and yields an easily installed native executable. The implementation forbids
unsafe code and has no third-party runtime or build dependencies.

Existing hidreport/hidparser libraries are valid alternatives. Owning this limited
parser allows resource budgets, precise rejection rules, source-byte provenance,
and normalization to form one auditable contract. That increases our maintenance
responsibility; it is justified only with negative, property and differential
tests. Parser novelty is not the project's claim.

## Conservative semantics

No item with unknown meaning is ignored. Version 1 deliberately excludes less
common local associations and buffered-byte reports. Numeric usage values support
vendor pages without pretending to understand a vendor's private protocol.

Collection sibling ordinals and repeated-usage occurrence order are deterministic,
but cannot infer author intent. Array grouping stays significant. These choices
can produce conservative differences; they avoid unstable or guessed matching.
Adding equivalence rules is a contract change that needs tests and documentation.

## Standards and prior art

- [USB HID 1.11](https://www.usb.org/sites/default/files/hid1_11.pdf), especially
  sections 5.4, 6.2.2 and 8, is the primary format reference.
- [USB-IF tool catalog](https://usb.org/hid) describes Waratah and related tools.
- [Linux HID introduction](https://docs.kernel.org/hid/hidintro.html) describes
  descriptor/report mismatch and the role of host quirks.
- [hid-tools](https://gitlab.freedesktop.org/libevdev/hid-tools),
  [hidrd](https://github.com/DIGImend/hidrd),
  [hidreport](https://docs.rs/hidreport/), and
  [hidparser](https://docs.rs/hidparser/) are substantial existing alternatives.

The implementation and synthetic fixtures are original. Standards are linked,
not redistributed. MIT licensing keeps the library usable in firmware and host
tooling with a straightforward attribution obligation.
