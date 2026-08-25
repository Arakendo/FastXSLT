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
```

The initial `hello` case is a portable design seed, not an executable test yet
and not a commitment to an XSLT version. The first vertical slice will add a
runner that reports semantic results separately from serialization differences.

External suites must live in a separately documented location with provenance,
license, acquisition, integrity, and selection instructions. Do not copy a
downloaded suite into `golden/`.

