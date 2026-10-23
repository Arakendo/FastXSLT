# OASIS Shared Context-Identity Attribute Value Template

- Date: 2026-09-25
- Status: Verified language-surface evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Two unchanged Microsoft key cases use zero-argument `generate-id()` in a
literal result attribute. FastXSLT already supported the same function in
ordinary value expressions and supported `generate-id(key(...))` in an
XSLT 1.0 attribute value template, but the simpler current-node AVT remained
at generic `FXST1031`.

## Implemented slice

The shared literal-attribute compiler now lowers one zero-argument
`generate-id()` expression, with optional static prefix and suffix text, to an
owned context-identity value. Runtime materialization uses the same stable
source-node identity function as ordinary value expressions, charges the XPath
operation, and constructs the attribute through the existing result tree.

This is a shared XSLT primitive rather than an XSLT 1.0 recovery behavior.
Source-node execution is admitted. A temporary-tree context remains explicitly
unsupported until its identity and lifetime behavior has focused evidence; it
is not silently mapped to a source identity.

## Verification and corpus disposition

Focused compiler and runtime tests prove static text composition and equality
with the ordinary `generate-id()` value-expression result for the same current
source node.

`Microsoft/Keys__91832#1` and `Microsoft/Keys__91833#1` leave `FXST1031`,
reducing that frontier from four cases to two. Both then reach the already
established XML security boundary because their shared principal source
contains a forbidden DTD. They move to invalid `FXXM0002`; the engine does not
relax external-entity or DTD authority to improve an archival count.

The full sweep therefore remains **2,319 initialized**, **2,271 successfully
executed**, and **2,105 / 3,173 exact matches (66.34%)**. The frontier movement
is useful language-surface evidence, but is not a pass and does not alter the
conserved totals.

## Reproduction

```powershell
cargo test -p fastxslt --all-features compiles_zero_argument_context_identity_inside_literal_text
cargo test -p fastxslt --all-features xslt10_generate_id_avt_uses_the_current_source_node_identity
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys__91832#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys__91833#1'
```
