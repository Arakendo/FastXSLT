# OASIS XSLT 1.0 Exact Preserve and First-Text String Length

- Date: 2026-09-24
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0016
- Related decision: ADR-0012

## Pressure

After exact expanded-name stripping was admitted, fourteen unchanged cases
remained at `FXST1043`. Five used only exact expanded-name
`xsl:preserve-space` declarations and therefore required no source view under
the default-preserve policy. One of those cases then exposed the independent
XSLT 1.0 expression `string-length(string(text()))`.

## Implemented slice

Compilation now expands exact preserve NameTests in stylesheet static namespace
context and retains the declarations in immutable compiled policy. Explicit
preserve-all and exact-name preserve declarations remain distinguishable from
the absence of a declaration even though all three avoid source-view
construction. Multiple exact preserve declarations and included-module
declarations compose additively.

An included preserve declaration cannot silently disappear when combined with
a strip declaration. Mixed strip/preserve rules remain explicit `FXST1043`
until NameTest priority, declaration conflict recovery, and import precedence
are implemented.

The XSLT 1.0 value-expression compiler also admits
`string-length(string(PATH))` through a typed location path. Runtime selects the
first node in document order, visits its effective string value without an
intermediate string allocation, counts Unicode scalar values with charged work,
and returns zero for an empty selection. Modern-mode semantics are unchanged.

Focused controls cover:

- unprefixed and prefixed exact preserve names;
- duplicate-name elimination;
- explicit preserve-all retention;
- additive include composition;
- rejection of mixed include strip/preserve policy;
- retained-capacity accounting for preserve names; and
- first-text-node length across four preserved whitespace values.

## Corpus result

Five cases left `FXST1043`. Four Microsoft cases now expose the independent
global temporary-tree constructor boundary and receive no pass credit:

- `Microsoft/Whitespaces__91421#1`
- `Microsoft/Whitespaces__91424#1`
- `Microsoft/Whitespaces__91426#1`
- `Microsoft/Whitespaces__91427#1`

After the typed first-text length composition was added, the unchanged
`Microsoft/Whitespaces_WhitespaceStripTest1#1` case became exact.

The full sweep moved from:

- 2,268 to 2,269 initialized cases;
- 2,216 to 2,217 successfully executed cases; and
- 2,062 to 2,063 exact expected-result matches.

The exact lower bound is therefore **2,063 / 3,173 (65.02%)**. The remaining
`FXST1043` frontier contains nine cases requiring namespace wildcards, mixed
strip/preserve selection, or declaration/import precedence.

## Non-claims

This tranche does not admit namespace wildcards, mixed strip/preserve
resolution, import precedence, same-priority recovery, schema-aware whitespace,
CDATA lexical-origin compatibility, a general nested-function evaluator, or a
public source-view abstraction. It is OASIS compatibility evidence, not an
XSLT 1.0 conformance claim.

## Reproduction

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
