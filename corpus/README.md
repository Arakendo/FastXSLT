# FastXSLT Corpus

The corpus contains small, reviewed examples and eventually imported standards
suites that pressure engine behavior and architecture.

Corpus evidence can reveal a missing semantic, an incorrect boundary, or a
performance problem. It does not silently widen the public contract or prove
standards conformance. Conformance claims must name the standards edition,
suite version, selection policy, exclusions, environment, and exact results.

## Layout

```text
golden/<case>/input.xml       Source document
golden/<case>/stylesheet.xsl Stylesheet
golden/<case>/expected.xml   Expected serialized result
overlays/<suite>/*.toml      First-party selections and classifications
```

Future first-party adversarial and performance inputs may use named owner or
workload directories once real cases exist. They remain distinct from immutable
external suites:

```text
adversarial/<owner>/<case>/  Hostile/boundary fixtures with named expected controls
performance/<workload>/     Correctness-gated benchmark inputs and manifests
```

Conformance asks whether selected standards behavior is correct. Adversarial
cases ask whether bounded hostile work terminates predictably. Performance cases
measure cost under a recorded configuration. A fixture may inform another
family, but its provenance, classification, and reported claim must not silently
change.

The initial `hello` case now executes through a test-only private vertical slice
that asserts its semantic result separately from serialization. It remains a
portable design seed, not a public feature or commitment to an XSLT version.

External suites must live in a separately documented location with provenance,
license, acquisition, integrity, and selection instructions. Do not copy a
downloaded suite into `golden/`. The XSLT30 overlay selects executable upstream
case `template-006` and preserves `avt-0701` as a visible compound-assertion
harness gap. The QT3 overlay preserves `Axes001-1` as selected but beyond the
current private engine slice. Tests load metadata and fixtures from pinned
submodules without modifying or duplicating them.
