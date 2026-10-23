# OASIS XSLT 1.0 Variable Key Name

## Question

Can a key lookup resolve its lexical key name from an XSLT 1.0 variable while
preserving the stylesheet's static namespace context and existing lookup
semantics?

## Method

- Replace the lookup's static name field with a typed name plan: either an
  already expanded static name or one unqualified variable reference plus the
  immutable in-scope stylesheet namespaces.
- Convert the variable through the existing XSLT 1.0 string-value owner.
- Charge runtime lexical-QName resolution, validate the lexical form, and
  resolve any prefix only against the captured static namespaces.
- Report an invalid or unbound dynamic key name as `XTDE1260`.
- Include the variable name, namespace-slice allocation, namespace strings,
  and lookup location in exact prepared-program capacity accounting.
- Test a prefixed variable value against a namespaced key declaration, then run
  the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case resolves `k:lookup` from a temporary-text global variable
using the namespace binding captured at the call site and selects the expected
source node.

The unchanged case `Lotus/idkey_idkey25#1` now executes and compares exactly.
The strict exact-result lower bound rises from 1,613 to 1,614. Initialized cases
rise from 2,003 to 2,004 and successfully executed cases rise from 1,898 to
1,899. Initialization failures fall from 1,132 to 1,131. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

The admitted dynamic name is one unqualified variable reference whose result
is interpreted as a lexical QName in the call site's immutable static namespace
context. It does not inspect source namespaces, introduce new resource
authority, admit arbitrary name expressions, or add a retained key index.

The result is bounded XSLT 1.0 compatibility evidence, not a general dynamic
QName, `key()`, or conformance claim.
