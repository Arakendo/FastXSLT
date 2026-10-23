# OASIS XSLT 1.0 Double-Precision Template Priority

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged `Microsoft/ConflictResolution__77622#1` declares competing template
rules whose explicit priorities differ lexically beyond IEEE-754 binary64
precision. XSLT 1.0 priorities inhabit the XPath 1.0 number domain, so each
pair compares equal after conversion and the later declaration must win by
stylesheet order.

FastXSLT retained explicit priorities as bounded exact decimals for the modern
engine. Applying that modern representation directly to an XSLT 1.0
stylesheet incorrectly distinguished the priorities and selected the earlier
green-producing rules.

## Implemented slice

Explicit priorities compiled under XSLT 1.0 compatibility are now converted
through finite `f64` before entering the existing ordered private priority
representation. The modern path continues to retain bounded exact decimals.
Non-finite or otherwise unrepresentable compatibility values remain an
explicit unsupported boundary rather than entering template dispatch.

A focused compiler test proves that both the very large integer pair and the
near-identical fractional pair collapse to equal XSLT 1.0 priorities while the
same modern decimal spellings remain distinct. A focused runtime test proves
that source order resolves the resulting XSLT 1.0 tie.

## Corpus result

`Microsoft/ConflictResolution__77622#1` now initializes, executes, and compares
exactly. The complete sweep retains 2,293 initialized and 2,241 successfully
executed cases while:

- exact expected-result matches rise from 2,082 to 2,083;
- XML comparison mismatches fall from 76 to 75; and
- initialization and execution failures remain 842 and 52.

The exact compatibility lower bound is therefore **2,083 / 3,173 (65.65%)**.

## Reproduction

```powershell
cargo test -p fastxslt --all-features retains_bounded_exact_template_priority_and_classifies_other_lexicals
cargo test -p fastxslt --all-features xslt10_template_priority_uses_double_precision_before_source_order
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/ConflictResolution__77622#1'
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
