# XSLT30 `conflict-resolution-0101` Template Priority and `xsl:text`

Date: 2026-08-28

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0101`
- Stylesheet: `conflict-resolution-0101.xsl`
- Environment: embedded `conflict-resolution-01` principal source
- Native assertion: `assert-xml` `<out>Match-of-qualified-name</out>`

## Method

The metadata-driven test resolves the case, embedded source, stylesheet, and
expected XML from the pinned test set. Source and stylesheet bytes are admitted
to one bounded sealed snapshot, the compiled program is reused for a batch of
one, and execution starts from the admitted principal source without ambient
filesystem access.

The stylesheet declares four competing matched templates: exact `doc`, exact
`foo`, element wildcard `*`, and any-node `node()`. Built-in document handling
reaches the exact `doc` rule. Its selected `foo` child then chooses the exact
name rule over both wildcard fallbacks according to the existing default
priority ordering.

Initial execution exposed one independent compiler gap: `xsl:text`. The new
private compiler path accepts attribute-free character content, retains its
owned value and source location, and reuses the existing text result
instruction. It does not treat stylesheet indentation as literal output. A
focused control preserves leading and trailing spaces inside `xsl:text` and
rejects nested element content with structured invalid code `FXST0026`.

## Result

| Case | Expected | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0101` | `<out>Match-of-qualified-name</out>` | semantically equal XML | passed |

## Claim boundary

This evidence admits only this exact qualified-name-versus-wildcard conflict
and the bounded `xsl:text` form needed to execute it. It does not establish the
complete 52-case apply-templates denominator, explicit priorities, import
precedence, ambiguity recovery/failure policies, namespace wildcard patterns,
mode conflict resolution, `disable-output-escaping`, general sequence
construction, or broad template-selection conformance. Execution work limits
for this case remain outside the claim.
