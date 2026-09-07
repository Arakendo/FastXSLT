# OASIS XSLT 1.0 Number Format Sequence and Grouping Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | Latin alphabetic, Roman, multi-token, separator, and static decimal-grouping number formats |
| Disposition | 32 new expected-result matches; one newly exposed later execution failure remains uncredited |

## Change

Number formatting now compiles into an immutable sequence of typed format
tokens and literal separators rather than one decimal token. Admitted tokens
are zero-padded decimal, Latin alphabetic `A`/`a`, and Roman `I`/`i`.
Multiple-level numbering applies successive tokens and separators, reusing the
last token and separator when the number list is longer than the picture.
Prefix and suffix punctuation are retained exactly.

Static `grouping-separator` plus positive-integer `grouping-size` are compiled
into the same plan and applied to decimal tokens after minimum-width padding.
Supplying only one grouping attribute preserves ungrouped output. Dynamic AVTs,
invalid multi-character separators, non-Latin numbering alphabets, and
`letter-value` remain explicit boundaries.

Known compiled retention includes token and separator vector capacity plus all
owned punctuation. Focused production tests cover alphabetic rollover, Roman
subtractive forms, mixed multi-level token pictures, repeated separators, and
static decimal grouping.

## Measurement

The tranche accumulated in three measured checkpoints:

- single alphabetic/Roman tokens: 915 to 923 expected-result matches;
- compiled multi-token/separator plans: 923 to 942;
- static decimal grouping: 942 to 947.

The complete conserved delta is:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,236 | 1,269 | +33 |
| Executed successfully | 1,025 | 1,057 | +32 |
| Expected-result XML matches | 915 | 947 | +32 |
| XML comparison mismatches | 76 | 76 | 0 |
| Execution failures | 211 | 212 | +1 |

`Microsoft/Number__84687#1` now reaches the existing multi-node value-of
boundary and reports `FXRT1001`; it remains visibly uncredited. The strict
lower bound over 2,742 standard-operation cases is now 34.54%, and the complete
catalog ratio is 29.85%. Neither is an XSLT 1.0 conformance claim.

