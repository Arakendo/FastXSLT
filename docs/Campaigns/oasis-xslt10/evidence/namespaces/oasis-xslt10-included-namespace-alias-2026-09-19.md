# OASIS XSLT 1.0 Included Namespace Alias

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The unchanged Lotus `namespace113#1` case executed but produced one literal
result subtree from an included module in the stylesheet alias namespace. The
same names constructed by the principal module were correctly rewritten to the
declared result namespace.

This was a real static-composition defect: an `xsl:include` behaves as if the
included declarations were present in the including module, so a namespace
alias declared by the principal module must also rewrite literal result names
compiled from included templates.

## Repair

- Reuse the existing namespace-alias declaration compiler to collect aliases
  from the principal module during include-graph composition.
- Apply those typed aliases to each already compiled included program before
  its templates, bindings, keys, and other static state are merged.
- Cover the single-include, two-include, and mixed import/include composition
  paths that currently admit included programs.
- Keep rewriting entirely in compiled stylesheet state; execution performs no
  namespace-alias lookup and gains no new resource access.

## Result

A focused sealed-resource regression proves that one principal alias rewrites
literal result elements from both principal and included templates. The
unchanged Lotus `namespace113#1` result is now XML-semantically equal to its
expected document.

The complete 3,173-case sweep remains at 2,026 initialized and 1,925
successfully executed cases. Exact XML-semantic matches rise to 1,795 and the
visible mismatch set falls to 56.

## Boundary conclusion

This repairs propagation of a principal module's already validated namespace
aliases into admitted included programs. Full conflict and import-precedence
composition among aliases declared by different modules remains separate work;
this result does not claim that broader namespace-alias graph is complete.
