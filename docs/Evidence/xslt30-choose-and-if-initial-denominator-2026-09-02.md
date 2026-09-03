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
  `choose-0602`, `choose-0603`, `choose-0604`, `choose-0605`, `choose-0606`, `choose-0609`,
  `choose-0701`,
  `choose-0702`, `choose-0801`, `choose-0901`, `choose-1001`, `choose-1101`,
  `choose-1201`, `choose-1202`,
  `choose-1203`, `choose-1204`, `choose-1301`, `choose-1401`, `choose-1501`,
  `choose-1502`, `choose-1601`, `choose-1701`, `choose-1702`, `choose-1703`,
  `choose-1704`, `choose-1706`, and `choose-1901` through `choose-1905`.
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
- Selected and passed: 46, comprising 42 result comparisons and 4 expected
  static-error comparisons.
- Engine unsupported: 0.
- Excluded by profile: 0.
- Visible default not run: 9.

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

`choose-0603` constructs two distinct temporary document nodes from untyped
global sequence constructors. Direct variable comparison and explicit
`string()` comparison both derive their atomized string values under XDM work
accounting rather than equating node identity or approximating the globals as
compiled atomic strings. A focused compiler control preserves the temporary
document form, and unrelated temporary globals are not materialized while
resolving a narrow value expression.

`choose-0604` applies `xml:space="preserve"` to `xsl:choose`. Whitespace-only
stylesheet text within the selected `xsl:when` and nested `xsl:if` sequence
constructors is retained through inherited stylesheet-space state instead of
being discarded as indentation. A focused negative control reports `XTSE0020`
for an invalid `xml:space` lexical on the admitted instruction.

`choose-1501` and `choose-1502` retain an exact typed `xs:boolean` global and
apply `xpath-default-namespace` at `xsl:if`, `xsl:choose`, `xsl:when`,
`xsl:otherwise`, and `xsl:value-of` boundaries. The exact
`count(location-path)` value form binds unprefixed element tests to the
effective namespace while leaving an explicit empty value as a reset. The
unchanged source contains identically named elements in two namespaces and no
namespace, so the result detects incorrect inheritance rather than merely
exercising syntax. Selection and counting use the ordinary controlled path
evaluator.

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

`choose-1201` walks context-relative `title` descendants and nests choices that
look ahead at successively higher tree levels. Its `following-sibling::*[1]`
steps filter the forward sibling axis to elements before applying the position,
then feed the selected expanded name into the existing exact unqualified
`name(path)` comparison. A focused path control separately covers named and
wildcard following-sibling selection.

`choose-1202` evaluates `.//*` relative to the current `doc` element, preserving
descendant document order without escaping to sibling or ancestor subtrees. Its
26-way choose compares unqualified node names and stops at the first match. A
case-specific comparator owns the native exact `/out = string-literal`
assertion without claiming arbitrary XPath assertions.

`choose-1203` reuses the same context-relative descendant sequence and selects
among 26 exact `string-length(.) = integer` branches. String length counts
Unicode codepoints rather than UTF-8 bytes, scans under invocation work
accounting, and retains the suite's exact root-string comparison.

`choose-1204` reuses that 26-way descendant/name shape under a
`default-collation` preference list. The compiler selects the first available
member, the W3C HTML ASCII case-insensitive collation, and records the
comparison strategy in compiled semantics. Lowercase source names therefore
match uppercase literals without changing QName identity. A focused negative
control reports `XTSE0125` when no listed collation is available; this does not
approximate the later UCA fallback.

`choose-1301` binds `position()` to an invocation-local variable inside the
matched `doc` template. The binding reads the existing source-template sequence
focus, charges one XPath operation, and supplies the resulting integer to the
already admitted variable comparison. A focused two-item `xsl:for-each`
control proves that successive local bindings observe positions one and two.

`choose-0606`, `choose-1703`, and `choose-1704` retain schema-namespace-resolved
`xs:string` and `xs:untypedAtomic` global values as typed atomic state. Bare
variable conditionals apply string-family effective boolean value, so non-empty
values are true and empty values are false. The prefix spelling is not trusted:
a focused compiler control accepts an alternate prefix bound to the XML Schema
namespace and rejects `xs` rebound elsewhere.

`choose-0609` retains an untyped global `xsl:sequence select="()"` as an actual
empty-sequence binding and keeps it distinct from an empty
`xs:untypedAtomic`. Exact `()`, `$variable=()`, and `boolean($variable)` tests
all produce false without constructing result content. Its unexecuted branch
also proves that literal-string `xsl:value-of` compiles without being mistaken
for a location path.

`choose-1601` and `choose-1706` retain literal `xs:integer` and `xs:double`
constructor values and the exact source-dependent
`xs:double(path div path)` global form. Controlled path selection and string
conversion precede one charged division. A missing operand produces the empty
sequence, `0 div 0` produces typed `NaN`, numeric zero has false effective
boolean value, and the tiny nonzero double remains true. A focused compiler
control also proves that constructor prefixes are resolved through the XML
Schema namespace rather than trusted by spelling.

`choose-1701` and `choose-1702` retain a narrow XPath conditional-expression
plan shared by conditional tests and `xsl:value-of`. Exact `contains(path,
string-literal)` and constant integer less-than conditions select integer
branches lazily, including one nested conditional. Path traversal, string-value
derivation, and the string containment operation remain work-accounted. A
focused compiler control rejects the unadmitted `lt` spelling instead of
silently widening the grammar.

`choose-1901` through `choose-1905` add a separate typed-path conditional plan.
Each condition resolves the lexical schema-constructor prefix, converts two
singleton path values to integers, and compares them for equality or ordering.
Branches return a path string value, retain a deliberately failing division,
or recursively contain another typed-path conditional. Only the selected branch
executes: two unchanged cases would report division by zero under eager branch
evaluation. A focused compiler control accepts an alternate prefix bound to the
XML Schema namespace and rejects the conventional `xs` prefix when rebound.

Current conserved XSLT30 accounting is 675 cases: 488 passed comparisons, 3
engine-unsupported cases, 55 profile exclusions, and 129 visible default
not-run cases across 17 complete test-set denominators.

## Limitation

This evidence does not admit the other 9 cases. In particular, it makes no
claim for general comparisons beyond the exact admitted variable and path
forms, boolean functions beyond the exact `not()` form and admitted `or`
composition, collations beyond the two retained strategies, schema-aware cases,
static typing, user functions, import composition, or arbitrary assertion
families. Those cases remain individually visible under the denominator's
default disposition rather than being inferred from the admitted tranche.
