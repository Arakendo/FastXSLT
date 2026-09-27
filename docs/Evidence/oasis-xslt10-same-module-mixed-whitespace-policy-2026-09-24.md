# OASIS XSLT 1.0 Same-Module Mixed Whitespace Policy

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0016, AR-0019
- Related decision: ADR-0012

## Pressure

Nine unchanged cases remained at `FXST1043` after exact-name stripping and
preserve-only declarations were admitted. Eight can use a mixed policy whose
winning rules are established within one compiled stylesheet module; some
also assemble dependencies whose declarations are absent or completely
shadowed by the principal module. The required rule is not
declaration order alone: an exact expanded-name test has greater priority than
the `*` wildcard, while the later declaration wins between equal-priority
tests.

## Implemented slice

Compilation now retains a private mixed policy containing:

- the winning same-precedence wildcard action;
- disjoint exact expanded names whose winning action is strip; and
- disjoint exact expanded names whose winning action is preserve.

Later exact declarations replace an earlier exact action for the same expanded
name. Exact actions override the wildcard regardless of their relative source
order. The invocation-owned visibility view and complete safe reference apply
the same predicate, retain `xml:space` inheritance, preserve visible `NodeId`
and provenance, and continue charging one XDM visit per inspected node.

At this tranche, the slice deliberately excluded namespace wildcards and
general mixed-policy composition across include/import precedence. A mixed
policy may pass through a dependency that contributes no competing rule, but
lower-precedence composition remains a private bounded subset rather than a
general claim. Namespace wildcards were admitted by the subsequent
[namespace-wildcard evidence](oasis-xslt10-namespace-wildcard-whitespace-2026-09-24.md).

## Verification

Focused compiler evidence retains the wildcard and disjoint exact actions.
Focused runtime evidence places exact preservation before the later strip-all
declaration, proving that NameTest priority—not source order—keeps the selected
element's whitespace. The visibility view and complete reference produce the
same result while the prepared source remains unchanged.

Eight unchanged cases leave `FXST1043`:

- `Lotus/namespace_namespace23#1`,
- `Lotus/whitespace_whitespace04#1`,
- `Lotus/whitespace_whitespace22#1`,
- `Microsoft/ConflictResolution_ConflictResBetweenStripSpaceAndPreserveSpace#1`,
- `Microsoft/Whitespaces__91453#1`,
- `Microsoft/Whitespaces__91454#1`,
- `Microsoft/Whitespaces__91455#1`, and
- `Microsoft/Whitespaces__91456#1`.

Five compare exactly. `namespace23` reaches a later namespace-alias mismatch;
`91455` and `91456` expose the already documented CDATA/indentation provenance
boundary. Those three remain visible mismatches and are not credited as passes.

The complete sweep moves from 2,293 to 2,301 initialized cases and from 2,241
to 2,249 successful executions. Exact expected-result matches rise from 2,083
to **2,088 / 3,173 (65.81%)**. XML mismatches rise from 75 to 78 because the
three newly executed later-boundary cases are now honestly visible.

## Reproduction

```powershell
cargo test -p fastxslt --all-features compiles_exact_preserve_all_as_the_default_whitespace_policy
cargo test -p fastxslt --all-features xslt10_mixed_whitespace_rules_preserve_exact_names_over_strip_all
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
