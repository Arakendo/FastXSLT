# OASIS XSLT 1.0 Literal-Document Descendant Dispatch

- Date: 2026-09-26
- Status: Verified semantic and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Implemented slice

XSLT 1.0 `xsl:apply-templates` and the shared node-selection path now admit
the bounded form:

```xpath
document('literal-relative-reference')//unqualified-name
```

The compiled selection retains the stylesheet base and literal reference.
Execution resolves only through the sealed snapshot, prepares the supplemental
document within the invocation, applies the compiled whitespace-visibility
policy, performs charged iterative descendant traversal in document order, and
dispatches every selected node with correct focus position/size and parameter
bindings. No ambient acquisition, recursion-dependent traversal, or
cross-document node storage is introduced.

## Evidence

The focused regression selects two nested `body` descendants and proves:

- document-order selection;
- `position()` and `last()` over the external selection;
- per-template parameter propagation; and
- external-document source context.

The unchanged Lotus `mdocs_mdocs01` case now compares exactly as `<out>ok</out>`.
Lotus `mdocs_mdocs11` also reaches execution and exposes its missing unadmitted
supplemental resource rather than remaining hidden behind the expression
grammar frontier.

## Conserved result

The sweep moves from 2,348 to **2,350 initialized** cases, from 2,297 to
**2,298 successful executions**, and from 2,140 to **2,141 exact matches
(67.48%)**. Initialization failures fall from 822 to 820; execution failures
rise from 51 to 52 because `mdocs11` now reaches its later missing-resource
boundary. Expected-error credit remains 423 / 431, comparator gaps remain 67,
and visible XML mismatches remain zero.

Dynamic `document()` arguments, the two-argument form, namespace-qualified
descendant tests, `node()`/`text()` descendants, and cross-document node-set
bindings remain outside this slice.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_apply_templates_to_literal_document_descendants_preserves_focus_and_parameters
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs_mdocs01
./scripts/verify.ps1
```
