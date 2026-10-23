# OASIS XSLT 1.0 context-name condition and XML line endings -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-comparator evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a boolean condition compare the lexical name of its context node without
losing source prefixes, and can the local XML comparator avoid rejecting an
otherwise equivalent result solely because the expected XML uses literal CRLF
line endings?

## Implemented slice

The shared boolean compiler now retains exact `name() = 'lexical-name'` and
`name(.) = 'lexical-name'` conditions, including the symmetric literal-first
spelling. Runtime evaluation requires a source-node context, charges the node
visit, reconstructs the lexical QName from the source name and prefix, and uses
the already-selected codepoint or HTML ASCII-insensitive string comparison.
The representation remains private and its retained lexical capacity is
included in prepared-state accounting.

The test-only OASIS XML comparator now performs XML 1.0 source line-ending
normalization before parsing. It also removes whitespace immediately following
an XML declaration, which is outside the document element and has no result-
tree significance. Normalization happens before entity and character-reference
expansion: a literal carriage return therefore becomes a line feed, while
`&#13;` remains a carriage return in the parsed result.

## Corpus result

Unchanged Lotus `axes_axes121` now evaluates `name(.) = 'center'` against each
selected descendant, emits the required line feed after `center`, and compares
exactly with its CRLF-serialized expected XML.

Across the complete sweep, six previously executing XML mismatches become
exact comparisons after the comparator stops treating XML source line-ending
and post-declaration whitespace spelling as result-tree differences. No case
changes initialization or execution disposition.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,705 | 1,705 | 0 |
| Initialization failures | 1,430 | 1,430 | 0 |
| Executed successfully | 1,522 | 1,522 | 0 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,388 | 1,394 | +6 |
| XML comparison mismatches | 107 | 101 | -6 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,394 / 2,742 = 50.84%` for standard-operation cases and
`1,394 / 3,173 = 43.93%` for the complete catalog.

## Boundaries

This slice admits only context-node lexical-name equality with a string
literal. It does not add general function-call comparison, namespace-axis
support, temporary-tree name semantics, or DTD authority. Microsoft
`Completeness__84351`, which independently contains the same condition shape,
remains rejected during source initialization because its principal input has
a DTD and FastXSLT's existing authority boundary forbids it.

The comparator change belongs only to the local compatibility measurement. It
does not alter engine XML parsing, result construction, serialization, or a
public equality contract.

## Verification

- A focused runtime test proves lexical-prefix preservation and both matching
  and non-matching context-name conditions.
- Comparator tests prove literal CRLF/CR normalization and prove that the
  `&#13;` character reference is not mistaken for a literal source carriage
  return.
- The full 3,173-case sweep records six mismatch-to-pass movements, no
  initialization/execution movement, and no panic.
- Formatting, strict Clippy, all workspace tests, documentation, Markdown
  links, unsafe-surface checks, and corpus inventories pass through
  `scripts/verify.ps1`.
