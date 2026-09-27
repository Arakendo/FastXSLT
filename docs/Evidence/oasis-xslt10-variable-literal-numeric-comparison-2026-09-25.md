# OASIS XSLT 1.0 Variable/Literal Numeric Comparison

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

FastXSLT already owned XSLT 1.0 numeric conversion for runtime variables,
ordered numeric comparison, named-template recursion, and numeric template
arguments. Its compatibility boolean compiler nevertheless admitted ordered
comparison only when both operands were variables. Standards-valid tests such
as `$foo < 2` therefore fell through to the location-path parser and stopped at
generic `FXXP1001` before the existing runtime semantics could execute.

Two unchanged Microsoft BVT cases exposed the same gap:

- `BVTs_bvt008#1` recursively calls a named template while `$foo < 2`;
- `BVTs_bvt062#1` recursively constructs nested output while
  `$nested-level < 20`.

## Implemented slice

The XSLT 1.0 boolean compiler now retains a typed comparison between one direct
unqualified variable and one finite static XPath 1.0 numeric literal. Either
operand order is accepted and the relational operator is reversed during
compilation when the literal appears first. Runtime converts the variable
through the existing XSLT 1.0 number path and uses the shared XPath numeric
comparison implementation.

The parser intentionally accepts only the XPath 1.0 decimal-literal shape. It
does not add a general expression grammar, exponent notation, non-finite
literals, dynamic operands, modern implicit conversions, or another numeric
evaluator. Compiled-retention accounting includes the owned variable name.

A focused runtime regression recursively advances a named-template parameter
from one through three and proves the literal comparison terminates the loop.

## Corpus result

Both unchanged Microsoft cases initialize, execute, and compare exactly. The
complete sweep advances from **2,330 to 2,332 initialized**, from **2,283 to
2,285 successfully executed**, and from **2,118 to 2,120 / 3,173 exact matches
(66.81%)**. Initialization failures fall from 840 to 838; execution failures,
mismatches, exclusions, and comparator gaps are unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_recursive_numeric_parameters_compare_against_literal_bounds
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt008
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt062
./scripts/verify.ps1
```
