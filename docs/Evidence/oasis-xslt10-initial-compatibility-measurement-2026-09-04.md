# OASIS XSLT 1.0 Initial Compatibility Measurement

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Archive SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Catalog denominator | 3,173 cases |
| Standard-operation denominator | 2,742 cases |
| Expected-error denominator | 431 cases |
| Definite unchanged XML passes | 366 standard-operation cases |
| Disposition | Local-only compatibility evidence; not a conformance claim |

This document preserves the initial baseline. The subsequent
[static computed-element tranche](oasis-xslt10-static-computed-element-tranche-2026-09-04.md)
raises the definite unchanged pass count to 386 without changing the conserved
denominators; the following
[location-path copy-of tranche](oasis-xslt10-location-path-copy-of-tranche-2026-09-04.md)
raises it to 395, and the following
[source node-kind copy tranche](oasis-xslt10-source-node-kind-copy-of-tranche-2026-09-04.md)
raises it to 403. The later
[static element namespace tranche](oasis-xslt10-static-element-namespace-tranche-2026-09-04.md)
raises it to 461. The subsequent
[descendant match-pattern repair](oasis-xslt10-descendant-match-pattern-repair-2026-09-04.md)
raises it to 462. The subsequent
[include-order and comment-text repairs](oasis-xslt10-include-order-and-comment-text-repairs-2026-09-04.md)
raise it to 464. The following
[static atomic copy-of tranche](oasis-xslt10-static-atomic-copy-of-tranche-2026-09-04.md)
raises it to 468. The following
[variable copy-of tranche](oasis-xslt10-variable-copy-of-tranche-2026-09-04.md)
raises it to 476. The subsequent
[copy-of path union tranche](oasis-xslt10-copy-of-path-union-tranche-2026-09-04.md)
raises it to 481. The following
[named processing-instruction path tranche](oasis-xslt10-named-processing-instruction-path-tranche-2026-09-04.md)
raises it to 485. The following
[context name and PI pattern tranche](oasis-xslt10-context-name-and-pi-pattern-tranche-2026-09-04.md)
raises it to 514. The following
[context expanded-name tranche](oasis-xslt10-context-expanded-name-tranche-2026-09-04.md)
raises it to 519. The following
[context string tranche](oasis-xslt10-context-string-tranche-2026-09-04.md)
raises it to 521. The following
[context normalize-space tranche](oasis-xslt10-context-normalize-space-tranche-2026-09-04.md)
raises it to 522. The following
[value focus tranche](oasis-xslt10-value-focus-tranche-2026-09-04.md)
raises it to 525. The following
[parent name tranche](oasis-xslt10-parent-name-tranche-2026-09-04.md)
raises it to 527. The following
[context string-length tranche](oasis-xslt10-context-string-length-tranche-2026-09-04.md)
raises it to 529. The following
[numeric literal comparison tranche](oasis-xslt10-numeric-literal-comparison-tranche-2026-09-04.md)
raises it to 553. The following
[homogeneous literal equality tranche](oasis-xslt10-homogeneous-literal-equality-tranche-2026-09-04.md)
raises it to 560. The following
[static concat tranche](oasis-xslt10-static-concat-tranche-2026-09-04.md)
raises it to 567. The following
[static binary string-functions tranche](oasis-xslt10-static-binary-string-functions-tranche-2026-09-04.md)
raises it to 592. The following
[static translate tranche](oasis-xslt10-static-translate-tranche-2026-09-04.md)
raises it to 598. The following
[static substring tranche](oasis-xslt10-static-substring-tranche-2026-09-04.md)
raises it to 606. The following
[context language tranche](oasis-xslt10-context-language-tranche-2026-09-04.md)
raises it to 607.
The following
[source node identity tranche](oasis-xslt10-generate-id-tranche-2026-09-04.md)
raises it to 608.
The following
[normalize-space path tranche](oasis-xslt10-normalize-space-path-tranche-2026-09-04.md)
raises it to 609.
The following
[expanded-name path tranche](oasis-xslt10-expanded-name-path-tranche-2026-09-04.md)
raises it to 613.
The following
[function-name whitespace tranche](oasis-xslt10-function-name-whitespace-tranche-2026-09-04.md)
raises it to 614.
The following
[constant integral-functions tranche](oasis-xslt10-constant-integral-functions-tranche-2026-09-04.md)
raises it to 638.
The following
[integral-function path tranche](oasis-xslt10-integral-function-path-tranche-2026-09-04.md)
raises it to 647.
The following
[string-function tranche](oasis-xslt10-string-function-tranche-2026-09-04.md)
raises it to 653 while retaining one newly visible modern cardinality failure.
The following
[name path tranche](oasis-xslt10-name-path-tranche-2026-09-04.md)
raises it to 657 while retaining lexical-prefix reconstruction and legacy
multi-node conversion as explicit boundaries.
The following
[qualified attribute path tranche](oasis-xslt10-qualified-attribute-path-tranche-2026-09-04.md)
raises it to 662 while retaining lexical-prefix reconstruction as an explicit
boundary.
The following
[reverse-axis tranche](oasis-xslt10-reverse-axis-tranche-2026-09-04.md)
raises it to 686 through 24 unchanged passes with reverse-axis positional
semantics.
The following
[following-axis tranche](oasis-xslt10-following-axis-tranche-2026-09-04.md)
raises it to 694 through eight unchanged forward-axis passes.
The following
[following node-kind tranche](oasis-xslt10-following-node-kind-tranche-2026-09-04.md)
raises it to 697 while leaving two later whitespace-profile failures visible.
The following
[ancestor-axis tranche](oasis-xslt10-ancestor-axis-tranche-2026-09-04.md)
raises it to 718 and repairs leading-`//` evaluation for arbitrary axes and
per-context positional predicates.
The following
[self node-kind tranche](oasis-xslt10-self-node-kind-tranche-2026-09-04.md)
raises it to 721 through three explicit XDM kind tests.
The following
[preceding node-kind tranche](oasis-xslt10-preceding-node-kind-tranche-2026-09-04.md)
raises it to 724 through three composed reverse-axis paths.
The following
[static AVT brace-escaping tranche](oasis-xslt10-static-avt-brace-escaping-tranche-2026-09-04.md)
raises it to 728 through four unchanged literal-result-attribute cases while
leaving dynamic attribute value templates explicit.
The following
[composed context-step tranche](oasis-xslt10-composed-context-step-tranche-2026-09-04.md)
raises it to 729 by treating `.` inside a path as the standard abbreviated
`self::node()` step.
The following
[axis-separator whitespace tranche](oasis-xslt10-axis-separator-whitespace-tranche-2026-09-04.md)
raises it to 731 through two unchanged attribute-axis cases while retaining one
newly exposed multi-node conversion boundary.
The following
[expanded-axis wildcard pattern tranche](oasis-xslt10-expanded-axis-wildcard-pattern-tranche-2026-09-04.md)
raises it to 734 by canonicalizing `attribute::*` and `child::*` to the existing
typed wildcard patterns.
The following
[expanded attribute-node pattern tranche](oasis-xslt10-expanded-attribute-node-pattern-tranche-2026-09-04.md)
raises it to 737 by canonicalizing `attribute::node()` to the same typed
any-attribute pattern and priority.
The exact `{.}` AVT then raises it to 738 through a shared charged context
string-value operation; two additional cases reach visible line-ending
comparison mismatches and remain uncredited.
Checked source-free arithmetic with an exactly integral result then raises it
to 747 through nine unchanged cases without adding a later failure or mismatch.
Static finite `number()` conversion then raises it to 749 through two unchanged
cases while leaving context, path, NaN, infinity, and compound arguments
explicitly unsupported.

## Outcome

FastXSLT now has a repeatable, hash-verified local measurement over every case
in the archival OASIS catalog. The first refined sweep establishes a definite
lower bound of **366 unchanged XML comparison passes**, or **13.35% of the 2,742
standard-operation cases** and **11.53% of the complete 3,173-case catalog**.

Those percentages deliberately count only successful engine execution followed
by a successful owned XML comparison. They do not count an arbitrary failure
on an expected-error case as a pass, infer behavior for a skipped dependency,
or turn an unimplemented comparator into an engine failure.

## Acquisition and execution

The archive remains under ignored `.workbench` storage and is not redistributed.
`scripts/measure-oasis-xslt10.ps1`:

1. resolves the local archive and extracted `TESTS` directory;
2. verifies the reviewed SHA-256 before execution;
3. requires both `catalog.xml` and `doubts.xml`;
4. rejects a catalog denominator other than 3,173; and
5. invokes the ignored release measurement with the extracted root supplied by
   environment rather than granting the engine filesystem authority.

For targeted investigation, an optional case-identity fragment emits a bounded
initialization failure, execution failure, or actual/expected comparison trace:

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/math_math04'
```

Tracing does not filter the conserved sweep or change any case's disposition.

The Rust measurement parses the catalog and doubts metadata through FastXSLT's
bounded XML/XDM path, assigns duplicate-safe `submitter/id#ordinal` identities,
loads principal and catalog-listed supplemental stylesheet bytes, and seals
them before compilation. Engine execution therefore remains memory-resident.
The harness reads expected bytes only after execution and owns comparison.

The comparator first attempts an XML document comparison. When legacy XML
comparison cases intentionally produce text-only or multi-root fragments, it
retries both sides inside a private comparison wrapper. UTF-8 and BOM-marked
UTF-16 expected results are decoded locally. DTD-bearing, other legacy-encoded,
or otherwise unparsable results stay comparator-unsupported rather than being
called failures.

## Conserved first sweep

| Observation | Cases |
| --- | ---: |
| Catalog cases | 3,173 |
| Cases with doubt/annotation metadata detected by the runner | 144 |
| Engine initialization succeeded | 506 |
| Engine initialization failed with a structured outcome | 2,629 |
| Supplemental-data cases not admitted | 35 |
| Missing principal source | 2 |
| Missing catalog-listed supplemental stylesheet | 1 |
| Execution succeeded after initialization | 410 |
| Execution failed with a structured outcome | 96 |
| Standard cases with XML comparison pass | 366 |
| Standard cases with XML comparison mismatch | 20 |
| Standard cases with comparator unsupported | 14 |
| Expected-error failure observed during initialization | 410 |
| Expected-error failure observed during execution | 8 |
| Expected-error case unexpectedly executed successfully | 10 |
| Engine panic | 0 |

The 418 observed expected-error failures are useful negative-path pressure, but
the draft catalog generally states only `execution-error`. Until the harness
maps the relevant specification citation, discretionary behavior, doubts
metadata, and expected phase/category, these are observations rather than 418
additional passes.

Thirty-five cases name supplemental data. They remain explicitly not admitted
because the current production engine does not expose the required sealed
runtime document lookup. Silently omitting those resources and accepting a
coincidental result would inflate the pass count.

## Current frontiers

The largest first-failure groups are shared modern-engine work rather than
evidence for a separate evaluator:

| Frontier | Cases |
| --- | ---: |
| Function-shaped expression outside the location-path grammar | 301 |
| Unsupported `xsl:element` instruction | 195 |
| Other `FXXP1001` location-path syntax | 157 |
| Unsupported `xsl:copy-of` selection | 152 |
| Unsupported `xsl:sort` instruction | 135 |
| Unsupported `xsl:number` instruction | 112 |
| Unsupported top-level `xsl:key` | 89 |
| Unsupported template match pattern (`FXST1005`) | 78 |
| Unsupported instruction/output attributes (`FXST1009`) | 77 |
| Stylesheet XML rejected by the current XML profile (`FXXM0001`) | 73 |

The broader diagnostic-code totals behind that split remain 773 `FXXP1001`
location-path failures, 501 unsupported instructions, and 267 unsupported
top-level declarations. The runner now splits common instruction/declaration
names and coarse XPath shapes so subsequent sweeps can measure movement in
actionable semantic families rather than only large diagnostic buckets.

The 20 known comparison mismatches already identify real semantic pressure in
template conflict resolution, include/import precedence, output indentation,
whitespace, namespace fixup, modes, named templates, copying, and `xsl:text`.
The 14 comparator gaps comprise ten actual results that are not accepted as an
XML document or fragment, two DTD-bearing/unparseable expected results, and two
expected results in encodings outside the runner's UTF-8/UTF-16 slice.

These are first-failure counts, not mutually complete feature requirements. A
case may expose another missing feature after its first blocker is implemented.

## Defect found and repaired

The initial sweep panicked on `Lotus/select_select13#1`. Its global variable
selected source nodes and a later `xsl:for-each` followed a child path from that
variable. The compiler represented variable-rooted paths using the same private
operation used for temporary-tree paths, while runtime dispatch assumed the
variable must contain a temporary tree.

The shared runtime now resolves source-node variables before temporary-tree
dispatch, walks each named child step under normal XPath work charging, and
normalizes the result to document order without duplicates. A focused
hand-authored XSLT 1.0 regression exercises the production path. The complete
legacy sweep then finished with zero panics.

This repair did not add a legacy backend or an XSLT 1.0-only data model. It
improved the same source-node variable semantics used by the staged modern
engine.

## What this establishes

- A nontrivial XSLT 1.0 compatibility floor already exists in the production
  engine.
- The suite can now pressure all 3,173 catalog cases without importing its bytes
  into the MIT repository.
- At least one defect discovered by the legacy suite was repaired through the
  shared modern execution path.
- Most large first blockers name XPath, XSLT instruction, template, import, and
  serialization work that is also on the XSLT 3.0 path.

It does **not** establish XSLT 1.0 conformance, expected-error correctness,
support for the suite's discretionary/recovery choices, legacy result-tree
fragment rules, backwards-compatible conversion behavior, general
`document()` acquisition, DTD/entity admission, legacy encodings, or complete
result comparison.

## Next experiment

Use the runner as a compatibility frontier, not as a source of easy-case
sampling:

1. classify the dominant XPath and instruction first-failure groups by actual
   feature;
2. repair the 20 semantic mismatches or mark an explicit compatibility-specific
   boundary;
3. reduce the 14 comparison gaps without weakening XML/resource policy;
4. design expected-error phase/category comparison around catalog citations and
   doubts metadata;
5. admit supplemental data only through the same sealed-resource authority used
   by production; and
6. compare every proposed legacy behavior against XSLT 3.0 semantics before
   allowing it to shape the core representation or public profile.

AR-0019 now owns the formal intermediate compatibility checkpoint: determine
the exact named XSLT 1.0 profile and isolate the behaviors that require explicit
version-dependent semantics while shared work continues toward XSLT 3.0.
