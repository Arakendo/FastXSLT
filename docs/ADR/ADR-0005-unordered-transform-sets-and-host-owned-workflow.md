# ADR-0005: Unordered Transform Sets and Host-Owned Workflow

- Status: Accepted
- Date: 2026-08-25
- Related decisions: ADR-0002
- Related reviews: AR-0003
- Supersedes: None

## Context

FastXSLT is intended to execute large volumes of transformations while reusing
sealed resources, prepared source documents where justified, and compiled
stylesheets. A representative workload may admit thousands of documents and
execute them with bounded concurrency.

Some transformations are independent. Others form an application workflow: a
later transformation may require a document produced by an earlier one. Treating
submission order as that dependency would require an evolving resource universe,
implicit result publication, dependency failure propagation, intermediate-result
lifetime rules, and ordered scheduling. Those behaviors conflict with ADR-0002's
sealed-snapshot boundary and would turn the volume executor into a workflow
orchestrator.

The project owner selected a narrower initial responsibility: FastXSLT completes
independent work with scheduling freedom; the calling application sequences
dependent stages and explicitly admits prior results as later resources.

## Decision

A FastXSLT transform set contains only independently executable requests,
regardless of the eventual public Rust type name.

Within one transform set:

- submission, start, execution, and completion order have no semantic meaning;
- requests may execute concurrently and FastXSLT may reorder them;
- no request may depend on another request in the same set running first;
- one request's result is not implicitly visible to a sibling request;
- every request and result has explicit logical identity independent of a
  filename, destination path, worker, or completion position;
- every invocation observes only its admitted snapshot and explicit
  capabilities; and
- parameters, dynamic context, messages, diagnostics, cancellation state,
  transient values, and result construction remain invocation-local.

A result-presentation API may offer submission-order lookup as a convenience,
but that does not constrain scheduling and does not make submission order a
workflow mechanism.

If a later transformation depends on an earlier result, the host completes the
prerequisite stage, selects and identifies the result, explicitly admits it into
a later resource snapshot, and submits the dependent stage separately:

```text
stage 1 snapshot -> unordered transform set -> identified results
                                                   |
                                    host admits selected results
                                                   |
                                                   v
stage 2 snapshot -> unordered transform set -> identified results
```

Producing a result does not itself admit a resource. A logical result identity
such as `abc.html` does not authorize FastXSLT to write that filesystem path.
Output publication remains an explicit host responsibility under ADR-0002.

FastXSLT does not initially provide ordered transform sets, a transformation
DAG, dependency inference, or implicit result-to-resource promotion.

Worker count, queue depth, admitted input count, parsed-input retention,
in-flight limit, eviction, failure collection, cancellation, and scheduling
algorithm are explicit bounded policies to derive from implementation evidence.
Examples such as 5,000 inputs and 10 workers are workloads, not defaults or
guarantees. This decision permits threads, work stealing, or another measured
executor without selecting one.

## Ownership

FastXSLT owns:

- validation and sealing of independently executable transform requests;
- bounded scheduling and per-invocation isolation;
- stable request/result correlation independent of completion order;
- resource resolution against the snapshot and capabilities admitted for an
  invocation; and
- freedom to schedule for throughput or locality without changing semantics.

The host owns:

- dependency discovery and workflow stages;
- ordering between dependent transformations;
- selecting which results become later inputs;
- constructing and sealing later snapshots;
- publication, retry, rollback, and transaction behavior; and
- the meaning of host destinations such as paths, responses, databases, or
  object-store keys.

XSLT owns standards-defined resource-reference semantics within an invocation.
It does not infer cross-request dependencies or search sibling results.

## Consequences

### Positive

- Workers can pull any ready request without preserving submission order.
- Snapshot contents remain immutable and deterministic during execution.
- Result correlation survives randomized, concurrent, or locality-aware
  scheduling.
- A large independent batch does not pay for graph validation or dependency
  propagation.
- Application workflow remains visible in application code.

### Negative

- Multi-stage pipelines require explicit host orchestration and synchronization.
- Intermediate results must be retained and admitted into later snapshots.
- Applications may repeat orchestration patterns that a future graph facility
  could centralize.
- Separate stages may miss optimization opportunities across a dependency DAG.

## Alternatives considered

### Preserve submission order

Sequential submission order limits parallelism while still failing to define how
later requests observe earlier results. It also encourages callers to encode
dependencies in an incidental list position.

### Ordered execution over evolving shared resources

Implicit publication conflates scheduling, resource admission, output identity,
and authority. Sibling requests would observe timing-dependent resource state,
weakening sealed-snapshot determinism.

### Core transformation graph

A DAG could eventually coordinate dependencies, but it requires graph identity,
validation, failure propagation, cancellation, intermediate budgets, provenance,
and result-to-resource semantics without current consumer evidence.

### Host-owned workflow stages

The selected alternative preserves simple unordered execution and explicit
resource generations. A future graph can be reconsidered without changing the
meaning of existing transform sets.

## Validation

- Randomize request acquisition, start, and completion order while preserving
  stable result identity and meaning.
- Execute representative sets under multiple worker and in-flight limits.
- Prove a sibling result cannot be resolved unless the host admits it into a
  later snapshot.
- Prove shared prepared inputs and compiled stylesheets remain immutable while
  invocation state remains isolated.
- Execute one stylesheet over many sources and many stylesheets over one source.
- Execute a two-stage host workflow with explicit result admission between
  snapshots.
- Verify a batch of one is semantically equivalent to the convenience path.
- Measure end-to-end throughput, memory, queueing, and host-boundary costs before
  selecting executor defaults or scheduling constraints.

## Reopening triggers

Reconsider a graph facility if representative consumers require substantial
transformation DAGs, explicit staging creates a measured copying or snapshot
bottleneck, transactional graph publication becomes a product requirement, or
an engine-owned graph can demonstrate meaningful benefit while preserving
snapshot, authority, isolation, diagnostic, cancellation, and memory invariants.
