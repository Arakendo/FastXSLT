# Template-Candidate Fanout and Cancellation Gap

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review pressure | Adversarial review Finding 7; AR-0010; AR-0013 |
| Scope | Private source and temporary-tree matched-template selectors |
| Toolchain | `rustc 1.95.0`; optimized tests; x86-64 Windows 10.0.26200; AMD Family 25 Model 97 |
| Claim | A separate candidate work domain and one local charge/check per considered candidate close the confirmed accounting and cancellation gap; no dispatch index selected |

## Instrumentation

All five private source and temporary-tree matched-template scan paths now
charge one `xslt-template-candidate` work unit immediately before testing each
candidate. This domain is distinct from entered XSLT instructions and XPath
node visits. The workbench's private bounded profile admits 1,000,000 candidates
per invocation; direct reference execution may still supply another explicit
limit or remain unbounded.

A test-only bypass retains the formerly uncharged semantic path solely for
overhead comparison. Candidate observations remain test-only and semantically
inert; ordinary builds perform the charge without retaining telemetry.

A focused regression compiles 32 nonmatching exact-name templates plus one
matching template, dispatches 16 source elements, and observes exactly
`33 × 16 = 528` candidate charges with a maximum one-candidate interval between
checks. XSLT instruction accounting remains separate and unchanged.

## Cancellation counterexample

A deterministic test signal injected at the first candidate is now observed by
that candidate's charge as `FXCT0001 / cancelled` in the
`xslt-template-candidate` domain. No later candidate is inspected. A zero-unit
candidate limit likewise returns `FXCT0002 / limit` before the first pattern
test and retains request plus work-domain identity.

## Optimized sweep

Five samples were taken for each generated workload; the table reports the
median complete execution time. Each selected source node scans every matched
template. Result construction is intentionally retained and identical within a
source-node tier, so these are mechanism observations rather than a general
engine benchmark.

| Matched templates | Source nodes | Candidates | Maximum gap | Uncharged median | Charged median |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 9 | 8 | 72 | 1 | 3.4 us | 3.4 us |
| 9 | 64 | 576 | 1 | 30.1 us | 26.3 us |
| 9 | 256 | 2,304 | 1 | 141.8 us | 114.6 us |
| 33 | 8 | 264 | 1 | 3.8 us | 4.0 us |
| 33 | 64 | 2,112 | 1 | 25.1 us | 27.8 us |
| 33 | 256 | 8,448 | 1 | 123.2 us | 111.9 us |
| 129 | 8 | 1,032 | 1 | 8.1 us | 9.2 us |
| 129 | 64 | 8,256 | 1 | 73.5 us | 72.2 us |
| 129 | 256 | 33,024 | 1 | 240.9 us | 284.2 us |

The paired timings are deliberately interleaved but remain locally noisy; small
samples invert, so they do not establish a stable cost per charge. The largest
shape measured about 18% overhead while reducing its cancellation/check gap
from 129 candidates to one and making all 33,024 checks budget-visible. This is
mechanism evidence, not an ASP.NET performance claim or a default-limit
calibration.

## Disposition

Finding 7 is **completed**. Every implemented candidate scan now has honest
layer-owned accounting, deterministic limit identity, and a one-candidate
cooperative observation interval. The internal one-million-unit workbench bound
is an experimental ceiling, not a supported public default.

An activated safe dispatch index remains an independent AR-0013 optimization
question. It may reduce charged work, but it is not required to close the
resource-contract defect and is not admitted by this evidence.

## Reproduction

```text
cargo test -p fastxslt template_candidate --all-features
cargo test -p fastxslt cancellation_signalled_during_simple_pattern_scan_stops_at_the_candidate_charge --all-features
cargo test --release -p fastxslt measure_template_candidate_fanout --all-features -- --ignored --nocapture
```
