# B03 early stop: UTF-16 ceiling enforcement

Status: **closed after maintainer-authorized bounded repair**; see
[repair and verification](b03-utf16-ceiling-repair-2026-10-23.md).
The following preserves the original stop evidence and disposition request.
Date: 2026-10-23. B03 remains **1/3 complete**. This is a candidate-admission
audit, not a completed breadth slice or an automatic addition to R0's denominator.

## Trigger and evidence

The fresh next-candidate witness `Lotus/output_output80#1` refuses its
`<xsl:output encoding="UTF-16"/>` declaration at 307..338. Before changing that
guard, inspection of the existing shared UTF-16 serializer found:

1. `serialize_utf16_be` calls the string serializer with `usize::MAX`, not the
   caller's result-byte ceiling.
2. It builds the body, combined declaration/body string and UTF-16 encoded
   buffer before calculating/checking `required > byte_limit`.
3. Only after that check does it charge remaining encoding expansion and build
   the final BOM-prefixed result.

Source: [byte encoding owner](../../../../crates/fastxslt/src/runtime/golden_runtime_experiment/byte_encoding.rs),
function `serialize_utf16_be` (inspection-time lines 352–405).
No implementation in that owner was changed during this audit.

A bounded manual probe compiles through the already admitted UTF-8 selected
entry, executes an 8,192-character ASCII body, and clones private output settings
solely to call the existing UTF-16 serializer with a 16-byte ceiling:

```text
UTF16_CEILING_OBSERVATION ceiling=16 payload=8192 serialized_work=8203 failure=FXSR0002
```

The output limit correctly rejects the result. The problem is the complete
intermediate construction and encoding before rejection. The work observation
does not measure RSS or allocator bytes; source inspection establishes the
allocation ordering. A separate finite `SerializedByte` work budget can limit
some work, but it does not make this per-result ceiling an early allocation
bound. No claim of an exploitable crash or returned oversized output is made.

The ignored probe deliberately asserts the existing failure category/code, not
that excessive work is desirable; a repair may reduce the printed charge count.
It also confirms selected compilation still refuses UTF-16.

## Boundary and disposition request

- **Do not admit UTF-16 or start another breadth slice until review.**
- Existing ISO-8859-1 S1 remains complete; its accounting behavior is unchanged.
- This is an existing shared byte-serializer concern, not a newly introduced
  selected-profile regression. UTF-16 is outside the promoted selected scope,
  so no existing R0 closure is automatically reopened.
- Record provisional candidate blocker **B03-C01**, outside the closed R0
  denominator. No AR/ADR, public contract or promotion change was made.
- Recommended disposition: authorize a bounded shared-serializer repair/audit
  under existing resource-control ownership, with preserved UTF-16 byte output,
  early admission/charging, cancellation, exact/one-less limits and safe cleanup.
  Return for architecture review only if that requires changing accepted owners
  or accounting semantics. Do not trade controls for corpus points.

No corpus rerun is claimed for the probe-only edit. Last full measurements
remain **491/3,173 selected**, **2,504/3,173 ordinary**, with unchanged S1 report
identities. No new exact case is credited by this audit.

## Reproduction and verification

Base commit `b5935c8e` plus pending campaign changes. Diagnostic test:
[selected encoding tests](../../../../crates/fastxslt/src/runtime/resource_compiler/selected_encoding_tests.rs).

```powershell
cargo test -p fastxslt --all-features selected_encoding_utf16_ceiling_observation -- --ignored --nocapture
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10 -TraceCase output_output80
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

The diagnostic probe passed. Post-probe repository verification passed: fmt,
Clippy, **2,320 workspace tests** (2,276 core, 22 native, 4 WASM, 18 worker),
docs/links, unsafe-surface and pinned conformance checks. The diagnostic remains
ignored by default (run explicitly above); official WASM build passed. These
green gates do not resolve B03-C01 or authorize UTF-16 selected admission.
