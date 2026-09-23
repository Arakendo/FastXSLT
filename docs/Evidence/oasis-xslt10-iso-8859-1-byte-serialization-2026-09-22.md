# OASIS XSLT 1.0 ISO-8859-1 Byte Serialization

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can the physical byte-output lane support the full ISO-8859-1 repertoire and
XML-safe references for other characters without weakening byte limits,
misrepresenting text output, or changing the UTF-8 string lane?

## Method

- Inventory the nine cases stopped at `FXSR1006` after semantic execution.
- Encode ISO-8859-1 characters directly as one byte.
- For XML/XHTML character data, attribute values, and CDATA content, emit a
  numeric character reference when the selected encoding cannot represent the
  codepoint.
- Reject unrepresentable codepoints in markup names, comments, processing
  instructions, declarations, and literal text/HTML output rather than
  silently changing their semantics.
- Continue charging serialized-byte expansion and enforcing the exact final
  byte limit.
- Decode declaration-less actual and expected corpus output using the compiled
  stylesheet's selected ISO-8859-1 encoding.
- Rerun the unchanged 3,173-case catalog.

## Result

All nine former `FXSR1006` cases now execute, eliminating that execution
frontier. Three unchanged Lotus cases become exact:

- `numberformat_numberformat06` emits the unrepresentable per-mille character
  as an XML numeric character reference;
- `output_output26` emits Latin-1 `U+00AC` and `U+00A9` directly; and
- `output_output86` additionally emits `U+00FF` directly.

Six newly executed cases remain visibly unresolved: one suite-declared
execution-error case now exposes an earlier permissive formatting boundary,
and five success cases reach ordinary comparison mismatches. These are not
credited as passes.

The conserved totals are 2,140 initialized cases, 995 initialization failures,
2,073 successful executions, 67 execution failures, 1,924 exact XML-semantic
matches, and 61 mismatches. The strict compatibility lower bound is
`1,924 / 3,173 = 60.64%`.

## Boundaries

- The supported string-returning serializer remains UTF-8. This evidence is
  for the private bounded physical byte lane.
- Numeric references are emitted only in XML contexts where references retain
  character semantics. Text and HTML output do not receive XML reference
  substitution.
- ISO-8859-1 selection does not imply Windows-1252. Several archival Microsoft
  expected files contain platform-specific bytes and remain visible comparator
  or semantic obligations.
- The newly visible formatting failures must be fixed in formatting semantics,
  not hidden in encoding.

## Reproduction

```powershell
cargo test -p fastxslt --all-features iso_8859_1
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXSR1006
./scripts/measure-oasis-xslt10.ps1 -TraceCase numberformat_numberformat06
./scripts/measure-oasis-xslt10.ps1 -TraceCase output_output26
./scripts/measure-oasis-xslt10.ps1 -TraceCase output_output86
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
