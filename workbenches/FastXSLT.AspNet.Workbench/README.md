# FastXSLT ASP.NET Workbench

This unsupported workbench exercises FastXSLT through a persistent isolated
Rust worker. It is evidence for AR-0002, not a production package or accepted
interop API.

Run the reproducible smoke and sequential measurement:

```powershell
./scripts/verify-aspnet-workbench.ps1
```

The workbench currently loads the pinned XSLT30 `for-004` source and stylesheet
once, retains one compiled stylesheet and prepared input in the worker, and
offers:

- `GET /health`
- `POST /transform/{requestId}`
- `POST /measure?requests=1000`
- `POST /transform/inprocess/{requestId}`
- `POST /measure/inprocess?requests=1000`
- `POST /transform/dotnet-xslt1`
- `POST /measure/dotnet-xslt1?requests=1000`
- `POST /benchmark/tiers?requests=250&concurrency=4`
- `POST /experiment/worker-recovery`
- `POST /experiment/cooperative-cancellation`
- `POST /experiment/active-cancellation`
- `POST /experiment/natural-cancellation-races`
- `POST /experiment/managed-cancellation`
- `POST /experiment/diagnostic-parity`
- `POST /experiment/instruction-budget`
- `POST /experiment/native-boundary`
- `POST /experiment/generation-replacement`
- `POST /experiment/host-file-replacement`
- `POST /transform/saxoncs`
- `POST /measure/saxoncs?requests=1000`

The Microsoft comparison first verifies that `XslCompiledTransform` cannot
execute the exact XSLT 2.0 `for-004` stylesheet. Its timed path uses a reviewed
XSLT 1.0 equivalent over the same prepared XML and produces equivalent
serialized XML.
Microsoft emits the encoding label as lowercase `utf-8`; FastXSLT and the local
SaxonCS lane emit `UTF-8`. Both stylesheets compile once and both timed loops
materialize each serialized result. This is an equivalent-workload comparison,
not a claim that the language surfaces are interchangeable.

An optional local SaxonCS lane can be supplied from the gitignored
`.workbench/saxoncs-comparison/` area and enabled with `-LocalSaxonCs`. The
measured local overlay uses `SaxonCS-HE` 13.0.0 in-process and runs the exact
XSLT 2.0 stylesheet over one prepared Saxon tree and one compiled executable. A
fresh transformer and serializer are created per invocation. Neither the
package, its adapter, nor its lock file is distributed by FastXSLT.

The primary smoke transport retains one explicit in-flight slot. The tiered
experiment adds a bounded pool. An opt-in operational experiment forcibly
terminates one process, gives its next request a structured operational failure
without retry, replaces only that slot from the same sealed generation, and
proves a sibling plus a later request still complete. A second experiment
atomically promotes a new explicitly identified generation while an acquired old
generation remains executable until its lease drains. These are workbench
lifecycle observations, not a production restart policy or public API.

The pre-dispatch cooperative-cancellation probe carries a cancellation state
that was already signalled by the host into a normal engine invocation. The
engine observes it at its first owned charge point, returns
`FXCT0001 / cancelled`, and the same process and prepared state execute a later
request. It neither kills nor replaces the worker.

The active-cancellation probe uses a correlated start acknowledgement and a
separate cancel command while a 500-item transform is running. The worker's
reader remains responsive while execution owns another thread, and the engine
observes the shared signal at a local charge point. If completion is committed
before the supervisor processes the matching signal, completion wins; otherwise
the invocation returns structured cancellation. Signals for another logical
identity do not affect the active invocation. This is still one active request
per worker, not general multi-invocation multiplexing or a deadline guarantee.
The deterministic probe pauses at its first real charge point until the control
message arrives. Its signal-to-observation timing therefore measures the
experimental host/worker route, not natural cancellation-check latency.

The natural-race endpoint removes that barrier and immediately cancels 25
started transforms over 20,000 items. It reports cancellation and completion
counts plus observed cancellation-response latency, conserving all trials. A
valid completion remains a permitted race outcome. The larger input is admitted
because the workbench's explicit XML-event limit is now supplied to prepared
input parsing instead of being shadowed by its earlier 1,024-event default.

The managed-cancellation endpoint adapts a .NET `CancellationToken` to that
same cooperative protocol. It preserves FastXSLT's structured cancellation
failure rather than selecting a public .NET exception policy. An active signal
still races with committed completion and does not imply hard termination.

The diagnostic-parity endpoint checks invalid request identity, malformed XML,
unsupported XSLT, and cancellation against the same fields asserted by the
direct Rust path, then proves the original worker can execute a valid request.
This is a four-case private matrix, not a stable error catalog.

The instruction-budget endpoint gives one invocation a zero XSLT-instruction
budget. It asserts `FXCT0002 / limit`, no retry or process replacement, and a
successful ordinary transform on the same compiled/prepared worker afterward.
The command is a narrow workbench probe, not a proposed public policy format.

The in-process endpoints load the unpublished ADR-0008 native library. Its
version-zero ABI copies bounded buffers, retains Rust state behind numeric
handles, transfers structured failure envelopes, and uses a managed `SafeHandle`
wrapper. The native operational probe covers diagnostics, independent-handle
concurrency, double disposal, and use-after-dispose rejection. ADR-0009 also
carries already-signalled cooperative cancellation and an invocation-local
XSLT-instruction budget as scalar values. It has no active mid-execution signal,
same-handle concurrency, or hard-termination guarantee.

The host-file variant imports source and stylesheet files into owned bytes,
closes the handles, renames and removes both originals while the old worker
generation remains live, writes changed source bytes at the same host path, and
promotes the newly imported generation. The old lease retains its old result;
new requests observe only the new generation. The scratch files live under the
gitignored `.workbench/` directory and are removed after the experiment.

Run those checks with:

```powershell
./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments
```

Deadlines, crash-loop policy, production pool lifecycle, public managed error
mapping, and broader native/isolated lifecycle parity remain future work. Hard
worker termination is intentionally not described as cooperative cancellation.

The opt-in tiered benchmark generates deterministic 5-, 50-, and 500-item
sources. It measures sequential and bounded-concurrent warm execution with
per-invocation p50/p95/p99 latency, aggregate throughput, approximate managed
allocation, process CPU, working-set observations, source size, and result size.
Run it with:

```powershell
./scripts/verify-aspnet-workbench.ps1 -TieredBenchmark -LocalSaxonCs
```

The tiered benchmark includes both a bounded pool of isolated workers and a
bounded pool of independent native engine handles. Each owns its own compiled
stylesheet and prepared source. The native pool deliberately does not imply
same-handle concurrency, which is outside the version-zero ABI contract.
