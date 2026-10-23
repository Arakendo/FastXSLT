# OASIS XSLT 1.0 local context-name variables -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can local variables bind the lexical QName returned by `name()` or `name(.)`
using the existing typed atomic-variable machinery, without treating a
context-dependent expression as a static value?

## Implemented slice

The compiler retains context-name bindings as a dedicated private instruction.
At execution, the instruction charges one XPath operation and one node visit,
obtains the current node's lexical name, preserves a source node's retained
prefix, and binds the result as the existing string atomic value. A missing
context and an atomic context remain typed dynamic errors.

The binding dispatcher delegates context-derived bindings to focused private
helpers, keeping its central dispatch below the enforced source-unit limit.
Retention accounting, semantic inspection, and stylesheet validation recognize
the new private instruction without exposing representation through the public
facade.

## Corpus result

Three cases leave `FXXP1008`, each with a conserved later disposition:

| Case | New disposition |
| --- | --- |
| `Lotus/namedtemplate_namedtemplate03` | Later unsupported template-argument expression `name(.)`, `FXXP1011` |
| `Microsoft/Miscellaneous__84425` | Later unsupported predicate `descendant::*[position() < $pos and name() = $name]`, `FXXP1001` |
| `Microsoft/BVTs_bvt093` | Executes with correct lexical names but remains a visible whitespace-result mismatch |

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,696 | 1,697 | +1 |
| Initialization failures | 1,439 | 1,438 | -1 |
| Executed successfully | 1,513 | 1,514 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,381 | 1,381 | 0 |
| XML comparison mismatches | 105 | 106 | +1 |
| `FXXP1008` initialization frontier | 13 | 10 | -3 |

The exact compatibility lower bound remains
`1,381 / 2,742 = 50.36%` for standard-operation cases and
`1,381 / 3,173 = 43.52%` for the complete catalog. Frontier movement is not
credited as a pass.

## Boundaries

This tranche does not generalize local variables to arbitrary functions or
node expressions. It does not add context-name expressions to template
arguments, admit the later predicate grammar, or change stylesheet whitespace
handling. The `BVT093` mismatch is therefore retained for independent
whitespace-semantics work rather than hidden behind the correct name values.

Temporary element and attribute nodes currently provide their retained local
name because their prepared representation does not retain a distinguished
lexical prefix. No public prefix-stability promise or cross-generation name
interning is inferred.

## Verification

- A focused runtime test binds `name(.)` while processing a prefixed source
  element and verifies the retained lexical `p:doc` value.
- The complete 3,173-case measurement accounts for all three frontier transfers
  and preserves the mismatch and exact-result denominators honestly.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
