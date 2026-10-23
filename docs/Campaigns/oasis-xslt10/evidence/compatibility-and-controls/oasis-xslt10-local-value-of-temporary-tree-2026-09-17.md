# OASIS XSLT 1.0 Local `value-of` Temporary Tree -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The runtime already represented XSLT 1.0 result-tree fragments as immutable
invocation-owned temporary trees and already admitted a global binding whose
sole constructor was `xsl:value-of`. The equivalent local binding fell through
to the static literal-element constructor, where the XSLT instruction was
reported as `FXST1015`. In addition, template-argument lookup could transfer
atomic and source-node variables but omitted the already-supported temporary-
tree parameter value kind.

## Repair

Under XSLT 1.0 static context only, the compiler now admits a local variable
whose complete content is one `xsl:value-of` with an admitted location path.
Execution evaluates that path relative to the current source context, applies
XSLT 1.0 first-node string conversion, and materializes the result through the
same charged parentless-text temporary-tree constructor used by the global
form. A variable-valued template argument may now carry that immutable
temporary tree into the callee's invocation-local parameter frame.

The slice does not admit general sequence constructors, arbitrary value
expressions, multiple constructor instructions, later-version result-tree-
fragment behavior, or cross-invocation sharing. Source traversal, string-value
construction, temporary-node construction, and parameter binding retain their
existing budget, cancellation, and safe clone-on-write behavior.

## Corpus result

Five unchanged Lotus named-template cases move from initialization failure to
exact XML comparison:

- `namedtemplate01`
- `namedtemplate02`
- `namedtemplate08`
- `namedtemplate14`
- `namedtemplate15`

The exact-result lower bound rises from 1,518 to 1,523. Initialized cases rise
from 1,866 to 1,871, executed-successfully cases rise from 1,667 to 1,672, and
the `FXST1015` initialization frontier falls from 48 to 40. No mismatch,
comparator-unsupported outcome, expected-error disposition, or panic is added.

## Verification

A focused runtime test proves current-context selection, first-node conversion,
temporary-tree construction, named-template argument transfer, and output from
the callee. The complete local measurement conserves all 3,173 identities:
1,871 initialize, 1,672 execute successfully, 1,523 compare exactly, 120 remain
visible mismatches, and 23 reach comparator-unsupported outcomes. Expected-
error accounting remains 403 initialization observations, 21 execution
observations, and four doubt-annotated unexpected successes.
