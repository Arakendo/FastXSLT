# Runtime Composition Cohesion Review

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Unit | `crates/fastxslt/src/runtime/golden_runtime_experiment.rs` |
| Line count | 1,039 at initial review; 901 after focused test extractions |
| Trigger | ADR-0004 1,001–2,000 inspection band during substantive modification |
| Disposition | Retain; reopen at named pressure |

## Responsibility inventory

Lines 1–517 contain the private reference runtime composition path:

- transform request/set construction and policy validation;
- stylesheet resource compilation entry;
- parse-per-invocation XML/XDM construction;
- instruction sequence execution and exact/built-in template dispatch;
- semantic result construction and work-control translation; and
- private structured operation failures.

Serialization is already a private child module. Lines 518 onward retain
focused semantic-composition tests for batch identity, resource authority,
limits, template dispatch, and result behavior. Phase-specific control tests,
host-owned workflow tests, and pinned-XSLT30 catalog adapter tests now live in
three named private sibling test modules.

The unit is modified by AR-0003, AR-0004, AR-0009, and AR-0010 evidence, which
supplies a responsibility trigger in addition to the size band. The reviews
meet here because this file is the composition owner; they do not create four
independent runtime subsystems inside it.

## Candidate decompositions

### Extract template dispatch now

Rejected at this checkpoint. The current helper is small and would require the
child module to consume the private program, XDM document/node, result-node,
execution-failure, invocation-control, sequence-execution, text-append, and
failure-translation internals. That reduces physical size without reducing
responsibility coupling.

### Move all tests to a child file

Deferred. It would lower the visible line count but the tests intentionally
exercise private composition types and functions. Moving them unchanged would
improve scrolling while preserving nearly total coupling, so it is not yet an
ADR-0004 decomposition win.

### Extract independently understandable test responsibilities

Selected when later work supplied named seams. Phase-control fault injection
moved to `golden_runtime_control_tests.rs`; the host-staging golden moved to
`golden_runtime_workflow_tests.rs`; and pinned-suite catalog navigation moved
to `golden_runtime_xslt30_tests.rs`. Each owns distinct fixtures and change
pressure while calling the same private semantic reference path. The main unit
fell from 1,120 lines at workflow-review pressure to 901 lines.

### Retain the composition owner

Selected. The production portion remains 517 lines, has one direction from
admitted resources and compiled semantics into invocation-local execution, and
does not contain host I/O, ASP.NET/FFI, corpus-ledger, persistent cache, or
alternate-backend mechanics.

## Conservation and consequences

At the latest disposition, all-feature verification covers 57 tests, Clippy
with warnings denied, formatting, documentation, Markdown links, submodule
integrity, and the 46,421-case inventory. No public/API/ABI, dependency, unsafe,
filesystem-authority, hot-path call, or serialization ownership change results
from the test-only extractions.

Keeping direct calls avoids adding hot-path indirection. The cost is navigation
pressure from a large colocated test body; the named triggers below prevent
that convenience from becoming an indefinite exception.

## Reopening triggers

Reopen and prefer a private semantic extraction when any occurs:

- the file reaches 1,200 physical lines or production code reaches 700;
- template selection gains priorities, modes, built-in mode variants, or a
  pattern index with a narrower input/output contract;
- messages, diagnostics collection, or another result channel enters sequence
  execution;
- a prepared-input or dispatched path duplicates the core execution loop;
- tests split into a second independently understandable responsibility; or
- ordinary changes repeatedly touch distant unrelated regions.
