# OASIS XSLT 1.0 Computed-Attribute Local Count -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 computed attribute use a lexically local source-node variable
for `count($name)` without introducing a general constructor-local frame or
weakening source-variable shadowing?

## Implemented slice

Yes. The private computed-attribute compiler now admits one bounded constructor
shape: an `xsl:variable` with a controlled source location path followed by an
`xsl:value-of` whose expression is exactly `count($same-name)`. The binding
name must be an unprefixed NCName and the variable must have no content.

The compiled plan retains the controlled path rather than exposing or retaining
a general runtime frame. Execution evaluates that path against the current
source focus under the existing XPath-node-visit, budget, and cancellation
controls and serializes its cardinality as the attribute value. The local name
cannot leak beyond the constructor, so surrounding variables continue to obey
their existing lexical scopes. Retention accounting includes the path.

General local variables, content-valued bindings, arbitrary expressions over
the binding, and multi-instruction computed-attribute constructors remain
unsupported.

## Corpus result

Unchanged Microsoft `Variables__78409` moves from `FXST1033` initialization
failure to an exact XML-semantic expected-result pass. Its computed
`book_count` is `7`, while the stylesheet's same-named variables in outer and
sibling scopes retain their independent values.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,778 | 1,779 | +1 |
| Initialization failures | 1,357 | 1,356 | -1 |
| Executed successfully | 1,594 | 1,595 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,450 | 1,451 | +1 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,451 / 2,742 = 52.92%` for standard-operation cases and
`1,451 / 3,173 = 45.73%` for the complete catalog.

## Verification

A focused production-lifecycle test counts three selected source elements from
the local constructor binding. The unchanged full sweep conserves all 3,173
identities and adds no failure, mismatch, or panic.
