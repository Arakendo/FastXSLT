# Private Prepared-Input Reuse

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Revision under test | Working tree after `7f6d8a7` |
| Fixture | `corpus/golden/hello/input.xml` and `stylesheet.xsl` |
| Scope | Private AR-0009 explicit prepared-input lifecycle experiment |
| Public guarantee | None |

## Experiment shape

The experiment adds a mutable builder that receives one sealed resource
snapshot and explicit source identities. Preparing an identity parses its
admitted bytes and constructs an immutable engine-owned XDM document under a
caller-supplied preparation control. Sealing consumes the builder and produces
a read-only set of shared documents.

This is explicit preparation, not a hidden cache:

```text
sealed resource snapshot
        |
        +-- caller selects source identities
        |
        v
bounded preparation builder
        |
        v
sealed prepared-input set
        |
        +-- immutable document A
        `-- immutable document B
```

The set retains the snapshot generation that produced it. Generation equality
uses shared owner identity, not a content fingerprint. A separately sealed
snapshot containing equal bytes is a different generation.

## Results

Thirty-five tests pass. The focused experiment establishes:

- one prepared document allocation is reused by two independently compiled
  stylesheet programs;
- the prepared result matches the existing parse-per-invocation semantic and
  serialization reference;
- one compiled stylesheet executes over two prepared resources containing
  equal bytes;
- equal bytes under distinct logical resource identities produce distinct
  document allocations and retain different provenance;
- the 87-byte golden source currently constructs six XDM nodes for each logical
  document, including the document node and retained whitespace text;
- a prepared set recognizes its original snapshot generation and rejects a
  separately sealed equal-content generation as the same owner;
- dropping another reference to the original snapshot does not invalidate the
  prepared document because the sealed set retains its generation;
- missing and duplicate preparation requests fail explicitly; and
- preparation observes the same cooperative cancellation token and XML/XDM work
  limits as invocation parsing and construction.

## Ownership observations

The prepared document contains only source-derived XDM state. Parameters,
current context, variables, messages, cancellation, work counters, result
construction, and serialization remain invocation-local. Compiled stylesheet
state remains separate and may consume the same prepared document without
storing stylesheet-dependent indexes in it.

Logical identity and provenance prevent equal bytes from collapsing two source
documents. Physical sharing occurs only when callers ask for the same prepared
identity from the same sealed set.

## Limitations

The prepared set is not connected to the transform-set API experiment and is
not a public handle, cache, or default execution strategy. There is no eager
whole-snapshot preparation, lazy memoization, eviction, reconstruction,
single-flight construction, concurrent first access, failure memoization,
retry, stylesheet-derived index, cross-snapshot reuse, or global cache.

Node count is not retained-memory measurement. The experiment has not measured
the allocation capacity of strings/vectors, parser construction peak, elapsed
parse time, contention, `Send + Sync` use across actual workers, or end-to-end
ASP.NET benefit. Parse per invocation remains the semantic reference until those
costs and concurrency properties are established.
