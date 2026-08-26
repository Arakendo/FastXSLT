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
- `POST /transform/dotnet-xslt1`
- `POST /measure/dotnet-xslt1?requests=1000`
- `POST /transform/saxoncs`
- `POST /measure/saxoncs?requests=1000`

The Microsoft comparison first verifies that `XslCompiledTransform` cannot
execute the exact XSLT 2.0 `for-004` stylesheet. Its timed path uses a reviewed XSLT 1.0
equivalent over the same prepared XML and produces the same exact serialized
result. Both stylesheets compile once and both timed loops materialize each
serialized result. This is an equivalent-workload comparison, not a claim that
the language surfaces are interchangeable.

An optional local SaxonCS lane can be supplied from the gitignored
`.workbench/saxoncs-comparison/` area and enabled with `-LocalSaxonCs`. The
measured local overlay uses `SaxonCS-HE` 13.0.0 in-process and runs the exact
XSLT 2.0 stylesheet over one prepared Saxon tree and one compiled executable. A
fresh transformer and serializer are created per invocation. Neither the
package, its adapter, nor its lock file is distributed by FastXSLT.

The transport has one explicit in-flight slot. Cancellation, a worker pool,
restart, snapshot replacement, and an in-process FastXSLT comparison remain
future experiments.
