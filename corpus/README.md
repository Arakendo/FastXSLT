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

The initial `hello` case now executes through a test-only private vertical slice
that asserts its semantic result separately from serialization. It remains a
portable design seed, not a public feature or commitment to an XSLT version.

External suites must live in a separately documented location with provenance,
license, acquisition, integrity, and selection instructions. Do not copy a
downloaded suite into `golden/`. The first XSLT30 overlay selects upstream case
`template-006`; its test loads the source, stylesheet, and assertion from the
pinned submodule without modifying or duplicating those fixtures.
