# OASIS XSLT 1.0 Mixed-Root Static Temporary Tree

Date: 2026-09-20

## Question

Can the existing static global temporary-tree representation preserve literal
text between top-level literal elements without admitting a dynamic sequence
constructor?

## Finding

Yes. The compiled global plan now retains a bounded list of static constructed
nodes rather than requiring every temporary-document root to be an element.
The same typed `ConstructedNode` representation already used for element
children now represents the roots, so literal elements and non-whitespace text
retain their lexical order.

Materialization remains invocation-owned and charges each retained node to the
XDM-node budget before retention. Namespace-alias rewriting and compiled-state
capacity accounting traverse the generalized node roots. An `as="element()"`
global still requires exactly one element root. Dynamic XSLT instructions,
comments and processing instructions in stylesheet syntax, and general mixed
sequence constructors remain outside this static slice.

## Corpus movement

The unchanged Lotus `variable46` case moves from the mixed-content global
constructor frontier to exact XML-semantic comparison.

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,092 | 2,093 | +1 |
| Initialization failures | 1,043 | 1,042 | -1 |
| Executed successfully | 2,006 | 2,007 | +1 |
| Execution failures | 86 | 86 | 0 |
| Exact XML-semantic matches | 1,875 | 1,876 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,876 / 3,173 = 59.12%`. The generic `FXST1015` frontier falls from 19 to 18
cases. Every remaining member contains executable XSLT instructions rather
than only static mixed roots. This is local compatibility evidence against the
hash-verified, non-redistributed OASIS CD04 archive; it is not a broad
conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_static_tree_preserves_mixed_top_level_nodes
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1015
./scripts/verify.ps1
```
