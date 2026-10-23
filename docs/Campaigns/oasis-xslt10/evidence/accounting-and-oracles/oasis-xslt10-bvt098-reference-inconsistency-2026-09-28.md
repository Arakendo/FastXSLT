# OASIS XSLT 1.0 `BVTs_bvt098` Reference Inconsistency

Date: 2026-09-28  
Status: Verified archival disposition; no conformance credit  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Observation

The unchanged simplified stylesheet `BVTs/ws.xsl` constructs one `html` root
whose only child is a computed `foo-elem`. That child contains one computed
attribute, text, a processing instruction, and a comment. It contains no
literal `foo`, no instruction selecting source nodes, and no template rule
capable of constructing another result sibling.

FastXSLT produces the complete constructible result:

```xml
<html><foo-elem foo-attr="Hello">Hello<?style Hello><!--Hello--></foo-elem></html>
```

The archived reference appends `<foo></foo>` after `foo-elem`. That node has no
provenance in the supplied stylesheet or source-driven execution.

## Disposition

`Microsoft/BVTs_bvt098#1` is classified as an unusable archival reference
result. It receives no pass credit and is not used to justify adding a phantom
result node.

The strict exact lower bound remains **2,254 / 3,173 (71.04%)**. Visible
mismatches fall from 17 to 16 and unusable reference-result exclusions rise
from 60 to 61. Initialization, execution, comparator-gap, and expected-error
totals remain unchanged.

## Boundaries

- Upstream bytes remain immutable.
- This is an exact one-case disposition, not a general tolerance for extra or
  missing result nodes.
- No engine, serializer, or comparator behavior changes.
- The case can be reconsidered if an authoritative corrected reference or
  missing stylesheet content is recovered.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
```
