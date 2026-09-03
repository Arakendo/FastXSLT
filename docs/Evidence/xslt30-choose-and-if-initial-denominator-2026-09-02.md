# XSLT30 Choose and If Initial Denominator

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set
  `tests/insn/choose/_choose-test-set.xml` with 55 cases.
- Unchanged cases `choose-0101`, `choose-0102`, `choose-0201`, `choose-0301`,
  `choose-0401`, `choose-0402`,
  `choose-0403`, `choose-0404`, `choose-0501`, `choose-0502`, `choose-0601`,
  `choose-0602`, `choose-0605`, `choose-0701`, `choose-0702`, `choose-0801`,
  `choose-0901`, `choose-1001`, `choose-1101`, `choose-1202`, `choose-1203`,
  `choose-1301`, and `choose-1401`.
- Unchanged negative cases `choose-1801` through `choose-1804`.

## Method

A first-party denominator overlay records all 55 catalog cases before
selection. The executable adapter parses the pinned catalog, rejects duplicate
case identities, checks the complete count, and requires every selected pass to
have an explicit overlay record. For each admitted case it imports the inline
or catalog-referenced principal source and unchanged stylesheet into a bounded
sealed resource snapshot, compiles once, executes through a transform set, and
compares the result to the catalog's native XML assertion.

The existing private `xsl:choose` and `xsl:if` instructions needed only a small
extension to conditional-expression compilation. A bare NCName is now compiled
as a relative child location path and tested by effective boolean value.
Literal strings and numeric constants are reduced to their effective boolean
value, equality between two literal strings is reduced at compile time, and an
exact context-item string comparison (optionally inside `not()`) is retained
for charged evaluation against each selected source node. An exact relative
path-to-string-literal comparison evaluates the location path and controlled
XDM string values with existential XPath general-comparison behavior. The path
may be an attribute step; parsing comparison operands precedes the bare-path
existence form so `@name='value'` is not misclassified as one malformed path.

The compiler validates the complete child structure of `xsl:choose` before it
compiles any branch expression or constructor. This makes structural static
errors deterministic even when an earlier branch contains an expression beyond
the admitted subset. Missing `test`, `xsl:when` after `xsl:otherwise`, and a
second `xsl:otherwise` are compared to the catalog's native `XTSE0010`
expectation.

No general XPath conditional evaluator or alternate execution backend was
introduced.

## Result

- Complete conserved denominator: 55 cases.
- Selected and passed: 27, comprising 23 result comparisons and 4 expected
  static-error comparisons.
- Engine unsupported: 0.
- Excluded by profile: 0.
- Visible default not run: 28.

The unchanged cases cover ordered first-match branch selection, an
`xsl:otherwise` branch, empty fall-through when no branch matches, two true
constant equality tests, and true/false effective boolean values for a
non-empty string and numeric zero. Two further cases compare each selected
node's string value with a literal, once positively and once through `not()`,
while preserving apply-templates order and focus.

`choose-0201` and `choose-0301` exercise attribute string comparisons during
source-node iteration. Present nonmatching and absent attributes both make each
branch false, and a choose without `xsl:otherwise` contributes no result.

Two earlier cases apply the path-to-string comparison inside `xsl:for-each` and
recursive template dispatch. One of them composes with the already accepted
exact strip-all whitespace view and verifies that empty branches do not produce
result nodes.

Two cases reuse the existing exact constant numeric evaluator: one tests
`round(3.7) > 3`, while the other reaches a nested `xsl:if` through
`xsl:otherwise` and tests `9 mod 3 = 0`.

`choose-1001` applies the exact relative-path-to-integer-less-than form to the
untyped integer lexical value `5`. Ordered evaluation stops at the first true
bound and instantiates only that branch; decimal, floating-point, and general
numeric comparison remain outside this slice.

`choose-0801` applies an exact final context-string predicate to a relative
child path, then reads sibling values from each matching parent context. It
also verifies that the explicit `./Name` spelling has the same relative-child
semantics as `Name`; this does not admit general predicate expressions.

`choose-0901` composes two already admitted attribute string comparisons with
the XPath `or` operator. Evaluation is left-to-right and short-circuits after a
true operand while preserving work accounting in every evaluated operand.

`choose-1101` nests seven `xsl:if` instructions and combines exact constant
arithmetic, literal and context string comparisons, and `name(..)` against an
unqualified literal. A focused control proves the name comparison does not
equate a namespaced parent with the same local name.

`choose-1202` evaluates `.//*` relative to the current `doc` element, preserving
descendant document order without escaping to sibling or ancestor subtrees. Its
26-way choose compares unqualified node names and stops at the first match. A
case-specific comparator owns the native exact `/out = string-literal`
assertion without claiming arbitrary XPath assertions.

`choose-1203` reuses the same context-relative descendant sequence and selects
among 26 exact `string-length(.) = integer` branches. String length counts
Unicode codepoints rather than UTF-8 bytes, scans under invocation work
accounting, and retains the suite's exact root-string comparison.

`choose-1301` binds `position()` to an invocation-local variable inside the
matched `doc` template. The binding reads the existing source-template sequence
focus, charges one XPath operation, and supplies the resulting integer to the
already admitted variable comparison. A focused two-item `xsl:for-each`
control proves that successive local bindings observe positions one and two.

Current conserved XSLT30 accounting is 675 cases: 469 passed comparisons, 3
engine-unsupported cases, 55 profile exclusions, and 148 visible default
not-run cases across 17 complete test-set denominators.

## Limitation

This evidence does not admit the other 28 cases. In particular, it makes no
claim for general comparisons, boolean functions beyond the exact `not()`
form and admitted `or` composition, collations, schema-aware cases,
static typing, user functions, import composition, or arbitrary assertion
families. Those cases remain individually visible under the denominator's
default disposition rather than being inferred from the admitted tranche.
