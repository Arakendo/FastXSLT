# OASIS XSLT 1.0 Variable `xsl:number` Value

Date: 2026-09-28  
Status: Verified implementation evidence; corpus case reaches an independent host-policy boundary  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged `Microsoft/Miscellaneous__84435#1` uses a temporary-tree template
parameter as the `value` expression of `xsl:number`. The compiler previously
classified the direct variable reference as unsupported `FXXP1022`, despite
the runtime already owning the required XSLT 1.0 variable string-value rules.

## Change

The private numbering plan now retains a direct XSLT 1.0 variable reference as
a typed number-value operand. At execution it:

1. resolves atomic, sequence, source-node, temporary-tree, or global-fallback
   values through the existing charged XSLT 1.0 string-value evaluator;
2. applies XPath 1.0 string-to-number conversion; and
3. reuses the established XSLT 1.0 rounding and number-formatting path.

Modern stylesheet compilation is unchanged. No general dynamic expression or
second numbering evaluator was added. Retained-capacity accounting includes the
owned variable name.

## Corpus result

The unchanged Microsoft case clears `FXXP1022` and reaches its source document.
That document contains a DTD, which remains rejected by the explicit host parser
policy as `FXXM0002/source-input:dtd-forbidden`. The standard-operation
`FXXP1022` frontier is eliminated, but the exact lower bound remains **2,242 /
3,173 (70.66%)** and initialization/execution totals remain 2,440 / 2,392.

The classified DTD-source frontier rises from 86 to 87. This is frontier
movement, not pass credit and not authority to admit DTD processing.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_number_converts_a_variable_value_to_number -- --nocapture
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Miscellaneous__84435#1'
./scripts/verify.ps1
```
