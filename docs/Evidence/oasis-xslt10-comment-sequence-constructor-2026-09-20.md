# OASIS XSLT 1.0 Comment Sequence Constructor

Date: 2026-09-20

## Question

Can dynamic XSLT 1.0 comment content use the same bounded text-constructor
machinery as computed attributes and processing instructions without creating
a comment-only evaluator or widening modern semantics?

## Implementation

After static folding, an XSLT 1.0 `xsl:comment` may retain an ordinary compiled
instruction sequence of at most 64 meaningful children. The body executes
through the reference instruction engine with the current focus, variables,
work limits, cancellation, and recursion controls. Only top-level text results
contribute comment data. Direct element, attribute, comment, copy, and
processing-instruction constructors are omitted with their content.

The completed dynamic value uses the same XSLT 1.0 lexical recovery as the
static path: a space is inserted after a hyphen that precedes another hyphen
or ends the value. Modern static contexts keep the existing `FXST1036` and
`FXST1037` boundaries.

The compiled body participates in named-template validation, decimal-format
binding, semantic inspection, and known-owned-capacity accounting. No public
representation, resource authority, or alternate execution backend was added.

## Executable evidence

The focused regression combines literal text, an ignored literal-result
element, a source-dependent `xsl:for-each`/`xsl:value-of`, `--`, and a trailing
hyphen. It produces exactly:

```xml
<out><!--prefixonetwo- -suffix- --></out>
```

The unchanged OASIS cases `Lotus/output_output55`, `output_output56`,
`output_output57`, and `output_output68` now compare exactly. Newly exposed
sibling cases retain their next honest dispositions: one reaches a late result
attribute error, one exposes a larger result-tree mismatch, and prohibited-DTD
sources remain rejected.

## Corpus movement

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,083 | 2,089 | +6 |
| Initialization failures | 1,052 | 1,046 | -6 |
| Executed successfully | 1,981 | 1,986 | +5 |
| Execution failures | 102 | 103 | +1 |
| Exact XML-semantic matches | 1,851 | 1,855 | +4 |
| XML comparison mismatches | 54 | 55 | +1 |

The measured exact compatibility lower bound is now
`1,855 / 3,173 = 58.46%`. This is local compatibility evidence against the
hash-verified, non-redistributed OASIS CD04 archive; it is not a broad
conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_comment_sequence_runs_text_producers_and_recovers_delimiters
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1036
./scripts/verify.ps1
```

