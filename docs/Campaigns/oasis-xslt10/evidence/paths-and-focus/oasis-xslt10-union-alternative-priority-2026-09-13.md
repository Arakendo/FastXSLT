# OASIS XSLT 1.0 union-alternative priority -- 2026-09-13

Date: 2026-09-13  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT union match retain the default priority of each alternative even
when two alternatives can match the same node?

## Implemented slice

For a union whose alternatives have different default priorities, compilation
now lowers each alternative to its own private matched-template entry while
sharing the compiled modes and template body. This preserves the XSLT rule that
each union alternative behaves as a separate template rule for matching and
priority selection. Equal-priority overlapping unions retain the existing
grouped representation; homogeneous qualified path unions also remain on their
existing bounded path representation.

Focused execution proves the distinction with `text() | chapter/text()`: an
ordinary text node competes at node-test priority, while a text child of
`chapter` wins at path priority.

## Corpus result

Five Microsoft cases leave the generic `FXST1005` overlapping-union frontier.
Two execute to exact expected results; three reach later independent compiler
boundaries and remain uncredited.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,679 | 1,681 | +2 |
| Executed successfully | 1,508 | 1,510 | +2 |
| Expected-result XML matches | 1,377 | 1,379 | +2 |
| Initialization failures | 1,456 | 1,454 | -2 |
| Execution failures | 171 | 171 | 0 |
| XML comparison mismatches | 104 | 104 | 0 |

The newly exact cases are `Microsoft/BVTs_bvt078#1` and
`Microsoft/ConflictResolution__84477#1`. The other three cases expose template
parameter and expression boundaries rather than disappearing from the report.

The strict standard-operation lower bound is now
`1,379 / 2,742 = 50.29%`; the conservative all-catalog ratio is
`1,379 / 3,173 = 43.46%`.

## Boundaries

This does not select a general union-pattern representation, alter explicit
priority, weaken multiple-match diagnostics, or merge semantically distinct
rules. It removes only the former restriction that required overlapping union
alternatives with different default priorities to fail compilation.

## Verification

- Focused compiler coverage checks separate retained priorities.
- Focused runtime coverage checks priority selection on overlapping text-node
  alternatives.
- The complete 3,173-case local measurement produced the counters above and
  reduced the generic match frontier from ten cases to five.
