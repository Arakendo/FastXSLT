# OASIS XSLT 1.0 Forward-Compatible Message

- Date: 2026-09-29
- Status: Verified implementation and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

`Lotus/ver_ver01#1` declares stylesheet version `8.5`, statically selects the
lower-version branch through `system-property('xsl:version')`, and emits a
non-terminating `xsl:message` before completing the principal result. The
engine already owned the required invocation-local message construction and
observation behavior for version `1.0`, but compilation rejected the same
common instruction subset solely because the declared version was newer.

## Implemented slice

The compiler now admits the existing bounded `xsl:message` subset for declared
versions `1.0`, `2.0`, `3.0`, and forward-compatible future versions:

- sequence-constructor message content;
- absent or static `terminate="no"`;
- static `terminate="yes"`, retaining the existing structured `XTMM9000`
  termination behavior; and
- invocation-owned observations that never enter the principal result tree.

This is one shared instruction implementation, not a second compatibility
backend. The focused compiler test covers versions `1.0`, `2.0`, `3.0`, and
`8.5` with each admitted termination spelling.

No broader modern `xsl:message` surface is inferred. Dynamic attributes,
`select`, `error-code`, public message sinks, adapter delivery, and ambient
logging remain outside this tranche.

## Corpus result

The unchanged `Lotus/ver_ver01#1` case now initializes, executes, and compares
exactly. The complete conserved sweep reports:

- 3,173 catalog cases;
- 2,442 initialized cases;
- 2,393 successful executions;
- **2,262 exact matches (71.29%)**;
- 728 initialization failures;
- 49 execution failures;
- zero visible comparison mismatches; and
- zero comparator-unsupported cases.

Expected-error credit remains independently conserved at 423 / 431. This is
compatibility evidence from a locally acquired archival corpus, not an XSLT
1.0 conformance claim.

## Verification

```powershell
cargo test -p fastxslt --all-features compiles_common_message_subset_after_validating_its_attributes
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/ver_ver01#1'
./scripts/verify.ps1
```

