# OASIS XSLT 1.0 Path-Substring Sort Key

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged `Lotus/sort_sort22#1` uses
`xsl:sort select="substring(@key,2,1)"`. FastXSLT already owned the required
XPath 1.0 first-node string conversion, numeric rounding, Unicode-codepoint
substring semantics, stable sort, and charged path evaluation, but sort-key
compilation did not compose those existing parts.

## Implemented slice

XSLT 1.0 sort compilation now reuses the existing typed path-substring plan
for a location-path first argument and one or two finite numeric literal
arguments. Per candidate, execution evaluates the path, converts its first
node in document order to a string, charges the substring operation, and uses
the resulting string as the ordinary stable sort key.

Prepared-state accounting includes the retained typed substring plan and path.
A focused runtime test covers an attribute path and verifies the resulting
stable order.

This does not admit dynamic substring bounds, variable operands, a general
sort-expression evaluator, or locale/case-order semantics. The latter remain a
separate `FXST1063` boundary and the implementation does not use this tranche
to imitate the implementation-dependent ordering of `sort08` or `sort27`.

## Corpus result

`Lotus/sort_sort22#1` now initializes, executes, and compares exactly. The
complete sweep moves from:

- 2,291 to 2,292 initialized cases;
- 2,239 to 2,240 successfully executed cases; and
- 2,080 to 2,081 exact expected-result matches.

Initialization failures fall from 844 to 843. Execution failures and XML
mismatches remain 52 and 76. The exact compatibility lower bound is therefore
**2,081 / 3,173 (65.58%)**.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_sort_uses_a_bounded_attribute_substring_key
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/sort_sort22#1'
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
