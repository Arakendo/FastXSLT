# Consumer Profile Intake and Template-Dispatch Slice

| Field | Value |
| --- | --- |
| Status | In Progress |
| Opened | 2026-08-25 |
| Owner | FastXSLT maintainers |
| Related reviews | AR-0001, AR-0002, AR-0003, AR-0004, AR-0007, AR-0011 |
| Depends on | Pinned W3C metadata for preview scope; consumer examples for product-fit and host priorities |

## Purpose

Use pinned W3C cases to establish a testable staged standards profile while
collecting representative embedded-consumer transforms in parallel to refine
product priorities, compatibility, workloads, and host integration.

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
- [x] Add default child selection, built-in document/element/text behavior, and
  context-item value selection through the same reference path.
- [x] Screen syntax-light XSLT30 apply-template candidates and retain the
  dependency/feature gaps when no complete case fits.
- [ ] Link one native XSLT30 case only when its complete dependency,
  environment, and assertion shape fits the implemented slice.

This slice remains private and version-neutral. It does not accept match
priority, modes, named templates, imports/includes, generalized pattern
grammar, or `id()`/DTD semantics merely because the peer or a nearby suite case
uses them.

## Slice 2: Parallel intended-consumer intake

**Status:** Awaiting consumer artifacts or answers; not blocking the
standards-driven preview.

- [ ] Record representative stylesheets or reduced semantic equivalents.
- [ ] Record current processor/version compatibility requirements.
- [ ] Inventory input/result sizes, batch shapes, compile/update frequency,
  concurrency, latency/throughput targets, and memory limits.
- [ ] Record secondary resources, parameters, messages, extensions, output
  methods, and diagnostic dependencies.
- [ ] Convert the confirmed needs into dependency-aware suite selections.

## Slice 3: Standards disposition

**Status:** In Progress from pinned W3C evidence.

- [x] Inventory dependency, environment, stylesheet, and assertion families in
  the pinned XSLT30 suite.
- [x] Select the complete six-case XSLT30 `template` test set as the first
  coherent preview denominator, retaining one pass and five engine-unsupported
  cases from complete upstream metadata and compiler outcomes.
- [ ] Compare the standards-driven preview and known consumer class against
  AR-0001 alternatives; use later consumer artifacts to reprioritize widening,
  not to retroactively define whether the preview was testable.
- [ ] Propose and accept an ADR naming the initial profile, exclusions, suites,
  reporting policy, and widening criteria.
- [ ] Replace provisional semantic-slice language with the accepted standards
  ownership or explicitly remove incompatible experiments.
