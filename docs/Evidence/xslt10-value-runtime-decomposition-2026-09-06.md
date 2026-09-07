# XSLT 1.0 Value Runtime Decomposition

Date: 2026-09-06  
Status: Verified structural conservation evidence

## Pressure

The private value evaluator reached 2,017 lines after the XSLT 1.0 variable,
path string-function, substring, translate, and sum tranches. This crossed the
ADR-0004 review threshold and exposed one cohesive responsibility: XPath 1.0
compatibility conversions and their first-node node-set behavior.

## Extraction

The compatibility operations moved to the private typed module
`runtime/value_evaluator/xslt10_compatibility.rs`.

| Source unit | Before | After |
| --- | ---: | ---: |
| `runtime/value_evaluator.rs` | 2,017 | 1,764 |
| Private compatibility module | 0 | 266 |

The parent retains exhaustive value-expression dispatch and the general value
runtime. The child owns only XSLT 1.0 variable conversions/comparisons,
first-node path string conversion, bounded path string functions, path
substring/translate, and `sum(path)`.

No parser, compiler, XDM, result-tree, host-policy, public API, or alternative
runtime responsibility moved into the module. The dependency remains private
and one-way.

## Conservation

- The focused XSLT 1.0 runtime suite passes after extraction.
- Strict crate Clippy passes after extraction.
- The full OASIS checkpoint remains 1,041 exact matches with 77 mismatches and
  173 execution failures.
- Full workspace verification is recorded after this campaign tranche.
