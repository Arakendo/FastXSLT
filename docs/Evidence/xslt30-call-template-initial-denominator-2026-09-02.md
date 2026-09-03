# XSLT30 Call-Template Initial Denominator

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set
  `tests/insn/call-template/_call-template-test-set.xml` with 42 cases.
- Unchanged case `call-template-0001`, `call-template-0101` through
  `call-template-0107`, plus
  `call-template-0109`, `call-template-0201`, `call-template-0401a`, `call-template-0801`,
  `call-template-0802`, `call-template-1101`, `call-template-1501`,
  `call-template-1701`, and `call-template-1801` through `call-template-1803`.

## Method

A first-party denominator overlay records all 42 catalog cases before
selection. The executable adapter parses the pinned catalog, verifies unique
case identities and the complete count, imports each selected principal source
and every catalog-declared principal or secondary stylesheet into a bounded
sealed resource snapshot, compiles once, and executes through the ordinary
transform-set path. Native XML assertions are compared structurally.

Catalog-supplied initial-template QNames are resolved against namespace
declarations on the catalog instruction before request admission. That keeps
lexical catalog prefixes out of engine identity and prevents the negative
qualified-name cases from passing merely because an unresolved prefix-shaped
string failed lookup.

Named-template declarations and calls now share one static QName
normalization path. An unprefixed name remains in no namespace. A prefixed name
is resolved from the instruction's stylesheet namespace context and retained as
an expanded `Q{namespace}local` identity. Names in the reserved XSLT namespace
remain limited to the standard initial-template name.

## Result

- Complete conserved denominator: 42 cases.
- Selected and passed: 19.
- Engine unsupported: 0.
- Excluded by profile: 1.
- Visible default not run: 22.

`call-template-0101` enters `temp` directly as the initial template and proves
the document-matching template is not selected instead. `call-template-0801`
resolves `foo:a` to the qualified declaration. `call-template-0802` calls the
unqualified `a` declaration while the qualified declaration with the same local
part remains present and distinct.

The negative cases conserve error behavior as evidence too:
`call-template-0001` enters without a source and reports `XPDY0002` when its
exact `ancestor-or-self::*` copy selection attempts to reference the absent
focus.
`call-template-0104` reports `XTDE0040` when an unqualified requested initial
template is absent. `call-template-0105` and `0107` resolve qualified catalog
names to expanded identities before reporting `XTDE0040`; the latter proves a
different lexical prefix does not disguise the namespace mismatch.
`call-template-0106` reports `XTSE0080` when a named template uses the reserved
XSLT namespace. `call-template-0401a` reports `XTDE0700` when its selected
initial template declares a required parameter: supplied stylesheet parameters
populate the global frame and cannot masquerade as initial-template arguments.
The XSLT 2.0-only companion `call-template-0401`, which expects edition-specific
code `XTDE0060`, is visibly excluded in favor of this XSLT 3.0 alternative.

`call-template-0102` resolves a qualified catalog entry to the matching
stylesheet declaration. Its template-local `exclude-result-prefixes` composes
with the existing ancestor-aware literal-result namespace filter, so the static
template prefix does not leak into the result. `call-template-0103` binds the
catalog's stylesheet parameter into the global frame while independently using
the selected initial template's literal string default. Its two bounded
literal-plus-atomic `concat()` expressions then produce the asserted text.

`call-template-0201` supplies a source while entering a named template and
copies the current document with `xsl:copy-of select="."`. The document node
acts as a sequence boundary: its children are copied into the containing
literal result element without manufacturing a nested result document.

`call-template-0109` normalizes whitespace around EQNames and gives `Q{}temp`
the same identity as unqualified `temp`. `call-template-1701` proves two
different prefixes bound to the same namespace select one expanded named
template identity. `call-template-1101` performs six nested named calls with
integer `select` arguments; the same path has a focused control proving that a
literal integer default applies when an argument is omitted.

`call-template-1501` combines descendant-existence choices with repeated named
calls. Supplied literal-content arguments and an omitted default are rebound on
each call without leaking a prior call's value.

`call-template-1801` through `1803` reuse the existing stylesheet dependency
loader. Named-template lookup selects a principal declaration over an imported
one and selects the later sibling import where two imports declare the same
name. All secondary modules come from explicit catalog metadata and are sealed
before compilation; execution performs no acquisition.

Adding this denominator changes conserved XSLT30 accounting to 675 cases: 459
passed comparisons, 3 engine-unsupported cases, 55 profile exclusions, and 158
visible default not-run cases across 17 complete test-set denominators.

## Limitation

This evidence does not admit the other 22 cases. Catalog QName resolution is
test-adapter evidence rather than a selected public host API. The slice does
not establish host-supplied initial-template parameters, typed parameters,
node-sequence or general XPath arguments, deep/tail recursion
behavior, arbitrary focus access, or the other assertion families used by the
set.
