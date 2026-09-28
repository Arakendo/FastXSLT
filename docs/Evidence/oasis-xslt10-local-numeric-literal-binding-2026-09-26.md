# OASIS XSLT 1.0 Local Numeric-Literal Binding

- Date: 2026-09-26
- Status: Verified semantic and corpus-frontier evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Two standard-operation Microsoft cases bind extremely small XPath 1.0 numeric
literals to local variables. The compiler previously sent those literals to a
cast-expression fallback and reported `unsupported/FXXP1008`. Direct
`xsl:value-of` numeric literals already used the bounded XPath 1.0 finite-double
conversion owner, but local bindings did not.

## Change

XSLT 1.0 local variable compilation now recognizes a finite numeric literal
after the existing string, integer, boolean, and exact-integral static forms.
The compiled binding is an immutable `xs:double` atomic value. Modern
stylesheets retain their existing typing path, and no runtime parser or
floating-point state was added.

The focused compiler test verifies that a positive subnormal literal remains a
finite, nonzero double binding rather than becoming a path or cast expression.

## Corpus observation

Unchanged cases `Microsoft/XSLTFunctions__minimalValue` and
`Microsoft/XSLTFunctions__minimumValue` now initialize and execute. Both remain
uncredited visible mismatches. Their archival expected outputs use a legacy
scientific rendering for the subnormal value, and both cases carry the suite's
doubt metadata; FastXSLT retains the XPath 1.0 decimal string form.

Consequently:

- exact matches remain **2,153 / 3,173 (67.85%)**;
- initialized cases rise from 2,361 to **2,363**;
- successful executions rise from 2,310 to **2,312**;
- initialization failures fall from 809 to **807**;
- the `FXXP1008` frontier falls from five cases to **three**; and
- two doubt-annotated XML mismatches become visible.

The three remaining `FXXP1008` cases bind source nodes from admitted secondary
documents. They retain the separate cross-document node-ownership and lifetime
boundary and are not inferred from numeric-literal support.

## Verification

```powershell
cargo test -p fastxslt xslt10_local_numeric_literal_retains_double_value
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```

