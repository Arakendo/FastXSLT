# Consumer Profile Intake and Template-Dispatch Slice

| Field | Value |
| --- | --- |
| Status | In Progress |
| Opened | 2026-08-25 |
| Owner | FastXSLT maintainers |
| Related reviews | AR-0001, AR-0002, AR-0003, AR-0004, AR-0007, AR-0011 |
| Depends on | Intended-consumer examples for profile closure |

## Purpose

Turn representative embedded-consumer transforms into an accepted initial
standards profile while allowing one narrowly justified private semantic slice
to continue exercising the architecture.

## Slice 0: Peer-derived candidate inventory

**Status:** Complete.

- [x] Inventory first-party TS XSLT goldens, workbench presets, curated XSLT30
  families, .NET resource pressure, and large stylesheet-graph evidence.
- [x] Separate peer-derived candidates from actual consumer requirements.
- [x] Identify exact element-name template dispatch through
  `xsl:apply-templates` as the leading private next slice.

## Slice 1: Private template dispatch

**Status:** In Progress.

- [x] Compile one root template plus exact unprefixed element-name templates.
- [x] Execute `xsl:apply-templates` over an explicit relative child-name path in
  source document order.
- [x] Retain compiled template rules separately from invocation context.
- [x] Add a first-party golden and focused invalid/unsupported cases.
- [ ] Link one native XSLT30 case only when its complete dependency,
  environment, and assertion shape fits the implemented slice.

This slice remains private and version-neutral. It does not accept match
priority, modes, named templates, imports/includes, or generalized pattern
grammar merely because the peer uses them.

## Slice 2: Intended-consumer intake

**Status:** Blocked on consumer artifacts or answers.

- [ ] Record representative stylesheets or reduced semantic equivalents.
- [ ] Record current processor/version compatibility requirements.
- [ ] Inventory input/result sizes, batch shapes, compile/update frequency,
  concurrency, latency/throughput targets, and memory limits.
- [ ] Record secondary resources, parameters, messages, extensions, output
  methods, and diagnostic dependencies.
- [ ] Convert the confirmed needs into dependency-aware suite selections.

## Slice 3: Standards disposition

**Status:** Pending.

- [ ] Compare the confirmed workload against AR-0001 alternatives.
- [ ] Propose and accept an ADR naming the initial profile, exclusions, suites,
  reporting policy, and widening criteria.
- [ ] Replace provisional semantic-slice language with the accepted standards
  ownership or explicitly remove incompatible experiments.
