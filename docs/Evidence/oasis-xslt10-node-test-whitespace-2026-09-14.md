# OASIS XSLT 1.0 node-test whitespace -- 2026-09-14

Date: 2026-09-14  
Status: Verified syntax and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the private location-path parser honor XPath whitespace between a node-test
name and its opening parenthesis without broadening arbitrary function-call
support?

## Implemented slice

After the existing bounded path-step parse and axis-whitespace normalization,
the parser now canonicalizes XPath whitespace around the parentheses of
recognized node tests: `node`, `text`, `element`, `comment`, and
`processing-instruction`. A processing-instruction argument is admitted only
when the existing named-target parser accepts it. Other function-shaped steps
remain unsupported or invalid under the existing classifier.

The canonicalized step lowers through the existing typed path representation;
there is no runtime branch, new authority, or alternate evaluator.

## Corpus result

Unchanged Lotus `select_select18` uses `comment ()`, selects the source comment,
copies it, and becomes an exact expected-result match.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,705 | 1,706 | +1 |
| Initialization failures | 1,430 | 1,429 | -1 |
| Executed successfully | 1,522 | 1,523 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,394 | 1,395 | +1 |
| XML comparison mismatches | 101 | 101 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,395 / 2,742 = 50.88%` for standard-operation cases and
`1,395 / 3,173 = 43.96%` for the complete catalog.

## Boundaries

This is lexical XPath whitespace handling for node tests only. It does not
admit arbitrary functions as path steps, generalized token rewriting, namespace
nodes, schema node tests, or new node kinds.

## Verification

- Focused syntax tests cover spaced `comment`, `text`, and named
  `processing-instruction` node tests, including explicit-axis spelling.
- The full 3,173-case sweep moves one case through initialization and execution
  to an exact result without introducing a mismatch or panic.
- Formatting, strict Clippy, all workspace tests, documentation, Markdown
  links, unsafe-surface checks, and corpus inventories pass through
  `scripts/verify.ps1`.
