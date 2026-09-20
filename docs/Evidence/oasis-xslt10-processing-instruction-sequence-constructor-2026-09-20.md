# OASIS XSLT 1.0 Processing-Instruction Sequence Constructor

Date: 2026-09-20

## Question

Can XSLT 1.0 processing-instruction content reuse FastXSLT's ordinary
instruction engine while preserving the legacy text-only construction and
lexical-recovery rules, without widening the modern static context?

## Implementation

An XSLT 1.0 `xsl:processing-instruction` with a static valid target may retain
an ordinary compiled instruction sequence of at most 64 meaningful children.
It executes with the current invocation context, variables, work controls, and
cancellation through the reference instruction engine. Only top-level text
results contribute to PI data. Direct element, attribute, comment, copy, and
processing-instruction constructors are omitted before compilation so their
content and attribute-set expansion cannot leak into the PI value.

The XSLT 1.0 recovery rule replaces every `?>` in the completed data with
`? >`. This recovery applies to both static and dynamically constructed data.
The existing modern-context `FXST1034` and `FXST1035` boundaries remain.

The compiled body participates in named-template validation, decimal-format
binding, semantic inspection, and known-owned-capacity accounting. No PI-only
expression evaluator, public representation, or new resource authority was
introduced.

## Executable evidence

The focused regression combines literal text, an ignored literal-result
element, a source-dependent `xsl:for-each`/`xsl:value-of`, and a generated
`?>`. It produces exactly:

```xml
<out><?joined prefixonetwo? >suffix?></out>
```

The unchanged OASIS cases `Lotus/output_output69` and
`Lotus/output_output72` now compare exactly. Other exposed cases retain honest
next boundaries: prohibited-DTD source documents remain rejected, a nested
illegal `xsl:template` remains unsupported, HTML PI delimiter behavior remains
a serializer mismatch, and disable-output-escaping=`yes` remains outside the
semantic result-tree slice.

## Corpus movement

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,079 | 2,083 | +4 |
| Initialization failures | 1,056 | 1,052 | -4 |
| Executed successfully | 1,977 | 1,981 | +4 |
| Exact XML-semantic matches | 1,848 | 1,851 | +3 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The measured exact compatibility lower bound is now
`1,851 / 3,173 = 58.34%`. This is local compatibility evidence against the
hash-verified, non-redistributed OASIS CD04 archive; it is not a broad
conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_processing_instruction_sequence_runs_text_producers_and_recovers_delimiter
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1034
./scripts/verify.ps1
```

