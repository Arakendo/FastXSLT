# OASIS XSLT 1.0 Grouped Document Variable Scalars

The 2026-10-07 continuation admits transparent grouped variable reads and
scalar conversion of existing sealed document-root bindings. Exact OASIS CD04
coverage remains 2,500 / 3,173 (78.79%). The unchanged archival root-node case
gets past compilation but still fails on its historical UNC reference, so it
earns no exact credit.

## Shared scalar semantics

The private grouped-selection compiler recognizes one complete `($variable)`
group, including whitespace and an already supported qualified variable name.
It lowers directly to the existing XSLT 1.0 variable-string plan. Operators,
path suffixes, predicates, nested groups and mixed unions retain their own
eligibility checks. No recursive general-expression compiler is introduced.

The shared legacy conversion now recognizes existing sealed document-root
values after higher-priority atomic and retained-source bindings. It applies
the existing invocation-owned effective whitespace view and traverses the
document root through charged XDM string-value evaluation. The owning document,
not a coincident local ID in the active source, supplies the value. A local
atomic binding shadows the global document; aliases use existing COW maps.
No document is reopened, mutated or newly admitted by conversion. URI resolution,
denial, budgets, cancellation and invocation teardown retain their existing owners.

The private document-scalar owner and grouped-selection owner contain the
implementation. The 3,520-line value compiler adds only typed routing; its
19-file subject directory and existing decomposition debt remain visible under
ADR-0004. This narrow change does not discharge that debt or justify growing
new evaluation families in the composition root. No public type, owner table,
cross-document ordering, namespace frame, unsafe code or host policy changes.

## Executable controls

Compiled tests distinguish principal and secondary document text, verify alias
reads and cross-kind shadowing, exercise both principal-source whitespace
representations and complete atomic/sequence frame clones, and compare four
concurrent invocations. Secondary scalar conversion itself uses the existing
visibility-view path. Exact and one-less XDM-node, string-traversal and result-byte
budgets, cancellation, denial before resolution, unbound variables and recovery
are pinned. A compiler control proves full-group eligibility and preserves
non-admission of unrelated grouped syntax. These authored fixtures earn no
corpus credit.

## Corpus accounting

Run `scripts/measure-oasis-xslt10.ps1 -TraceCase UseDocumentFnToReturnRootNode`
against unchanged OASIS CD04 archive SHA256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
`Microsoft/Variables_UseDocumentFnToReturnRootNode#1` progresses from expression
compilation failure to `FXRS0005 / InvalidReference` on the stylesheet's original
backslash UNC spelling. No UNC path is opened or rewritten, and no catalog,
expected result, exclusion or resolver mapping is changed.

The conserved replay reports 2,701 initialized, 469 initialization failures,
2,652 successful executions, 49 execution failures, 2,500 exact comparisons,
14 mismatches, four comparator gaps and 423 / 431 expected-error credits.
The direct-DTD attribution total remains 134. AR-0029 remains deferred, and
another 39 exact cases are still required for 80%.

The three new controls pass through full workspace verification: formatting,
strict Clippy, 1,663 core tests (60 manual probes ignored), 22 native tests
(three ignored), four WASM tests, 18 worker tests, documentation, the reviewed
unsafe surface and conformance inventories. Final local-link verification
checks 1,985 Markdown files. The official all-features
`wasm32-unknown-unknown` check and `git diff --check` pass.
No performance or general conformance claim follows from this continuation.
