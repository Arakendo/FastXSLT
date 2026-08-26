# ASP.NET Isolated Persistent-Worker Baseline

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core on .NET 8 |
| Worker | Optimized `fastxslt-worker` Rust process |
| Workload | Native XSLT30 `for-004` at suite revision `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Command | `./scripts/verify-aspnet-workbench.ps1` |
| Claim | First isolated host-boundary baseline; no production ABI, concurrency, or application-performance claim |

## Lifecycle exercised

The ASP.NET process reads the pinned source and stylesheet into managed byte
arrays and closes the file handles before worker initialization. A persistent
child process receives the logical identities and bytes once through a bounded
length-prefixed binary protocol. Inside the worker, the experimental safe-Rust
facade:

1. admits both resources to a bounded in-memory snapshot;
2. compiles the stylesheet once;
3. prepares the source XDM once; and
4. executes and serializes later requests against the retained state.

Per-request protocol frames contain only an operation and logical request
identity. Responses repeat that identity and carry either the serialized result
or structured code, category, optional request identity, and detail fields.
Input and output frames are capped at 1 MiB. The facade is feature-gated,
documentation-hidden, and the crate remains unpublished; this is not a supported
Rust or managed API.

## Smoke result

The live local smoke check observed:

```text
GET  /health                 -> ready, isolated-persistent-worker, max in-flight 1
POST /transform/smoke-001    -> 200 application/xml
result                       -> <?xml version="1.0" encoding="UTF-8"?><out>36.02</out>
POST /measure?requests=1000  -> 36.6236–65.2054 ms
                              -> about 15,336–27,305 transforms/second
```

The range covers three local smoke runs. Each measurement is one warm sequential
loop inside one HTTP request. It includes managed/native process transport,
request/result framing, worker dispatch,
prepared execution, serialization, and result transfer. It excludes HTTP setup
per transform, concurrent requests, worker startup, resource transfer,
compilation, preparation, file import, cancellation, restart, and snapshot
replacement from the timed loop.

## Guarantee boundary

- The worker is a real process boundary and may eventually be terminated for a
  hard timeout or crash, but this checkpoint does not implement supervision or
  automatic replacement.
- The ASP.NET client deliberately permits one in-flight protocol operation.
  Concurrent HTTP calls queue at that gate; no concurrency throughput claim is
  made.
- Engine structural/work/result bounds are present, but there is no cooperative
  cancellation channel from ASP.NET to an executing worker yet.
- Only the isolated candidate is exercised. No comparison with an in-process
  native ABI, WASM, or other host mechanism exists.
- The pinned standards transform is useful semantic evidence but not a
  representative consumer workload.

AR-0002 remains Proposed and AR-0010 remains Incubating. The next evidence must
cover bounded worker concurrency, cancellation and hard termination distinctions,
worker failure/restart, snapshot replacement, diagnostics, and an in-process
comparison before any host boundary is accepted.
