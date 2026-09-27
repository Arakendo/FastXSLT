# XSLT30 call-template deep-recursion boundary

Date: 2026-09-25

## Result

The unchanged W3C XSLT30 case `call-template-1001` was probed but was not
admitted as a passing case. Compilation now reaches its actual execution
pressure: 500 non-tail recursive named-template calls overflow the native Rust
test-thread stack. The case remains under the denominator's visible default
disposition.

The probe did admit four reusable modern-expression primitives that were
previously confined to narrower paths:

- variable/integer `!=` uses the existing typed equality plan under logical
  negation;
- template arguments admit modern variable arithmetic with zero-or-one operand
  selection;
- modern `concat($variable, 'literal')` uses zero-or-one variable string
  conversion rather than XSLT 1.0 first-item conversion;
- `string-length(.)` can supply a named-template argument through the shared
  context-string traversal and work accounting.

A focused bounded-recursion regression executes these primitives together and
produces `1|2|3|3`. The complete unchanged 500-call case is intentionally not
used as that regression because increasing the test thread's stack would hide
the engine boundary rather than resolve it.

## Harness

The call-template corpus adapter now understands file-backed `assert-xml`
expectations in addition to inline assertions. That harness capability is
retained even though `call-template-1001` did not pass, so a future execution
stack design can compare the unchanged result directly with the pinned upstream
file.

## Boundary

This evidence does not select tail-call elimination, heap-backed invocation
frames, a larger native stack, or a public recursion guarantee. Deep non-tail
recursion remains a runtime representation question. The existing bounded
recursion limit remains necessary but does not by itself prevent native stack
exhaustion below that limit on all builds and hosts.

## Accounting

No corpus disposition changes. The complete 42-case `insn/call-template`
denominator remains at 29 passes, one profile exclusion, and 12 visible
defaults. Across conserved XSLT30 denominators, the total remains 500 passes,
12 engine-unsupported cases, 2,879 profile exclusions, and 108 visible defaults
out of 3,498 cases.

## Validation

- focused bounded modern named-template argument regression
- unchanged selected `insn/call-template` corpus execution
- strict crate Clippy after private responsibility extraction

