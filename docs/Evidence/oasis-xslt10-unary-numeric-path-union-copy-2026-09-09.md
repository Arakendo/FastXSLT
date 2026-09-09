# OASIS XSLT 1.0 unary numeric path-union copy -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:copy-of` construct the atomic result of unary numeric conversion over
a node-set union without adding an alternate expression evaluator or relaxing
modern sequence cardinality?

## Decision in the experiment

The exact-rational binary numeric representation now has a typed path-union
operand. It delegates union evaluation to the ordinary controlled location-path
union evaluator, which restores document order and removes duplicate node
identity before numeric conversion. XSLT 1.0 compatibility selects the first
node in document order, then applies checked unary negation.

`xsl:copy-of` retains this result as an atomic-value copy operation and uses the
same bounded text construction as other atomic results. Semantic inspection
continues to report `CopyOf`, and retained-capacity accounting includes the
compiled numeric expression and all union paths.

An initial experiment admitted every unary numeric root. That also routed large
numeric literals through exact-rational formatting and produced one new
mismatch plus one overflow because XSLT 1.0 expects historical IEEE-double
lexical behavior there. The admitted root was therefore narrowed to the proven
unary path-union shape before this result was recorded.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,576 | 1,577 | +1 |
| Executed successfully | 1,405 | 1,406 | +1 |
| Expected-result XML matches | 1,285 | 1,286 | +1 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/math_math103#1` case now matches exactly. The strict
standard-operation lower bound is now `1,286 / 2,742 = 46.90%`; the
conservative all-catalog ratio is `1,286 / 3,173 = 40.53%`. These are local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not select general unary numeric expressions, IEEE-double
compatibility formatting, general atomic `xsl:copy-of`, or a second expression
backend. Modern mode retains zero-or-one numeric operand cardinality. No corpus
bytes or expected results changed.

## Verification

- A focused runtime test deliberately writes the union in reverse lexical
  order and proves conversion uses the first node in document order.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
