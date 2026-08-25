# FastXSLT

FastXSLT is a new Rust-native XSLT engine for use inside other applications.
Its motivating consumers include ASP.NET applications that need reusable,
high-throughput transformations without treating an external command-line tool
as the primary product boundary.

The repository is currently at **M0: project scaffold**. It builds and its
development gates run, but it does not transform documents yet. In particular,
the supported XSLT/XPath standards profile has not been selected; see
[AR-0001](docs/Architectural%20Reviews/AR-0001-initial-standards-profile.md).

## Getting started

FastXSLT declares Rust 1.85 as its minimum supported Rust version and uses the
current stable toolchain for development, including `rustfmt` and Clippy. The
checked-in toolchain file configures the development toolchain automatically
through rustup, while CI also checks the minimum version.

```powershell
cargo test --workspace
cargo lint
cargo docs
```

Run every local gate with:

```powershell
./scripts/verify.ps1
```

## Repository layout

```text
crates/fastxslt/  Public library crate and initially private engine layers
corpus/           Small, reviewed transform cases that drive implementation
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
- evidence records observations but does not create guarantees.

## Project status

The [roadmap](docs/Plans/roadmap.md) defines milestone outcomes. The next useful
slice is a tiny, end-to-end transform chosen only after AR-0001 decides the
initial standards profile and conformance baseline.

## License

FastXSLT is licensed under the [MIT License](LICENSE) so it can be embedded and
distributed by other applications, including commercial applications.
