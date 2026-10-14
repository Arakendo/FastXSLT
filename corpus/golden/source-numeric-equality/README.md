# Source numeric equality

First-party MIT-licensed fixture authored for FastXSLT on 2026-10-13.
No upstream content is copied; this is regression evidence, not conformance credit.

Exercises ordinary production global node-set equality against integer literals:
a matching later node, an integer beyond i32 range, relational literal comparisons
(including reversed operands), pairwise variable relations, a numeric scalar
variable operand, and an empty selection.
Explicit `string()` equality separately proves first-node rather than existential
conversion, plus empty-node-set conversion to the empty string.
Bare scalar equality and inequality compare a numeric global with the differently
spelled string `02`, exercising type-dependent conversion rather than lexical equality.
Source-to-source equality and inequality are independently existential: a varying
nonempty set is both equal and unequal to itself, while inequality with an empty
set is false.
Mixed node-set/numeric-variable equality and reversed inequality exercise later
matching nodes and independent unequal pairs through ordinary production globals.
AR-0030's owned-value tests execute this fixture without demand hooks.
Mixed explicit-string/bare-node equality retains existential comparison on the
bare side: `string($threshold) = $nodes` finds the matching later node.
