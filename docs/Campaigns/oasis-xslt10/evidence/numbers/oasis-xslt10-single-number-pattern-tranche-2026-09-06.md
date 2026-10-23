# OASIS XSLT 1.0 Single-Number Pattern Tranche

Date: 2026-09-06

## Question

Can single-level `xsl:number` use static `count` and `from` patterns through a
bounded shared traversal without prematurely admitting the complete XSLT
pattern language or `level="multiple|any"`?

## Implemented slice

- exact unprefixed and namespace-qualified element-name patterns;
- the `*` any-element pattern and `/` document pattern;
- ancestor-or-self search for the first node matching `count`;
- a `from` boundary that stops the search before an out-of-scope number is
  produced;
- preceding-sibling counting against the selected pattern;
- charged ancestor and sibling inspection;
- retained-capacity accounting for compiled expanded names.

Union, predicate, key/id, general node-kind, and variable-dependent patterns
remain unsupported, as do `level="multiple"` and `level="any"`.

## Focused verification

A first-party runtime test exercises `count="note"`, `from="chapter"`,
unrelated siblings, three numbered nodes, and composition with the existing
zero-padded punctuation format.

## Full local OASIS observation

Command:

```powershell
./scripts/measure-oasis-xslt10.ps1
```

Relative to the 871-pass formatting/context-value checkpoint:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,188 | 1,197 | +9 |
| Executed successfully | 980 | 989 | +9 |
| XML comparison pass | 871 | 880 | +9 |
| XML comparison mismatch | 75 | 75 | 0 |
| Execution failure | 208 | 208 | 0 |

The strict unchanged-result lower bound is now 880 of 2,742
standard-operation cases (32.09%), or 880 of all 3,173 catalog cases (27.73%).

## Interpretation

The result supplies the first explicit numbering-pattern traversal with a clean
nine-for-nine corpus movement. The representation is intentionally narrower
than the template-pattern system; broader sharing should be earned when unions,
predicates, or other pattern families require it rather than exposing template
selection internals prematurely.
