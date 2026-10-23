# XSLT30 Corpus Test-Owner Decomposition Review

Date: 2026-08-29

## Trigger

Adding the explicit conflict-recovery variants raised
`golden_runtime_xslt30_tests.rs` to 1,117 physical lines. The file also owned
two independently executable corpus responsibilities: XSLT template dispatch
across the declaration and apply-templates test sets, and XPath path-expression
execution across the separate `expr/path` test set. It therefore crossed
ADR-0004's 1,000-line calibration threshold while satisfying a responsibility
trigger.

## Decision

Extract the complete path-corpus responsibility into the private
`xslt30_path_tests.rs` module. It owns:

- loading and interpreting the pinned path test-set metadata;
- admitting each source and stylesheet into a bounded sealed snapshot;
- executing the ten selected cases through the production runtime;
- comparing the asserted result element semantics; and
- conserving the complete ten-case path denominator.

Rename the remaining source to `xslt30_template_dispatch_tests.rs`. It owns the
template declaration and apply-templates corpus adapters and their template
selection, mode, priority, continuation, parameter, and diagnostic tests.

The former 1,117-line unit becomes an 866-line template-dispatch owner and a
342-line path-corpus owner. Both depend directly on the same private production
runtime boundary. Neither imports sibling internals, receives a broad mutable
context, or adds callbacks, traits, public APIs, or a crate boundary. Each keeps
its small metadata/XDM lookup scaffolding local so the independently executable
corpus adapters do not acquire an artificial shared test framework.

## Conservation

The extraction changes no engine, stylesheet, source, overlay, assertion, or
case disposition. Focused conservation runs retain:

- all 11 path-module tests: the complete inventory plus `path-001` through
  `path-010`; and
- all 33 template-dispatch tests, including the six-case template denominator,
  the 40 admitted apply-templates cases, structured error cases, and exact
  result assertions.

The full workspace gate additionally preserves formatting, strict Clippy,
unsafe-surface enforcement, all engine/native/worker tests, local Markdown
links, pinned corpus cleanliness and inventory, and generated documentation.
No hot-path indirection or allocation changes because only test ownership and
module registration moved. Compile-time pressure did not trigger this review,
so no build-time improvement is claimed.

## Disposition and reopening

The required review is discharged by private, one-way corpus owners. Reopen if
the template-dispatch unit crosses another ADR-0004 calibration threshold,
acquires another independent corpus family, or if common test scaffolding grows
enough that duplication becomes a responsibility rather than a few local
adapter functions.
