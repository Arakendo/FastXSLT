# OASIS XSLT 1.0 `id()` Without Typed IDs

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged Xalan `idkey09` calls `id('c')/@id` against a source that has an
ordinary `id` attribute but no DTD declaration assigning it the XML ID type.
XPath 1.0 therefore requires an empty node-set. FastXSLT previously rejected
the function-rooted path before it could demonstrate that behavior.

## Implemented slices

The XSLT 1.0 value-expression compiler now recognizes a literal-string
`id()` call, optionally followed by a syntactically valid relative path. It
retains a typed private operation rather than folding the expression to an
unexplained empty string.

The same typed boundary now accepts a location-path argument in both
`xsl:value-of` and node-selection contexts such as `xsl:for-each` and
`xsl:apply-templates`. The argument path is evaluated through the ordinary
controlled path evaluator before the ID lookup returns empty, preserving
context errors, cancellation, and XPath work accounting. Both `/` and `//`
tails after `id()` are syntax-checked even though the current empty selection
cannot feed them.

FastXSLT's current XML parser policy admits no DTD/type information, so the
operation returns an empty selection and charges one XPath work unit. An
ordinary attribute named `id` is deliberately not treated as typed merely
because of its lexical name. The modern expression profile, match-pattern
support, DTD authority, and any future `xml:id` policy remain unchanged.

## Verification and corpus disposition

A focused runtime regression compiles and executes `id('c')/@id` against a
DTD-free source containing `id="c"` and proves that the result is empty.
Another regression evaluates `id(doc/ids/@values)` through both value and
node-selection consumers, and validates a descendant tail, without inventing
ID typing for the selected lexical values.

Unchanged `Lotus/idkey_idkey09#1` now initializes, executes, and matches its
expected result. The complete sweep reaches **2,329 initialized**, **2,282
successfully executed**, and **2,117 / 3,173 exact matches (66.72%)**.
Initialization failures fall to 806; the 69 visible XML mismatches, eight
host-parser-policy exclusions, and three unusable-reference-result exclusions
remain unchanged.

After supplemental-resource admission, the path-argument slice moves seven
unchanged cases out of the generic function-rooted-path frontier. Six now
reach the source parser's explicit `DtdForbidden` boundary, including
`copy16` and `idkey57`; those tests rely on DTD-declared ID types and therefore
receive no pass credit under the current parser policy. Aggregate measurement
remains **2,329 initialized**, **2,282 executed**, and **2,117 / 3,173 exact
matches (66.72%)**. This is useful frontier attribution, not evidence that
DTD-backed `id()` semantics are implemented.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_id_lookup_is_empty_when_the_parser_admits_no_typed_ids
cargo test -p fastxslt --all-features xslt10_id_path_argument_is_evaluated_before_the_empty_typed_id_lookup
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey09#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey57#1'
```
