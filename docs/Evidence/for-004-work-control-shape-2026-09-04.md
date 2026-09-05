# `for-004` Work-Control Shape Measurement

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Source checkpoint | `ee659758a867fa6698e6468043f554223f73d15c` plus the uncommitted performance review |
| Status | Charge inventory complete; production candidate later measured and rejected |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |

## Question

After both obvious scan candidates failed the host-visible retention gate, how
much control traffic does the exact `for-004` evaluator perform, and is there
enough evidence to change budget or cancellation mechanics?

## Exact charge shape

A retained ignored release-mode probe replays the evaluator's current domain
sequence without executing XML/XDM work. For `N` selected items it performs:

- `4N` XPath-node-visit charges: one binding-path child plus the current
  one-then-two attribute visits per item;
- `2N` XDM-string-value charges;
- `2N + 1` XPath-operation charges: multiplication, addition, and final format;
- `8N + 1` total charge calls.

The probe verifies consumed totals after every sample. It does not change the
production evaluator or `InvocationControl`.

| Items | Charge calls | Test-loop baseline | Test-loop charged | Delta |
| ---: | ---: | ---: | ---: | ---: |
| 5 | 41 | 0.004 us | 0.049 us | 0.044 us |
| 50 | 401 | 0.045 us | 0.508 us | 0.464 us |
| 500 | 4,001 | 0.457 us | 4.681 us | 4.224 us |
| 5,000 | 40,001 | 4.533 us | 46.266 us | 41.733 us |

Values are medians of five samples, normalized per replay. They show linear
control traffic at about 1.04-1.06 ns above the matching baseline per call in
this test build.

## Critical limitation

These deltas are not production attribution. Rust test configuration compiles
test-only candidate-gap observation into `InvocationControl::charge`; the
production library does not. The probe therefore provides exact call counts and
a test-build upper-pressure signal, not a claim that production transforms
spend 4.224 us in control at 500 items.

A proposed temporary no-charge production A/B was rejected before build or
execution because it would globally weaken cancellation and budget enforcement
in a non-test artifact. The control implementation was restored immediately;
no weakened workbench executable ran.

## Disposition

Retain the ignored charge-shape inventory because it makes domain counts and
linear scaling reproducible. Do not optimize or weaken work control from its
timings. A next experiment must preserve every charge's pre-work budget check,
cancellation observation, failure domain, accepted-work count, and deterministic
fault behavior while comparing a narrow implementation technique through the
real host boundary.

The two P1 scan candidates are closed. A later production-shaped candidate kept
every charge and control check while removing test-only state from release code
and making the small dispatch eligible for inlining. Its longer ASP.NET
comparison was contradictory across sequential and four-way lanes, so it was
rejected and the production implementation restored.

[Production experiment](for-004-work-control-production-experiment-2026-09-05.md)
