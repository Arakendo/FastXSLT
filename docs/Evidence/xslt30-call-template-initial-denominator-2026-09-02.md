# XSLT30 Call-Template Initial Denominator

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set
  `tests/insn/call-template/_call-template-test-set.xml` with 42 cases.
- Unchanged cases `call-template-0101`, `call-template-0801`, and
  `call-template-0802`.

## Method

A first-party denominator overlay records all 42 catalog cases before
selection. The executable adapter parses the pinned catalog, verifies unique
case identities and the complete count, imports each selected principal source
and stylesheet into a bounded sealed resource snapshot, compiles once, and
executes through the ordinary transform-set path. Native XML assertions are
compared structurally.

Named-template declarations and calls now share one static QName
normalization path. An unprefixed name remains in no namespace. A prefixed name
is resolved from the instruction's stylesheet namespace context and retained as
an expanded `Q{namespace}local` identity. Names in the reserved XSLT namespace
remain limited to the standard initial-template name.

## Result

- Complete conserved denominator: 42 cases.
- Selected and passed: 3.
- Engine unsupported: 0.
- Excluded by profile: 0.
- Visible default not run: 39.

`call-template-0101` enters `temp` directly as the initial template and proves
the document-matching template is not selected instead. `call-template-0801`
resolves `foo:a` to the qualified declaration. `call-template-0802` calls the
unqualified `a` declaration while the qualified declaration with the same local
part remains present and distinct.

Adding this denominator changes conserved XSLT30 accounting to 675 cases: 443
passed comparisons, 3 engine-unsupported cases, 54 profile exclusions, and 175
visible default not-run cases across 17 complete test-set denominators.

## Limitation

This evidence does not admit the other 39 cases. In particular, it does not
establish catalog/host resolution of a qualified initial-template name, EQName
syntax, required or typed parameters, deep/tail recursion behavior, arbitrary
focus access, or the other assertion families used by the set. The unchanged
`call-template-0102` case also carries `exclude-result-prefixes` on the named
template; that standard attribute and its namespace-copy consequences remain a
visible boundary rather than being ignored to obtain a pass.
