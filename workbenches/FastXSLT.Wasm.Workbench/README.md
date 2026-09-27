# FastXSLT WASM Workbench

This unpublished workbench is the first AR-0015 feasibility adapter. It builds
the ordinary safe FastXSLT workbench lifecycle for `wasm32-unknown-unknown`,
generates a private Node binding, verifies the unchanged XSLT30 `for-004`
result, verifies a sealed in-memory include plus representative budget and
invalid-input diagnostics, and measures one sequential retained engine.

Run:

```powershell
./scripts/measure-wasm-workbench.ps1
```

The local probe requires the `wasm32-unknown-unknown` Rust target, Node.js, and
`wasm-bindgen-cli 0.2.126`. The CLI version is pinned to the crate version so
generated glue and the built module agree.

Pass a smaller or larger base iteration count when needed:

```powershell
./scripts/measure-wasm-workbench.ps1 -Iterations 5000
```

The generated binding and WASM artifact live under the gitignored
`.workbench/wasm-bindgen/` directory. The checked-in Rust adapter and JavaScript
harness are private evidence surfaces, not a supported package or API.

The report separates module loading, compile/prepare plus initial input copying,
warm execution/serialization, result transfer, a byte-copy calibration probe,
known retained engine capacity, and current linear-memory pages. The copy probe
is directional boundary evidence, not an exact subtraction oracle. The setup
measurement cannot yet separate host-to-WASM copies from compile and prepare.

The first benchmark deliberately uses one active invocation in one instance.
It does not select browser packaging, WASI, Web Workers, WASM threads,
cross-instance caches, or a public diagnostic/result binding.
