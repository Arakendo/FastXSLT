# OASIS XSLT 1.0 Template-Argument Path-Union Count

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can an XSLT 1.0 `xsl:with-param` carry the integer result of `count()` over a
path union without introducing a second union evaluator or erasing the atomic
argument type?

## Method

- Admit at most eight typed XSLT 1.0 location paths inside a template-argument
  `count()` union.
- Evaluate the alternatives through the ordinary charged path evaluator.
- Reuse shared document-order normalization and node-identity deduplication
  before counting.
- Pass the result as a typed integer invocation parameter.
- Add a complete call-template lifecycle covering
  `count(following::node() | following::*/@*)`, then trace unchanged Lotus
  `idkey_idkey30` and run the conserved measurement.

## Result

The focused lifecycle counts the two following nodes and one following-element
attribute, then observes the typed integer value `3` in the called template.
Unchanged Lotus `idkey_idkey30` clears its former `FXXP1011` template-argument
frontier and reaches a later independent conditional expression involving
`contains()`, `concat()`, and `generate-id()`.

The conserved totals remain 2,130 initialized cases, 2,041 successful
executions, 89 execution failures, 1,910 exact XML-semantic matches, and 55
mismatches. The strict compatibility lower bound therefore remains
`1,910 / 3,173 = 60.20%`.

## Boundaries

- The union is limited to eight already admitted XSLT 1.0 location paths.
- General function or variable operands inside the union remain separate typed
  expression families.
- The result is an invocation-owned integer parameter; no selected nodes or
  normalized union are retained across invocations.
- The later `idkey_idkey30` recursive string/identity boundary remains explicit
  and receives no partial approximation.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_template_arguments_count_normalized_path_unions
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/idkey_idkey30#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
