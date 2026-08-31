# Template-Candidate Fanout and Cancellation Gap

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 7; AR-0010; AR-0013 |
| Scope | Private source and temporary-tree matched-template selectors |
| Toolchain | `rustc 1.95.0`; optimized tests; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | Measurement evidence only; no budget unit, check frequency, or index selected |

## Instrumentation

All four private matched-template scan paths now report candidates considered to
a test-only invocation observation. Existing work charges delimit the number of
candidates traversed between cancellation observations. The hook is a no-op in
ordinary builds: it does not add a work domain, change a limit, check
cancellation, select telemetry, or alter template ranking.

A focused regression compiles 32 nonmatching exact-name templates plus one
matching template, dispatches 16 source elements, and observes exactly
`33 × 16 = 528` candidates with a maximum 33-candidate interval between charge
points. XSLT instruction accounting remains separate and unchanged.

## Cancellation counterexample

A deterministic test signal is injected after the first candidate in a
129-candidate simple-pattern scan. Exact-name and mode checks perform no work
charge, so the selector considers the remaining 128 candidates before the next
literal-result instruction observes `FXCT0001 / cancelled`. This confirms the
review's attribution and cooperative-observation gap without assigning those
candidates to an existing budget prematurely.

## Optimized sweep

Five samples were taken for each generated workload; the table reports the
median complete execution time. Each selected source node scans every matched
template. Result construction is intentionally retained and identical within a
source-node tier, so these are mechanism observations rather than a general
engine benchmark.

| Matched templates | Source nodes | Candidates | Maximum candidate gap | Median execution |
| ---: | ---: | ---: | ---: | ---: |
| 9 | 8 | 72 | 9 | 3.3 us |
| 9 | 64 | 576 | 9 | 40.8 us |
| 9 | 256 | 2,304 | 9 | 108.4 us |
| 33 | 8 | 264 | 33 | 4.2 us |
| 33 | 64 | 2,112 | 33 | 28.3 us |
| 33 | 256 | 8,448 | 33 | 123.2 us |
| 129 | 8 | 1,032 | 129 | 7.5 us |
| 129 | 64 | 8,256 | 129 | 75.7 us |
| 129 | 256 | 33,024 | 129 | 429.3 us |

The exact counts demonstrate the structural `selected nodes × matched
templates` mechanism. Timing is local and noisy—the 9-template/64-node sample
is visibly non-monotonic relative to 33 templates—so it does not establish a
stable cost per candidate. The largest measured shape nevertheless performs
33,024 unbudgeted candidate checks and permits 129 checks between cancellation
observations.

## Disposition

Finding 7 advances from “measurement required” to **confirmed; remediation
decision required**. The next step is to compare candidate charging/check
frequencies and, separately, an activated safe dispatch index. Any selected
mechanism must conserve template semantics, diagnostics, cancellation,
preparation cost, retained memory, and host-visible performance. This evidence
does not justify an index or a new public limit by itself.

## Reproduction

```text
cargo test -p fastxslt template_candidate --all-features
cargo test -p fastxslt cancellation_signalled_during_simple_pattern_scan_waits_for_the_next_charge --all-features
cargo test --release -p fastxslt measure_template_candidate_fanout --all-features -- --ignored --nocapture
```
