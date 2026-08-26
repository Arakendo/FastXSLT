# ASP.NET XSLT Engine Comparison

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core targeting .NET 8 |
| FastXSLT base | `2da565a9b55197ed4f570e31f38f113d68b0a131` plus this comparison harness |
| Workload | XSLT30 `for-004` at suite revision `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Source | `for03.xml`, 216 bytes, prepared once per engine |
| Runs | Five warm sequential loops of 1,000 transformations |
| Command | `./scripts/verify-aspnet-workbench.ps1 -LocalSaxonCs -MeasurementRuns 5` |
| Claim | Local comparative evidence only; not a product benchmark or standards ranking |

## Compared paths

FastXSLT used the exact 377-byte XSLT 2.0 `for-004` stylesheet through the
persistent isolated worker. The source bytes crossed the process boundary once;
the worker retained one compiled stylesheet and one prepared XDM document. Each
timed invocation included request/result framing, process transport, execution,
serialization, and transfer of the serialized result.

SaxonCS-HE 13.0.0 ran in-process using the same exact stylesheet, one compiled
`XsltExecutable`, and one prepared Saxon `XdmNode`. Each timed invocation created
a fresh transformer, serializer, and result buffer. The Saxon adapter and
package were supplied from the gitignored `.workbench/saxoncs-comparison/`
overlay and are not distributed by FastXSLT.

Microsoft `XslCompiledTransform` could parse the version-2.0 stylesheet under
forward-compatible handling but could not execute its XPath 2.0 `for`
expression; execution reported `Expected token ')', found '$'.` Its timed lane
therefore used a reviewed 929-byte XSLT 1.0 equivalent. That stylesheet traverses
the same five `order-item` elements, multiplies the same `price` and `qty`
attributes, formats the same total, reuses one `XPathDocument` and compiled
transform, and materializes equivalent serialized XML. It is equivalent work,
not the same expression or language surface.

FastXSLT and SaxonCS produced:

```xml
<?xml version="1.0" encoding="UTF-8"?><out>36.02</out>
```

Microsoft produced the semantically equivalent serialization
`<?xml version="1.0" encoding="utf-8"?><out>36.02</out>`. The original
PowerShell verifier used a case-insensitive comparison and therefore did not
expose that byte difference; the verifier now uses case-sensitive expectations
for each lane.

## Five-run observations

| Engine path | Minimum transforms/s | Median transforms/s | Maximum transforms/s | Median per-run ratio to FastXSLT |
| --- | ---: | ---: | ---: | ---: |
| FastXSLT isolated worker, exact XSLT 2.0 | 15,396 | 23,994 | 30,085 | 1.00× |
| SaxonCS-HE in-process, exact XSLT 2.0 | 15,701 | 28,297 | 30,133 | 1.03× |
| Microsoft in-process, equivalent XSLT 1.0 | 98,956 | 108,612 | 113,220 | 4.12× |

FastXSLT and SaxonCS were close on this very small exact workload: their
per-run SaxonCS/FastXSLT ratios ranged from 0.94× to 1.34×. The large shared
slowdown in the fifth run also shows why absolute microbenchmark values from
this workstation should not be promoted into a stable claim.

The Microsoft path was consistently faster here, but the result does not show
that it can replace either modern engine: it could not execute the original
stylesheet and measured a structurally different XSLT 1.0 recursion. It is a
useful lower-complexity .NET baseline, not an XSLT 2.0 conformance comparison.

## Licensing and reproducibility boundary

Saxonica's current Saxon 13 documentation describes Home Edition as MPL-2.0,
states that SaxonCS-HE is now available, and says HE needs no license key.
However, the restored `SaxonCS-HE` 13.0.0 NuGet archive identifies its bundled
`LICENSE.txt` as the package license, and that file describes SaxonCS as
proprietary, key-activated software. The archive SHA-256 observed locally was
`1201669A4CA90843038CB3622FEB15CB8A798434CB64FD72B4BD2E433D130ACB`.

Because the issued payload and public product documentation disagree, FastXSLT
does not admit, restore, vendor, or redistribute SaxonCS. The ordinary ASP.NET
workbench build contains no SaxonCS package dependency. The ignored local
overlay is retained only for maintainer comparison pending licensing clarity.

Relevant upstream pages:

- [Saxonica .NET downloads](https://www.saxonica.com/html/download/dotnet.html)
- [Saxon 13 product overview](https://www.saxonica.com/html/documentation13/about/index.html)
- [Saxon 13 license-key guidance](https://www.saxonica.com/html/documentation13/about/license/licensekey.html)

## Limits

- Each engine ran sequentially with one explicit in-flight measurement loop.
- The engine order was FastXSLT, Microsoft, then SaxonCS in every run; order was
  not randomized.
- The timed loops excluded HTTP setup per transform, import, preparation,
  compilation, worker startup, and cold-start behavior.
- FastXSLT paid process transport; both comparison engines ran in-process.
- The workload is tiny, standards-derived, and not representative of an ASP.NET
  application's stylesheet mix, document sizes, concurrency, or memory policy.
- No allocation, retained-memory, cancellation, failure, or diagnostic-parity
  comparison was measured.

This evidence does not select an interop mechanism or establish a performance
guarantee. AR-0002 remains Proposed.
