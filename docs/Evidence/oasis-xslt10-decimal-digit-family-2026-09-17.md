# OASIS XSLT 1.0 Decimal Digit Family -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic slice and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding and implementation

The remaining zero-digit frontier uses XSLT 1.0's consecutive-character digit
family rule, including `zero-digit="a"` and the resulting `a` through `j`
digits. The formatter now validates that ten consecutive Unicode scalar values
exist and substitutes only digits produced by numeric formatting. Literal
prefixes, suffixes, NaN labels, and infinity labels are unchanged.

## Corpus result

Five additional cases initialize, one executes and compares exactly, and four
reach later explicit execution boundaries. The exact lower bound rises from
1,558 to 1,559. The conserved measurement is now 1,939 initialized, 1,725
executed successfully, 1,559 exact matches, 137 visible mismatches, and 23
comparator-unsupported outcomes. No panic occurs.

Six former digit-frontier cases are instead classified by other declaration or
expression boundaries. This tranche does not widen dynamic format-name lookup
or cross-module decimal-format composition.
