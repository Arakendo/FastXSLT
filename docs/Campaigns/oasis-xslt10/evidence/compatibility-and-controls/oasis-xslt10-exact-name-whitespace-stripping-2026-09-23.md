# OASIS XSLT 1.0 Exact-Name Whitespace Stripping

- Date: 2026-09-23
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0016
- Related decision: ADR-0012

## Pressure

After inherited source `xml:space` was admitted for the exact strip-all policy,
31 unchanged cases still stopped at `FXST1043`. Seventeen used one or more
exact element QNames in `xsl:strip-space/@elements`; the remainder required
`xsl:preserve-space`, namespace wildcards, or declaration conflict and
precedence semantics.

## Implemented slice

Compilation now expands exact whitespace NameTests in stylesheet static
namespace context and retains their expanded names in immutable compiled
policy. Multiple exact strip declarations and included-module declarations
compose additively. Namespace wildcards and selective preserve declarations
remain explicit `FXST1043` boundaries.

At execution, the accepted invocation-owned visibility view removes a
whitespace-only text child only when its parent has a retained expanded name.
The complete safe derived-document implementation applies the same predicate
and remains the differential oracle. Both forms continue to derive inherited
source `xml:space`; `preserve` protects a matching subtree and `default`
restores the compiled strip policy. Prepared XDM remains immutable and visible
nodes retain their original identities and provenance.

Focused controls cover:

- unprefixed and prefixed exact expanded names;
- duplicate declaration tokens;
- nonmatching element preservation;
- inherited `xml:space="preserve"`;
- complete-reference/view result parity;
- additive include composition; and
- retained-capacity accounting for compiled expanded names.

## Corpus result

The unchanged full sweep moved from:

- 2,260 to 2,268 initialized cases;
- 2,208 to 2,216 successfully executed cases;
- 2,054 to 2,062 exact expected-result matches; and
- 31 to 14 cases at `FXST1043`.

The exact lower bound is therefore **2,062 / 3,173 (64.99%)**.

Eight unchanged cases became exact:

- `Lotus/conflictres_conflictres24#1`
- `Lotus/match_match01#1`
- `Lotus/position_position70#1`
- `Lotus/whitespace_whitespace01#1`
- `Lotus/whitespace_whitespace02#1`
- `Lotus/whitespace_whitespace05#1`
- `Lotus/whitespace_whitespace06#1`
- `Lotus/whitespace_whitespace12#1`

Nine other cases left the whitespace frontier and exposed independent ID,
path, match-pattern, or global temporary-tree limitations. They receive no
pass credit.

## Non-claims

This tranche does not admit `prefix:*`, selective `xsl:preserve-space`,
same-name conflict resolution, import-precedence selection, schema-aware
whitespace, CDATA lexical-origin compatibility, a retained view cache, or a
public navigation abstraction. It is OASIS compatibility evidence, not an
XSLT 1.0 conformance claim.

## Reproduction

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
