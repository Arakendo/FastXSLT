# OASIS XSLT 1.0 Static Namespace Alias -- 2026-09-17

Date: 2026-09-17  
Status: Verified single-module semantic slice and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

Forty-three unchanged cases stopped at the blanket unsupported top-level
`xsl:namespace-alias` diagnostic even though literal result names and namespace
bindings were already stylesheet-derived immutable state. This made a bounded
static rewrite preferable to a runtime namespace-policy branch.

## Repair

The compiler now validates `stylesheet-prefix` and `result-prefix`, resolves
ordinary and `#default` prefixes in the declaration's static namespace
context, detects conflicting aliases at one precedence, and rewrites compiled
literal result element names, literal attribute names, and retained
namespace bindings. Nested literal results and statically constructed temporary
trees use the same rewrite. An undeclared `#default` denotes the null namespace;
an undeclared ordinary prefix is `XTSE0812`.

The rewrite is compile-time only. Runtime construction, result ownership,
serialization, cancellation, and budgets are unchanged. Computed `xsl:element`
and `xsl:attribute` names are not rewritten. General alias precedence across
included/imported modules remains outside this tranche; the corpus keeps every
later comparison or execution boundary visible rather than treating frontier
movement as success.

## Corpus result

The 43-case blanket namespace-alias frontier is eliminated:

- 27 additional cases initialize;
- 22 additional cases execute successfully;
- eight become exact expected-result matches;
- fourteen reach visible XML comparison mismatches, predominantly archival
  discretionary indentation/empty-element spelling after the required aliased
  expanded names are produced;
- five reach later execution boundaries; and
- sixteen are classified by the declaration validator or another earlier
  precise static boundary.

The exact-result lower bound rises from 1,525 to 1,533. Initialized cases rise
from 1,876 to 1,903 and executed-successfully cases rise from 1,677 to 1,699.
Visible mismatches rise from 123 to 137 and execution failures from 199 to 204.
Expected-error accounting remains 403 initialization observations and 21
execution observations; the four doubt-annotated unexpected successes remain
visible. No corpus panic occurs.

## Verification

Focused compiler tests prove nested literal element and attribute rewriting,
the retained result prefix binding, required/unbound-prefix diagnostics, and
null-default-namespace aliasing. The complete local measurement conserves all
3,173 catalog identities: 1,903 initialize, 1,699 execute successfully, 1,533
compare exactly, 137 remain visible mismatches, and 23 reach comparator-
unsupported outcomes.
