# OASIS XSLT 1.0 Source-Valued Global Temporary Tree

- Date: 2026-09-24
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Exact preserve-only whitespace declarations exposed four unchanged Microsoft
cases whose global result-tree-fragment constructors contained a nested
`xsl:value-of`. The existing global tree form retained only static literal
nodes, even though the runtime already materialized each global temporary tree
inside the invocation.

## Implemented slice

An XSLT 1.0 global temporary-tree plan may now retain a typed source location
path as a dynamic text node inside an otherwise literal constructed tree. The
compiled program retains only stylesheet-derived names, namespaces,
attributes, text, and path plans. For each invocation, runtime evaluates the
path from that invocation's principal source, applies XSLT 1.0 first-node
string conversion, and materializes a distinct temporary tree under the
existing XDM-node and XPath work budgets.

Stylesheet-constructor whitespace follows inherited `xml:space`: whitespace-
only stylesheet text is omitted under `default` and retained under `preserve`.
This is separate from source-document `xsl:strip-space` and
`xsl:preserve-space` policy.

Focused controls prove:

- two concurrent invocations sharing one compiled program receive different
  source-derived values and independent temporary trees;
- literal and source-derived descendants retain document order;
- `xml:space="preserve"` and `xml:space="default"` select the expected
  constructor text.

Retained-capacity accounting includes the compiled location path, while no
invocation value enters compiled state.

## Corpus result

Nine unchanged cases leave the `FXST1015` initialization boundary and execute:

- `Microsoft/BVTs_bvt069#1`; and
- `Microsoft/Whitespaces__91421#1` through `91428#1`.

Three become exact:

- `Microsoft/BVTs_bvt069#1`;
- `Microsoft/Whitespaces__91426#1`; and
- `Microsoft/Whitespaces__91427#1`.

The other six remain visible expected-result mismatches rather than receiving
pass credit. `91421` through `91428` are all listed in the suite's doubt
metadata; this implementation does not weaken comparison or reinterpret those
expected results.

The full sweep moves from:

- 2,269 to 2,278 initialized cases;
- 2,217 to 2,226 successfully executed cases; and
- 2,063 to 2,066 exact expected-result matches.

The exact lower bound is therefore **2,066 / 3,173 (65.11%)**. Only three
`FXST1015` cases remain, involving other global instruction-sequence shapes.

## Non-claims

This tranche does not admit a general global sequence constructor, arbitrary
expressions inside global trees, dynamic literal attributes, comments or
processing instructions constructed by instructions, cross-invocation
temporary-tree retention, or a new source-access abstraction. It is OASIS
compatibility evidence, not an XSLT 1.0 conformance claim.

## Reproduction

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
