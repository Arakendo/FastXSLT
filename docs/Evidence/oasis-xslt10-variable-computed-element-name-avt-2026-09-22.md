# OASIS XSLT 1.0 Variable Computed-Element Name AVT

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can the existing XSLT 1.0 computed-element path safely admit names that compose
literal text with a variable, without broadening modern semantics or accepting
general XPath inside the AVT?

## Method

- Reuse the bounded computed-attribute name-AVT representation for literal,
  variable, and focus-position parts.
- Convert invocation-local atomic and temporary-tree variables through the
  existing charged XSLT 1.0 string-value path.
- Retain the existing runtime lexical-QName, namespace, and reserved-name
  validation for the final value.
- Add a focused end-to-end `node-{$kind}` regression and rerun the unchanged
  3,173-case OASIS catalog.

## Result

The unchanged Lotus `axes_axes15` case now constructs its `From-*` element
names and compares exactly. Microsoft `BVTs_bvt073` leaves `FXST1047` and
reaches a separate unsupported qualified-variable path in its body. The
broader `{$foo/ElementType/@name}` shape remains explicitly unsupported.

The complete sweep initializes 2,154 cases, reports 981 initialization
failures, executes 2,098 successfully, and reports 56 execution failures.
Exact XML-semantic matches reach 1,954, with 62 mismatches. The strict
compatibility lower bound is `1,954 / 3,173 = 61.58%`.

## Boundaries

- The new AVT form is compile-selected only for XSLT 1.0 compatibility.
- It admits literal text, unprefixed variable references, and the already
  supported focus-position part; it does not admit arbitrary XPath.
- Runtime variable lookup remains invocation-owned, charged, and bounded.
- Dynamic QName and namespace errors remain structured runtime failures.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_variable_xsl_element_name_composes_static_text
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/axes_axes15#1
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/BVTs_bvt073#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
