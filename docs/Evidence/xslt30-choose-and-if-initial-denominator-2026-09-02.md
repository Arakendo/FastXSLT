# XSLT30 Choose and If Initial Denominator

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set
  `tests/insn/choose/_choose-test-set.xml` with 55 cases.
- Unchanged cases `choose-0401`, `choose-0402`, `choose-0403`, `choose-0404`,
  `choose-0501`, `choose-0502`, `choose-0601`, `choose-0602`, `choose-0701`,
  and `choose-0702`.

## Method

A first-party denominator overlay records all 55 catalog cases before
selection. The executable adapter parses the pinned catalog, rejects duplicate
case identities, checks the complete count, and requires every selected pass to
have an explicit overlay record. For each admitted case it imports the inline
principal source and unchanged stylesheet into a bounded sealed resource
snapshot, compiles once, executes through a transform set, and compares the
result to the catalog's native XML assertion.

The existing private `xsl:choose` and `xsl:if` instructions needed only a small
extension to conditional-expression compilation. A bare NCName is now compiled
as a relative child location path and tested by effective boolean value.
Literal strings and numeric constants are reduced to their effective boolean
value, equality between two literal strings is reduced at compile time, and an
exact context-item string comparison (optionally inside `not()`) is retained
for charged evaluation against each selected source node.
No general XPath conditional evaluator or alternate execution backend was
introduced.

## Result

- Complete conserved denominator: 55 cases.
- Selected and passed: 10.
- Engine unsupported: 0.
- Excluded by profile: 0.
- Visible default not run: 45.

The unchanged cases cover ordered first-match branch selection, an
`xsl:otherwise` branch, empty fall-through when no branch matches, two true
constant equality tests, and true/false effective boolean values for a
non-empty string and numeric zero. Two further cases compare each selected
node's string value with a literal, once positively and once through `not()`,
while preserving apply-templates order and focus.

Current conserved XSLT30 accounting is 633 cases: 432 passed comparisons, 3
engine-unsupported cases, 54 profile exclusions, and 144 visible default
not-run cases across 16 complete test-set denominators.

## Limitation

This evidence does not admit the other 45 cases. In particular, it makes no
claim for general comparisons, boolean functions beyond the exact `not()`
form, compound boolean expressions, collations, schema-aware cases,
static typing, user functions, import composition, or arbitrary assertion
families. Those cases remain individually visible under the denominator's
default disposition rather than being inferred from the seven passes.
