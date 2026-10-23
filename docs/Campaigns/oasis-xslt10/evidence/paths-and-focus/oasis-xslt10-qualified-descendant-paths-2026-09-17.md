# OASIS XSLT 1.0 Qualified Descendant Paths -- 2026-09-17

Date: 2026-09-17  
Status: Verified shared XPath and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The typed location-path runtime already distinguished document-rooted and
context-rooted descendant origins and already matched expanded element and
attribute names. The qualified-path parser nevertheless rejected leading `//`
and `.//`, and it had no typed representation for namespace wildcards such as
`my:*` or `@dt:*`. Six unchanged cases therefore stopped before reaching the
existing evaluator.

## Repair

Qualified paths now retain document-descendant or context-descendant origin at
compile time. Prefixed element and attribute name tests still resolve to
expanded names through the stylesheet namespace context. A prefixed `*` local
part compiles to a typed element-namespace or attribute-namespace test rather
than being approximated as a local-name wildcard.

The runtime uses the ordinary charged location-path traversal, document-order
normalization, cancellation, and source identity. Unbound prefixes remain
static `XPST0081`; interior descendant abbreviations, non-ASCII name parsing,
and other unimplemented path shapes remain explicit.

## Corpus result

Six cases leave the generic `FXXP1001` frontier:

- Microsoft `ConflictResolution__77847` and `ConflictResolution__77870`
  become exact XML comparisons.
- Microsoft `ConflictResolution__77871` and `ConflictResolution__84476`
  execute to visible whitespace/serialization mismatches.
- Lotus `copy_copy51` advances to its independent `local-name(.)` expression
  boundary.
- Microsoft `BVTs_bvt061` advances to its independent dynamic `xsl:number`
  grouping boundary.

The exact-result lower bound rises from 1,523 to 1,525. Initialized cases rise
from 1,872 to 1,876, executed-successfully cases rise from 1,673 to 1,677, and
visible mismatches rise from 121 to 123. No comparator-unsupported outcome,
expected-error disposition, or panic is added.

## Verification

Focused path tests distinguish document and context descendant origins and
prove namespace-wildcard filtering for both elements and attributes. The
complete local measurement conserves all 3,173 identities: 1,876 initialize,
1,677 execute successfully, 1,525 compare exactly, 123 remain visible
mismatches, and 23 reach comparator-unsupported outcomes. Expected-error
accounting remains 403 initialization observations, 21 execution observations,
and four doubt-annotated unexpected successes.
