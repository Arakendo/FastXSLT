# OASIS XSLT 1.0 Direct Numeric-Literal String Conversion

- Date: 2026-09-26
- Status: Verified semantic and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Implemented slice

The XSLT 1.0 value-expression compiler now recognizes direct XPath numeric
literals, including a leading minus and the permitted leading-dot spelling,
before attempting to parse the expression as a location path. It converts the
literal through the existing finite `f64` compatibility path and retains the
result as immutable literal text.

This is compatibility behavior selected only by an XSLT 1.0 static context.
It does not change modern XPath decimal typing, introduce runtime floating-
point state, or create a second value evaluator.

## Evidence

The focused constant-numeric test covers negative integers, leading-dot
decimals, and a value whose XPath 1.0 double string value differs from its
source lexical form.

Unchanged Lotus `string132`, `string133`, `string134`, and `string135` now
compile, execute, and compare exactly. Together they exercise positive and
negative integer and decimal literals across double precision and very small
decimal magnitudes. The pre-existing OASIS reference comparison remains the
oracle; no fixture or expected output changed.

The apparently adjacent Microsoft `Attributes__89459` case remains outside
this slice because its numeric token occurs in a different instruction grammar,
not an `xsl:value-of` value expression.

## Conserved result

The sweep moves from 2,149 to **2,153 exact matches (67.85%)**, from 2,357 to
**2,361 initialized** cases, and from 2,306 to **2,310 successful executions**.
Initialization failures fall from 813 to 809. Execution failures remain 51,
expected-error credit remains 423 / 431, comparator gaps remain 67, and visible
XML mismatches remain zero.

## Verification

```powershell
cargo test -p fastxslt --all-features folds_xpath10_numeric_literals_through_double_string_conversion
./scripts/measure-oasis-xslt10.ps1 -TraceCase string_string132
./scripts/measure-oasis-xslt10.ps1 -TraceCase string_string133
./scripts/measure-oasis-xslt10.ps1 -TraceCase string_string134
./scripts/measure-oasis-xslt10.ps1 -TraceCase string_string135
./scripts/verify.ps1
```
