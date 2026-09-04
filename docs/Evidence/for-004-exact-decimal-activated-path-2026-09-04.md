# `for-004` Exact-Decimal Activated-Path Evidence

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Status | Complete focused experiment |
| Scope | Private exact-decimal evaluator admitted for XSLT30 `for-004` |
| Related review | [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md) |
| Prior localization | [ASP.NET native boundary breakdown](aspnet-native-boundary-breakdown-2026-09-03.md) |

## Question

The controlled .NET 8/.NET 10 comparison localized 93-99.8% of the listed
native-export component time to transformation and serialization. Which part of
that engine-owned work dominates the deterministic 5-, 50-, and 500-item
`for-004` workload, and can a private safe specialization remove the pressure
without changing semantics, work accounting, or host boundaries?

## Method

An ignored release-mode Rust probe split the already-prepared transform into
source lookup, invocation-control construction, semantic plan execution, direct
exact-decimal evaluation, and serialization. The probe uses the unchanged
XSLT30 `for-004` stylesheet and the same generated sources as the ASP.NET tiered
workbench. It is diagnostic instrumentation, not an ordinary test or benchmark
claim.

The profile nominated two allocations performed for each of the two attributes
on every selected item:

1. general XDM string-value construction copied the already-retained attribute
   lexical into a new `String`; and
2. exact-decimal parsing concatenated the integral and fractional slices into a
   second `String` before parsing it.

The comparison path now borrows an attribute's immutable retained lexical after
charging the same `XdmStringValueNode` work, and accumulates checked ASCII
digits directly into `i128`. The previous owned-string and concatenating parser
remain test-only complete references. No prepared representation, public API,
ABI, XDM identity, unsafe code, cache, or cross-invocation state was added.

## Focused observations

Values below are medians of five samples from the release-mode phase probe.
They are local means per invocation and include the probe's measurement shape.

| Tier | Before semantic execution | After semantic execution | Change | Before direct decimal evaluator | After direct decimal evaluator | Change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 5 items | 2.027 us | 1.193 us | -41.1% | 1.493 us | 0.642 us | -57.0% |
| 50 items | 12.255 us | 4.151 us | -66.1% | 11.713 us | 3.584 us | -69.4% |
| 500 items | 113.540 us | 31.633 us | -72.1% | 110.888 us | 30.984 us | -72.1% |

The allocation-observed 500-item evaluator comparison reported 20 allocation
requests for the activated path and 2,020 for the complete reference: exactly
four avoided requests per item, or 2,000 per transform. In that focused sample,
the activated path measured 32.435 us and the reference 114.223 us. These
absolute timings are diagnostic; the allocation-count delta and retained
mechanism identify the reason for the improvement.

Differential tests cover successful evaluation, exact work consumption in the
XPath operation, XPath node-visit, and XDM string-value domains, whitespace and
fractional lexical variants, malformed forms, and `i128` boundaries. The
existing behavior that refuses to invent decimal-rounding semantics remains
unchanged.

## ASP.NET result

Five fresh .NET 10 workbench processes exercised 1,000 base requests per tier
(10,000 transforms for 5 items, 4,000 for 50 items, and 1,000 for 500 items),
both sequentially and with four independent engines/workers. Values are medians
of the five process-level reports.

| Tier | Native sequential | Native x4 | Isolated sequential | Isolated x4 |
| --- | ---: | ---: | ---: | ---: |
| 5 items | 306,892/s | 1,034,854/s | 21,126/s | 83,649/s |
| 50 items | 166,110/s | 556,266/s | 19,556/s | 79,145/s |
| 500 items | 27,387/s | 107,662/s | 11,550/s | 42,033/s |

Against the pre-specialization .NET 10 medians recorded on 2026-09-03, native
sequential throughput changed by +17%, +166%, and +272% across the three tiers;
native x4 changed by +35%, +150%, and +298%. Isolated sequential changed by
+6%, +22%, and +116%; isolated x4 changed by +16%, +30%, and +90%.

The larger tiers provide the clearest attribution because evaluator work
dominates their fixed host costs. One of five 500-item native reports was a low
outlier, but it did not determine the median. Managed allocation figures remain
unchanged in meaning and exclude Rust allocations. This is one narrow admitted
execution plan, not a general FastXSLT or competing-engine performance claim.

## Disposition

Retain the safe activated path and both complete test references. This is
positive AR-0013 evidence for compile-selected, narrow machinery: a measured
hot loop was improved without a new representation or unsafe exception, and
the consumer-visible gain survived both native and isolated host boundaries.

Do not infer that general XDM string-value construction should return borrowed
storage, that all decimal semantics fit this representation, or that similar
specializations should be added without profiling. Continue phase-attributed
allocation work on representative language paths and preserve the complete
reference whenever practical.
