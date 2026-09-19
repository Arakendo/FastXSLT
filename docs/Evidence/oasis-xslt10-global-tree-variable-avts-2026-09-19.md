# OASIS XSLT 1.0 Global Tree-Variable AVTs

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Why could a literal-result attribute use a local text-tree variable but report
the same kind of global variable or empty global parameter as unbound?

## Finding

The literal-result AVT runtime had a private variable conversion that inspected
only invocation-local atomics and temporary trees. It bypassed global fallback,
source-node values, atomic sequences, and declared empty sequences even though
the shared XSLT 1.0 value runtime already owned all of those rules.

This was observable in the ordinary Microsoft photograph AVT fixture, whose
`$image-dir` is a global variable constructed from text, and four namespace-
alias fixtures whose `name="{$variable-name}"` consumes a global parameter.

## Repair

- Route AVT variable string conversion through the shared charged XSLT 1.0
  conversion owner.
- Preserve the literal attribute's stylesheet location on a conversion failure.
- Retain lexical shadowing, global fallback, budgets, cancellation, typed AVT
  plans, namespace behavior, and result construction.
- Add a focused global text-tree plus empty-global-parameter AVT case.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

The focused case produces `/images/headquarters.jpg` from a global text-tree
variable and an empty attribute value from a declared empty parameter.

All five unchanged Microsoft cases now execute. `AVTs__77536` becomes an exact
XML-semantic pass. The four namespace-alias cases expose later, independently
visible namespace/comparison behavior: three comparison mismatches and one
unsupported comparator input.

The exact-result lower bound rises from 1,618 to 1,619. Successful execution
rises from 1,903 to 1,908, execution failures fall from 105 to 100, visible
mismatches rise from 214 to 217, comparator-unsupported results rise from 64 to
65, and the `FXRT0002` execution frontier falls from 13 to 8. Initialization
remains 2,008.

## Boundary conclusion

This removes an AVT-specific variable-kind split. It does not add another
variable representation, broaden AVT grammar, claim the exposed namespace-
alias results as correct, or weaken XML-semantic comparison.

