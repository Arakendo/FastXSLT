# XSLT30 `include-0501` Sibling-Import Global Parameter Precedence

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0501`
- Principal stylesheet: `include-0501.xsl`
- Imported stylesheets, in declaration order: `include-0501a.xsl`,
  `include-0501b.xsl`
- Source: inline `<doc/>`

## Execution

The sealed snapshot contains the source, principal stylesheet, and two sibling
imports. Dependency preparation resolves both relative references before
semantic compilation, charges three module occurrences under the private
depth-one profile, and retains declaration order without filesystem or network
fallback.

The first import declares `$first` and `$second`; the later import declares a
second `$second`. FastXSLT assigns the first and second imported modules
precedence `-2` and `-1`, respectively, below principal precedence `0`.
Whole-program assembly processes winning declarations from high to low
precedence. It therefore discards the first module's shadowed `$second` default
before invocation materialization while retaining its unshadowed `$first`.

The assembled global defaults are observed in stable evaluation order as:

- `$first = 'aaa, as defined in first.xsl'`
- `$second = 'ZZZ, as defined in second.xsl'`

## Result

| Case | Expected | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0501` | Two `p` elements containing the winning parameter defaults | XML-equivalent result | passed |

The conserved 16-case include denominator now has 7 explicit passes and 9
visible default not-run dispositions.

## Claim boundary

This evidence admits exactly two sibling imports with no nested dependencies
and supported global parameter defaults whose names compete across import
precedence. It does not admit mixed include/import assembly, three or more
direct dependencies, nested imports, duplicate named-template precedence,
imported output declarations, or a public module-graph contract.
