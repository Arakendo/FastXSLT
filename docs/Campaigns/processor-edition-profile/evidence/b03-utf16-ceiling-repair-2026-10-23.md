# B03-C01: bounded shared UTF-16 serialization repair

Date: 2026-10-23. Maintainer authorized the repair after the
[candidate audit](b03-utf16-ceiling-blocker-2026-10-23.md).
Scope: existing shared UTF-16 byte serialization only. Selected-profile UTF-16
admission, other encoders, public contracts and default behavior are excluded.
This control repair does not increment the semantic breadth counter (B03 1/3).

## Implementation

The shared byte encoder now reserves the BOM/declaration's encoded length from
the caller's ceiling before constructing the body. For a remaining encoded-byte
budget B, UTF-8 staging is limited to `3 * floor(B / 2)` (saturating at the machine
limit): a BMP scalar uses at most three UTF-8 bytes per UTF-16 unit, while a
supplementary scalar uses four UTF-8 bytes for two units. This permits exact-sized
BMP-heavy output without using an unlimited staging ceiling.

After bounded shared serialization, the encoder counts exact UTF-16 bytes with
checked remaining capacity and cooperative cancellation. Only after admission
and the existing expansion charge does it allocate the final BOM-prefixed byte
vector. Encoding also polls cancellation. The concatenated declaration/body
string and separate encoded intermediate vector are removed.

Successful `SerializedByte` units retain the existing accounting:
UTF-8 body charges plus `max(0, final bytes - UTF-8 body bytes)`. Zero-unit
charges poll cancellation without consuming additional units. This is bounded
logical staging and output admission, not an RSS or allocator-capacity guarantee.
The shared string serializer can charge an attempted chunk before rejecting it;
work charges alone are not a measurement of retained allocation.

## Focused evidence

`shared_utf16_bounded_staging_preserves_unicode_and_limits` checks ASCII, Latin,
three-byte BMP and supplementary scalars, XML/text, declaration present/absent,
empty text, BOM, zero/one-byte ceilings, exact/one-less output limits,
`usize::MAX`, exact/one-less work budgets, cancellation and clean reuse.
Expected bytes are independently built using standard UTF-16 encoding.

`shared_utf16_small_ceiling_stops_large_body_before_retention` checks the original
8 KiB/16-byte witness. Its character-chunk emission stops below 64 charged bytes;
the earlier observation was 8,203. Both return FXSR0002 with no partial result.
UTF-16 remains rejected by selected compilation during this repair.

Initial validation caught use of a test-only cancellation helper and a missing
empty-output cancellation observation. The final implementation uses the existing
production charge API, including a cancellation poll before output allocation.

## Validation checkpoint

**B03-C01 closed.** Focused tests and final repository gates pass: fmt, Clippy,
2,322 workspace tests (2,278 core, 22 native, 4 WASM, 18 worker), docs/links,
unsafe-surface and pinned conformance checks. Official WASM build passes.

Both full conserved replays are byte-for-byte identical to their S1 reports:
[selected 491/3,173](post-r0-utf16-repair-selected-report-2026-10-23.txt),
[ordinary 2,504/3,173](post-r0-utf16-repair-baseline-report-2026-10-23.txt).
No disposition changes or regression. The
[run identity](post-r0-utf16-repair-run-identity-2026-10-23.json) pins the source,
toolchain and reports. Unsupported UTF-16 selected admission remains unchanged
at this repair checkpoint. Next breadth must have its own declared scope.
