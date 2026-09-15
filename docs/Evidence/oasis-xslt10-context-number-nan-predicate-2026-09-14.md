# OASIS XSLT 1.0 context-number NaN predicate -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing XSLT 1.0 numeric conversion semantics participate in a
context-dependent boolean predicate without admitting a general nested XPath
expression evaluator?

## Implemented slice

For version 1.0 stylesheets only, the instruction boolean compiler recognizes
the exact predicate `string(number(.)) = 'NaN'`, including its symmetric
literal form. It lowers the expression to a private typed boolean plan rather
than retaining source syntax or adding runtime version branching.

Execution obtains the context node's string value through the controlled XDM
operation, charges the numeric conversion as XPath work, and applies the
existing XPath 1.0 numeric lexical rules. The equivalent expression in a
version 3.0 stylesheet remains outside this compatibility slice.

## Corpus result

Unchanged Microsoft `Number__84431` moves from initialization failure to
successful execution. Its five numeric branch decisions are correct: malformed
lexicals are labelled `is not a number`, while valid integers and decimals are
not.

The case remains a comparison mismatch because source-document indentation is
emitted by XSLT 1.0 built-in template processing while the expected artifact
omits it. The upstream suite lists this identity in `doubts.xml`. FastXSLT does
not suppress the normative whitespace merely to obtain an exact match.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,727 | 1,728 | +1 |
| Initialization failures | 1,408 | 1,407 | -1 |
| Executed successfully | 1,544 | 1,545 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,411 | 1,411 | 0 |
| XML comparison mismatches | 106 | 107 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound therefore remains
`1,411 / 2,742 = 51.46%` for standard-operation cases and
`1,411 / 3,173 = 44.47%` for the complete catalog. Executable coverage rises
without turning a doubts-annotated expected artifact into a conformance credit.

## Boundaries and verification

This tranche does not admit general function composition, arbitrary comparisons,
non-context `number()` operands, or a modern XPath numeric-conversion contract.
Focused lifecycle evidence covers numeric and malformed context strings and
proves compile-time XSLT-version selection. The complete local corpus sweep
confirms the one-case frontier movement, unchanged execution-failure count, and
absence of panics.
