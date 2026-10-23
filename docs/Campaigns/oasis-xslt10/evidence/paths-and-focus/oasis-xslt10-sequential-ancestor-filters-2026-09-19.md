# OASIS XSLT 1.0 Sequential Ancestor Filters

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the compatibility path preserve reverse-axis position and sequential
predicate semantics for the two ancestor filters exercised by the remaining
Lotus `axes13` frontier?

## Implemented slice

Yes. Two exact XSLT 1.0 conditional forms now compile to an owned ancestor
filter plan:

- `ancestor::*[@attribute='literal'][not(text())]`; and
- `ancestor::*[positive-integer][@attribute]`.

Runtime walks element ancestors from the context node toward the document root,
so a positional filter uses reverse-axis proximity before the following
attribute predicate. The non-positional form applies its attribute-value and
absence-of-text-child predicates to the same candidate. Every ancestor,
attribute, and inspected child is charged before use. Traversal uses the
invocation's effective source document, so admitted whitespace views remain
authoritative.

The implementation does not rewrite sequential predicates into a conjunction,
does not broaden the generic path grammar, and does not admit arbitrary
predicate composition or modern expressions.

## Corpus result

The unchanged Lotus `axes13` case now executes both conditional forms alongside
its existing `name(ancestor::*[N])` expressions and matches its expected result
exactly. Against the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,052 | 2,053 | +1 |
| Initialization failures | 1,083 | 1,082 | -1 |
| Executed successfully | 1,950 | 1,951 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,821 | 1,822 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

A focused regression distinguishes an ancestor with no direct text child from
one with a direct text child, then proves that `[2][@new]` selects the second
ancestor before testing its attribute. It also preserves explicit modern
rejection. The unchanged corpus case provides text-template dispatch and
reverse-axis name-selection coverage. Full corpus accounting remains conserved
without a new mismatch, execution failure, or panic.
