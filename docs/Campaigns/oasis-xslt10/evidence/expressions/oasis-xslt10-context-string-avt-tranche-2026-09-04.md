# OASIS XSLT 1.0 Context String-Value AVT Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 737 definite unchanged XML passes; 1,019 initialized cases |
| Result | 738 definite unchanged XML passes; 1,022 initialized cases |
| Disposition | Shared context-item string-value operation; not a conformance claim |

## Change

The literal-result-attribute compiler now recognizes the exact attribute value
template `{.}` as a typed context string-value operation. Execution computes the
complete XPath string value for source and temporary nodes through their owning
tree representations, uses the lexical value for the admitted integer atomic
focus, and reports `XPDY0002` when no context item exists.

String-value traversal is charged at the nodes visited. The compiler records
whether a literal-result element needs this value, so attributes that do not use
`{.}` retain the prior execution path and perform no additional traversal or
allocation. This tranche does not admit mixed literal/expression AVTs or a
general AVT expression parser.

## Corpus disposition

Three standard-operation cases cross initialization and execution:

- `Lotus/attribvaltemplate_attribvaltemplate01#1` becomes a definite XML pass;
- `Microsoft/AVTs__77574#1` reaches a visible comparison mismatch because the
  expected attribute retains CRLF through character references while the
  parsed source string value has XML-normalized LF; and
- `Microsoft/AVTs__77591#1`, which carries suite doubts metadata, reaches the
  same visible line-ending mismatch over its large buffer case.

The two mismatches receive no compatibility credit. Their traces show the
expected source text is otherwise present, including the large value; this
tranche does not change XML line-ending or comparison policy to manufacture a
pass.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,019 | 1,022 | +3 |
| Execution succeeded | 818 | 821 | +3 |
| XML comparison passes | 737 | 738 | +1 |
| XML comparison mismatches | 48 | 50 | +2 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **738 / 2,742 = 26.91%** of standard-operation
cases and **738 / 3,173 = 23.26%** of the complete catalog.

## Verification

A focused compiler test preserves `{.}` as a typed operation. A runtime test
executes the same compiled form over source and temporary elements containing
nested text and proves identical descendant-text concatenation. The complete
local OASIS sweep and focused traces preserve both non-passing cases as visible
evidence rather than folding them into the pass count.
