# OASIS XSLT 1.0 Namespace-Fixup Reference

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Microsoft `BVTs_bvt057` exercises prefix conflicts between a literal result
element and a computed attribute. Its stylesheet constructs literal `foo:p`
while `foo` is bound to `urn:foo`, then adds `foo:attr` from an
`xsl:attribute` site where `foo` is bound to `urn:foo2`.

FastXSLT preserves the element's expanded name `{urn:foo}p` and allocates a
different result prefix for the `{urn:foo2}attr` attribute. The archival
expected result instead rebinds `foo` to `urn:foo2` on the child and serializes
both names with that prefix. Reparsing that result changes the literal
element's expanded name to `{urn:foo2}p`.

## Disposition

Namespace fixup may choose prefixes, but it may not change the expanded name of
an already constructed literal result element. The case therefore receives an
exact `unusable-reference-result-excluded` disposition rather than pressuring
FastXSLT to reproduce a semantically different tree.

The case remains named in the conserved 3,173-case denominator and receives no
pass credit. Prefix spelling elsewhere in the result is already ignored by the
XML-semantic comparator, so this disposition is specifically about the changed
namespace URI, not cosmetic prefix choice.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from 13 to 12, and unusable-reference-result exclusions rise
from 38 to 39.

## Verification

The measurement adapter's exact bounded-list test names this identity. Existing
namespace-aware result comparison and result-tree tests retain expanded-name
semantics independently of serialized prefix spelling.

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt057
./scripts/verify.ps1
```
