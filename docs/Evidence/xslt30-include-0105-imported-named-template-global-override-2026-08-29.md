# XSLT30 `include-0105` Imported Named Template and Global Override

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0105`
- Principal stylesheet: `include-0105.xsl`
- Imported stylesheet: `include-0105a.xsl`
- Source: inline `include-01` environment

## Execution

The sealed snapshot contains the source and exactly two stylesheet modules. The
principal module declares `$test` as the text literal `OK`; the imported module
declares the same global as `ERROR` and provides named template `two`.

Compilation builds the principal module without prematurely rejecting its call
to `two`, then assembles the imported declarations and performs whole-program
named-reference validation. Global precedence removes the shadowed imported
binding rather than materializing and later overwriting it. The compiled program
therefore retains exactly one text global named `test`, with value `OK`, plus one
named template `two`.

Runtime named-template frames begin with invocation-local global atomics before
binding their own parameters. Both the principal root template and imported
named template consequently read the same `$test` value.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0105` | `<out><one>OK</one><two>OK</two></out>` | XML-equivalent result | passed |

The conserved 16-case include denominator now has 5 explicit passes and 11
visible default not-run dispositions.

## Claim boundary

This evidence admits one imported named template with no competing same-named
declaration and one same-named text global overridden by the principal module.
It does not admit duplicate named-template precedence, imported root templates,
arbitrary global dependency ordering, non-text shadowed defaults, multiple or
nested imports, or import/include composition.
