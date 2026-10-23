# QT3 `fn:empty` Initial Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Complete native test set `fn/empty.xml` with 54 cases.
- First-party overlay
  `corpus/overlays/qt3/empty-denominator-v0.toml`.

## Method

The typed QT3 denominator loader parses the immutable upstream set, verifies
its exact case identities and count, composes explicit private-ledger passes
with native dependency exclusions, and applies a visible
`harness-unsupported/not-run` default to every remaining case.

A private, safe, source-free production expression executes `fn:empty` and unprefixed
`empty` over bounded atomic literal sequences. It reuses the atomic sequence
parser already exercised by `fn:deep-equal`, including validated integer
subtypes, exact decimals, floats, doubles, empty sequences, nested sequence
flattening, and string literals. The evaluator additionally handles `fn:not`,
charges each executed XPath operation, and distinguishes zero- or multi-argument
calls from unsupported grammar so native `XPST0017` assertions remain exact.

The adapter reads the unchanged expression and native assertion from
`fn/empty.xml`, compiles it inside `xsl:value-of`, executes the ordinary runtime,
and compares the serialized boolean or compile diagnostic. It does not encode
expected booleans from case names and does not infer results for the five
unexecuted XPath cases. A cardinality-composition sentinel reaches the same
expression through the host-facing workbench engine.

## Result

| Test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/empty.xml` | 54 | 47 | 2 | 5 |

The 47 passes comprise 39 typed numeric singleton cases, two native arity
errors, and six direct or negated literal-sequence cardinality cases. The two
`XQ10+` cases are excluded from the XPath-in-XSLT profile. The five visible
defaults require current-time/remove/exists composition or `for`, range,
predicate, `boolean`, and `floor` semantics beyond this deliberately narrow
adapter.

The audited QT3 subtotal is now 716 cases: 503 selected passes, 181 profile
exclusions, and 32 visible default not-run cases. The remaining 31,105 QT3
cases stay at catalog inventory only.

## Boundary

This tranche establishes sequence cardinality for the admitted source-free
atomic grammar. It does not establish general `fn:empty`, node-sequence
evaluation, dynamic sequence construction, range predicates, higher-order
functions, or arbitrary constructors.
