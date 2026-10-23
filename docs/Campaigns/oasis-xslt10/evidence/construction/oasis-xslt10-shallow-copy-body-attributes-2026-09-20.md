# OASIS XSLT 1.0 Shallow-Copy Body Attributes

Date: 2026-09-20

## Question

Does source and temporary `xsl:copy` preserve attributes produced by its body,
including the ordinary identity-transform shape where an attribute template
returns an attribute to the copied element?

## Finding

The element-copy paths materialized their statically compiled attributes but
stored every body result directly as a child. An attribute produced through
`xsl:apply-templates`, `xsl:call-template`, or another nested instruction
therefore escaped the copied element and later failed as `XTDE0410`. Literal
and computed element construction already owned the required assembly rule.

This was a shared result-construction defect, not an XSLT 1.0 recovery gap.

## Implementation

A private result-tree helper now assembles an element body by:

- absorbing pending attributes before child content;
- rejecting duplicate expanded attribute names;
- rejecting attributes produced after child content; and
- retaining ordinary child order.

Literal/computed element construction, source `xsl:copy`, and temporary-tree
`xsl:copy` use the same helper. The change adds no compatibility-only recovery,
public representation, resource authority, or alternate evaluator. Existing
`XTDE0410` behavior remains intact for genuinely late and duplicate attributes.

The focused identity-transform regression copies attributes through
`xsl:apply-templates select="@*|node()"` and compares the complete serialized
result.

## Corpus movement

Twelve unchanged OASIS cases move from execution failure to exact
XML-semantic comparison:

- Lotus `copy01`, `copy03`, `copy07`, `copy17`, `copy31`, `copy34`, `copy35`,
  `copy36`, `copy37`, and `copy39`;
- Microsoft `ConflictResolution_77746`; and
- Microsoft `Keys_91730`.

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,089 | 2,089 | 0 |
| Initialization failures | 1,046 | 1,046 | 0 |
| Executed successfully | 1,986 | 1,998 | +12 |
| Execution failures | 103 | 91 | -12 |
| Exact XML-semantic matches | 1,855 | 1,867 | +12 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,867 / 3,173 = 58.84%`. The execution `XTDE0410` frontier contracts from 17
cases to five. One additional case remains an initialization-time `XTDE0410`.
Those cases exercise genuine late or escaping attributes and are not silently
credited by this shared fix.

This is local compatibility evidence against the hash-verified,
non-redistributed OASIS CD04 archive; it is not a broad conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features copy_absorbs
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier XTDE0410
./scripts/verify.ps1
```
