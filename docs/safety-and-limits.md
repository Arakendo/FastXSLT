# Safety and Limits

Read this page before evaluating FastXSLT as if it were a mature XSLT engine.
It is a concise status summary, not a replacement for the owning
[design](Specifications/FastXSLT%20Software%20Design%20Document.md),
[ADRs](ADR/), or [security policy](../SECURITY.md).

## Current maturity

FastXSLT is in M1 pre-stability work. The Rust workspace, verification gates,
bounded-resource experiment, private XML/XDM path, and test-only golden transform
run, but no public transform API or supported language surface exists. The
project has not selected its initial XSLT/XPath profile and makes no conformance,
compatibility, performance, production-readiness, or security-audit claim.

## Standards support

No XSLT or XPath edition is currently advertised as supported. A golden fixture
is a harness seed, not evidence that the engine implements the language. Future
support reports must name the standards edition, suite and revision, selection
policy, exclusions, environment, and result categories.

Unsupported behavior and invalid input must remain distinguishable. Passing a
selected case proves that case under its recorded conditions, not general
conformance.

## Resource access and files

The intended default boundary is host-controlled and memory-resident:

- a host adapter may read files, streams, uploads, databases, or other sources;
- it admits owned bytes into a bounded resource set and closes source handles;
- compilation and transformation consume a sealed snapshot;
- the engine does not reopen provenance paths or create hidden temporary,
  spill, memory-map, or persistent-cache files; and
- an unadmitted or denied resource fails explicitly rather than falling back to
  ambient disk or network access.

This design reduces retained handles and repeated file access. It does not
claim that the host's import or output publication avoids operating-system
security scanning.

## Memory and resource exhaustion

Keeping work in memory does not make memory free or unbounded. Entry count,
per-entry bytes, aggregate bytes, parse growth, recursion, sequence growth,
diagnostics, messages, output, and elapsed work all need explicit policies.
Their concrete types, defaults, and deterministic guarantees are not selected
yet.

Until those limits are implemented and tested, hostile or merely large inputs
must be assumed capable of exhausting process resources.

## Concurrency and reuse

Compile-once and transform-many is an intended product boundary. ADR-0005 fixes
one semantic: requests in a transform set are independent and have no execution
or completion-order guarantee; dependent stages belong to the host. Thread
safety, reentrancy, cancellation, concurrent invocation, resolver safety,
snapshot replacement, worker/in-flight limits, and failure collection remain
open. No type currently carries a public concurrency guarantee.

## Diagnostics and inspection

The design requires structured, source-located diagnostics. Stable error codes,
policy categories, and a read-only semantic inspection surface will be derived
from implemented vertical slices and real consumers. There is no stable error
catalog or inspection API today.

Future hosts should be able to distinguish reportable semantic findings from an
operation that failed before producing a trustworthy result, without parsing
human-readable strings or importing private engine types.

## Unsafe Rust

The workspace currently forbids first-party `unsafe` code. Tests alone cannot
authorize an exception. [ADR-0003](ADR/ADR-0003-unsafe-rust-exception-policy.md)
requires measured necessity, rejected safe alternatives, a written safety
contract, a small contained surface, focused verification, demonstrated value,
and removal criteria before any exception can be accepted.

## Performance

The name FastXSLT is an objective, not a benchmark result. A future performance
claim must name the workload, baseline processors, hardware and software,
correctness gate, preload and compilation policy, warm/cold state, host-boundary
cost, result transfer, retained memory, and peak memory.

Rust-only microbenchmarks do not establish ASP.NET application performance.

## API and compatibility

There is no public engine API, stable ABI, serialized compiled-artifact format,
or compatibility promise yet. FastXSLT is MIT licensed, but pre-stability API
and behavior may change as evidence shapes the first usable slice.
