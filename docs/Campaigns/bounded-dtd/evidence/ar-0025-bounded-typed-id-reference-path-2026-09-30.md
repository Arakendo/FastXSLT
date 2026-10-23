# AR-0025 Bounded Typed-ID Reference Path

## Purpose

Record the first typed-attribute and XPath `id()` tranche built on AR-0025's
private bounded internal-subset parser. This remains measurement-only evidence;
ordinary FastXSLT parsing continues to reject every DTD.

## Implemented reference slice

The private safe-Rust internal-subset parser now admits non-defaulting
`ATTLIST` declarations for `CDATA`, `ID`, and `IDREF` attributes using only
`#REQUIRED` or `#IMPLIED`. It rejects default/fixed values and all other
attribute types rather than discarding observable declaration semantics.

The XML adapter resolves the declaration against lexical element and attribute
names, applies XML tokenized-attribute whitespace normalization to `ID` and
`IDREF`, and emits an explicit `is_id` fact. XDM consumes that fact into a
private per-document ID index. Parser-native declaration objects do not cross
the XML boundary.

The XSLT 1.0 `id()` evaluator now:

- accepts the already compiled string-literal or node-set argument forms;
- splits each resulting string on XML whitespace;
- looks up only explicitly DTD-typed IDs;
- returns unique elements in document order, independently of token order;
- supports the existing bounded relative-path continuation; and
- charges node-string evaluation and lookup work through invocation control.

DTD-free documents retain the previous result: an ordinary attribute named
`id` is not silently treated as type ID.

## Corpus result

The complete hash-verified OASIS XSLT/XPath 1.0 CD04 denominator remains 3,173
catalog cases. The conserved direct DTD frontier remains 117 cases with the
same 87-source/30-stylesheet role split and declaration-property inventory.

The bounded reference parser now reports:

| Reference outcome | Cases |
| --- | ---: |
| Parsed | 12 |
| Explicitly unsupported declaration/external semantics | 105 |
| Conserved total | 117 |

All ten newly admitted typed-ID source cases initialize and execute. Eight
compare exactly. The other two, `Microsoft/Keys__91832` and
`Microsoft/Keys__91833`, use the archive's manual comparator and therefore do
not receive manufactured exact-comparison credit.

The complete sweep moves from 2,350 to **2,358 exact comparisons out of 3,173
(74.31%)**. Initialization moves from 2,539 to 2,549 and successful execution
from 2,493 to 2,503. The seven visible XML mismatches, four comparator gaps,
and 423 / 431 expected-error credit remain unchanged. Reaching 75.00% now
requires 22 additional exact cases.

## Focused verification

Focused tests prove:

- ID and IDREF whitespace normalization while CDATA whitespace is preserved;
- explicit ID marking at the XML event boundary;
- literal and node-set `id()` arguments;
- document-order normalization and duplicate removal;
- relative attribute selection after `id()`;
- unchanged empty lookup on DTD-free untyped attributes; and
- unchanged default DTD denial at the ordinary workbench constructor.

The complete verification gate is recorded separately by the repository run
that accompanies this evidence.

## Non-claims

- Production DTD support is not enabled.
- This is not a validating XML processor and does not infer ID type from an
  attribute name.
- Defaulted/fixed attributes, enumerations, IDREFS, NMTOKEN(S), ENTITY(IES),
  notations, external subsets, and external entities remain unsupported.
- The two manual-comparison cases are executions, not exact-match credit.
- This compatibility evidence does not establish XSLT 1.0 conformance.
