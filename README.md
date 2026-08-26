# FastXSLT

FastXSLT is a new Rust-native XSLT engine for use inside other applications.
Its motivating consumers include ASP.NET applications that need reusable,
high-throughput transformations without treating an external command-line tool
as the primary product boundary.

The repository is currently in **M1 pre-stability development**. A test-only
private slice executes the `hello` golden transform through in-memory resources,
XML, owned XDM, XSLT/XPath compilation, runtime, semantic result, and separate
serialization. There is no public transform API or supported standards claim;
the XSLT/XPath profile remains under
[AR-0001](docs/Architectural%20Reviews/AR-0001-initial-standards-profile.md).

FastXSLT is not production ready and has not been security audited. Read
[Safety and Limits](docs/safety-and-limits.md) before evaluating it for an
application, and use the [Security Policy](SECURITY.md) for the present threat
scope and vulnerability-reporting route.

## Getting started

FastXSLT declares Rust 1.85 as its minimum supported Rust version and uses the
current stable toolchain for development, including `rustfmt` and Clippy. The
checked-in toolchain file configures the development toolchain automatically
through rustup, while CI also checks the minimum version.

Initialize the pinned W3C conformance suites after cloning:

```powershell
git submodule update --init --recursive
```

```powershell
cargo test --workspace
cargo lint
cargo docs
```

Run every local gate with:

```powershell
./scripts/verify.ps1
```

The full gate verifies that the QT3 and XSLT 3.0 submodules are present, clean,
and at the revisions recorded in
[the W3C suite provenance record](docs/Corpus/w3c-test-suites.md). Suite
availability does not establish FastXSLT conformance.

## Repository layout

```text
crates/fastxslt/  Public library crate and initially private engine layers
corpus/           Small, reviewed transform cases that drive implementation
vendor/           Immutable upstream conformance-suite submodules
docs/             Specifications, decisions, reviews, plans, and evidence
scripts/          Repeatable development and verification commands
```

FastXSLT begins as a modular monolith. XML, XDM, XPath, stylesheet compilation,
and execution have distinct logical ownership, but remain in one crate until
real dependency or release pressure justifies extraction. The rationale is in
[ADR-0001](docs/ADR/ADR-0001-evidence-led-modular-monolith.md).

The Rust API and future host adapters must preserve one engine contract. The
ASP.NET integration mechanism remains under review in
[AR-0002](docs/Architectural%20Reviews/AR-0002-aspnet-host-integration.md).

For volume workloads, FastXSLT is expected to admit resources into a bounded
in-memory snapshot, compile reusable stylesheets from that snapshot, and execute
sets of transformations without reopening files for every call. The snapshot
and batch boundary is under review in
[AR-0003](docs/Architectural%20Reviews/AR-0003-memory-resource-snapshots-and-batch-transforms.md).

[ADR-0002](docs/ADR/ADR-0002-memory-resident-execution.md) makes memory-resident
execution the default product boundary: host adapters may read files into owned
bytes, but the engine does not retain file handles, reopen source paths, or use
implicit temporary/spill/cache files during compilation and transformation.

## Documentation authority

The [documentation index](docs/README.md) explains which records are binding:

- specifications describe current intended contracts;
- ADRs record accepted architectural decisions;
- Architectural Reviews preserve open questions and evidence;
- plans sequence work but do not change architecture;
- change requests preserve consumer needs without granting them architectural
  authority;
- evidence records observations but does not create guarantees.

## Project status

The [roadmap](docs/Plans/roadmap.md) defines milestone outcomes. The private
architecture slice, bounded resources, invocation controls, prepared-input
reuse, and corpus-ledger invariants are executable. M1 now uses complete pinned
W3C case metadata to select a staged standards-driven preview and close AR-0001.
Representative consumer transforms refine application priorities and
ASP.NET/performance decisions in parallel; they do not block preview testing.

## License

FastXSLT is licensed under the [MIT License](LICENSE) so it can be embedded and
distributed by other applications, including commercial applications.
