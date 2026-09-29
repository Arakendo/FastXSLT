# OASIS XSLT 1.0 Literal XML-Whitespace Boundary

Date: 2026-09-28  
Status: Implemented and verified

## Problem

The stylesheet compiler used Rust's Unicode `char::is_whitespace` predicate
when deciding whether a literal stylesheet text node was ignorable. XSLT's
stylesheet-stripping rule is limited to XML whitespace: space, tab, carriage
return, and line feed. A non-breaking space such as U+00A0 is literal result
text and must not disappear.

## Change

- Expose the existing byte-exact XML-whitespace predicate from the private XDM
  owner.
- Reuse it for literal text compilation, adjacent text-run classification,
  meaningful stylesheet children, and required-global-default detection.
- Retain the existing `xml:space` behavior and add a focused compiler
  regression for `&#160;`.

## Result

Three unchanged OASIS cases become exact:

- `Lotus/output_output04#1`;
- `Lotus/copy_copy38#1`; and
- `Microsoft/Output__84428#1`.

The strict lower bound rises from **2,258 / 3,173 (71.16%)** to
**2,261 / 3,173 (71.26%)**. Visible mismatches fall from 22 to 19. The single
remaining comparator gap and all lifecycle, expected-error, policy, and
archival-exclusion counters are unchanged.

## Boundaries

- This does not preserve ordinary indentation made only of XML whitespace when
  the stylesheet rules strip it.
- This does not change source-document whitespace policy or ADR-0012's
  invocation-owned visibility view.
- The shared helper remains a private semantic utility, not a public API.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features literal_text_discards_only_xml_whitespace
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/output_output04'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/copy_copy38'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Output__84428'
./scripts/verify.ps1
```
