# ASP.NET Managed Cancellation and Diagnostic Parity

Date: 2026-08-26

## Question

Can an idiomatic managed `CancellationToken` request cooperative cancellation
through the isolated worker while retaining FastXSLT's structured failure, and
do representative direct-path diagnostics survive the process boundary without
category or identity drift?

## Method

The private ASP.NET 8 workbench added a `CancellationToken` overload over the
unpaused controlled-transform protocol. An already-signalled token uses the
pre-dispatch command; a later signal sends one identity-correlated cancellation
command. The adapter awaits the engine result in both cases, so a cancellation
winner remains `FXCT0001 / cancelled` and a result committed first remains a
successful completion.

A direct Rust test and the live isolated endpoint asserted the same code,
category, request identity, and detail fields for:

| Case                          | Code       | Category      | Request identity   |
| ----------------------------- | ---------- | ------------- | ------------------ |
| Empty request identity        | `FXWB0003` | `invalid`     | none               |
| Mismatched source closing tag | `FXXM0002` | `invalid`     | none               |
| Unsupported `xsl:message`     | `FXST1006` | `unsupported` | none               |
| Host cancellation             | `FXCT0001` | `cancelled`   | correlated request |

The unsupported diagnostic retained its stylesheet identity and byte span. The
malformed-source diagnostic retained its source identity. The live check then
executed a valid recovery request on the original worker.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 -OperationalExperiments -MeasurementRequests 100 -MeasurementRuns 1
```

## Observation

- The already-signalled managed token returned `FXCT0001 / cancelled` with the
  exact direct-path request identity and detail.
- The active 20,000-item managed-token attempt cancelled in this run. This is a
  race observation, not a promise; completion remains valid if committed first.
- The four-case isolated diagnostic matrix matched the direct Rust assertions.
- The valid recovery produced `<out>20000.00</out>` after managed cancellation.
- The invalid-identity and cancellation probes reused the same worker process.
- Initialization failures were transferred before a worker became eligible for
  use; their failed processes were disposed by the client bootstrap path.

## Interpretation

The experiment closes the basic managed-token ergonomics question for the
isolated candidate without selecting a public exception mapping. Preserving the
engine failure is useful evidence, but a future supported .NET surface must
still decide whether and how to project cancellation into .NET conventions
without losing code, category, request identity, or detail.

The matrix establishes direct-versus-isolated parity only for four
representative private diagnostics. It does not establish a stable diagnostic
catalog, unknown-code compatibility, disclosure policy, in-process parity,
deadlines, or hard termination.
