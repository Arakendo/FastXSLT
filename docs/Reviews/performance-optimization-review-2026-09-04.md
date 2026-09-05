# FastXSLT Performance Optimization Review

| Field             | Value                                                                                                                                                                                                     |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Date              | 2026-09-04                                                                                                                                                                                                |
| Source checkpoint | `ee659758a867fa6698e6468043f554223f73d15c`                                                                                                                                                                |
| Review type       | Adversarial performance and allocation review                                                                                                                                                             |
| Primary workload  | Pinned XSLT30 `for-004`, 5/50/500 deterministic `order-item` elements                                                                                                                                     |
| Status            | Complete review; P1 candidates closed, bounded safe-text P2 retained, result-builder P2 nominated                                                                                                          |
| Input evidence    | [ASP.NET native boundary breakdown](../Evidence/aspnet-native-boundary-breakdown-2026-09-03.md); [`for-004` exact-decimal activated path](../Evidence/for-004-exact-decimal-activated-path-2026-09-04.md) |
| Governing review  | [AR-0013 prepared representation and data-layout audit](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md)                                                              |

## Executive result

The managed/native boundary is not the current optimization target. At 500
items, the existing boundary probe attributes 99.8% of its listed native
components to transformation plus serialization. Registry lookup, outcome
publication, release, and request copying are individually measured in hundredths
of a microsecond. No evidence supports redesigning those paths for this
sequential workload.

The proposed `45% XPath / 30% result construction / 20% serialization / 5%
plan-dispatch` split is a useful scenario, but it is **not a measured result in
the cited evidence**. The conversation that introduced it explicitly said “if
that comes back something like”. Treating it as observed would send work toward
the wrong components.

A fresh release-mode run of the checked-in phase probe gives a much more skewed
500-item result for `for-004`: the direct exact-decimal evaluator is about 96%
of independently measured semantic-execution-plus-serialization time;
serialization is about 1%; and all remaining semantic work is about 2%. The
result is tiny (`<out>...</out>`), so this workload cannot establish that result
construction or serialization is generally cheap. It establishes only that
they are not the next target for this case.

The highest-value next experiment is a compile-selected, order-preserving path
loop for the simple child binding in `for-004`, followed by one-pass lookup of
its two required attributes. The general evaluator currently materializes
candidate and match vectors and sorts/deduplicates every step even where static
plan facts can prove that a child-axis scan is already ordered and unique. The
specialized decimal evaluator then scans the same attribute slice separately
for `price` and `qty`.

Result-heavy and text-heavy workloads now have separate fixtures. The
append-oriented result-builder candidate has measured linear allocation
pressure but has not been prototyped. A private bounded safe-text writer reduced
local serializer time by 72-77% and improved median throughput through every
measured native and isolated ASP.NET lane, so it was retained with the complete
character-wise path as oracle.

## Evidence and limits

The following values are medians of five local samples from:

```powershell
cargo test --release -p fastxslt --all-features `
  measure_workbench_transform_phases -- --ignored --nocapture
```

They are diagnostic mean microseconds per invocation, not a stable benchmark
contract. “Direct decimal” separately evaluates the dominant expression; it is
not a nested timer and therefore must not be arithmetically subtracted as if all
measurements came from one instrumented call.

| Tier      | Semantic execution | Direct decimal | Serialization |
| --------- | -----------------: | -------------: | ------------: |
| 5 items   |           1.184 us |       0.650 us |      0.321 us |
| 50 items  |           4.230 us |       3.642 us |      0.334 us |
| 500 items |          33.715 us |      32.897 us |      0.442 us |

At 500 items, using semantic execution plus serialization as a directional
denominator gives approximately 96.3% direct decimal evaluation, 2.4% remaining
semantic work, and 1.3% serialization. Probe overhead and the independent
evaluation prevent stronger precision. The checked-in evidence already records
that the activated evaluator reduced the 500-item path from 113.540 us to
31.633 us and removed 2,000 of 2,020 observed allocation requests. This review
therefore looks for remaining linear work rather than reopening the string-copy
optimization that produced that gain.

One additional ignored release test measured an unexhausted `charge` call at
1.479 ns versus a 0.203 ns loop baseline. That test is compiled with test-only
observations and is not a production attribution. It merely makes work-control
traffic worth measuring in a production-shaped profile before changing its
semantics.

## Ranked optimization candidates

### P1 — Specialize monotonic, duplicate-free location paths

**Observed mechanism.** `evaluate_location_path_controlled` creates `current`
and `next` vectors for path stages. For every input node it also obtains a fresh
candidate vector from `step_candidates`, creates a second `named_candidates`
vector, and finally calls `sort_unstable_by_key` and `dedup` on the step output.
The default child and attribute candidate paths copy retained slices with
`to_vec()`.

The `for-004` binding is the simple relative child path `order-item`. From one
context node, the retained child slice is already in document order and cannot
contain duplicate node identities. Candidate copying, match materialization,
sorting, and deduplication are therefore implementation costs rather than work
required by this specific plan shape.

**Candidate experiment.** Compile a private path operation only when static
facts prove an order-preserving, duplicate-free child or attribute step with no
predicate requiring a materialized match count. Scan the retained slice once,
filter by the pre-resolved name test, and append directly. For the exact-decimal
plan, test fusing that scan with its consumer so the selected `tuples` vector is
not created at all.

**Conservation requirements.** Preserve node identity, document order,
predicate context size, diagnostic provenance, work-domain totals, failure
ordering, and bounded cancellation observation. Keep the complete location-path
evaluator as a differential oracle. Do not generalize the shortcut to paths
whose axes, overlapping contexts, or predicates can introduce disorder or
duplicates.

**Admission evidence.** Report allocations and bytes, nodes visited versus
returned, 50/500/5,000-item latency, end-to-end ASP.NET throughput, and bounded
failure parity. Include low-selectivity and mixed-node child lists so a fast
happy path is not bought with pathological rejection cost.

### P1 — Resolve the decimal plan's paired attributes in one pass

**Observed mechanism.** For every selected `order-item`, `evaluate_using` calls
`find_attribute` once for `price` and again for `qty`. Each call starts at the
beginning of `document.attributes(context)` and charges each node visited. In
the deterministic fixture order this commonly inspects one attribute for the
first name and two for the second: three scans/visits for two retained nodes.

**Candidate experiment.** Add a plan-private paired lookup that scans the
attribute slice once and records both pre-resolved expanded-name matches. Stop
as soon as both are found where semantics permit. Measure it before considering
a prepared per-element name index; two attributes are unlikely to repay a
general index's construction and retention cost.

**Conservation requirements.** Missing and duplicate-attribute behavior,
namespace matching, visit accounting, cancellation points, and which failure is
observed first must remain deliberate. If fewer physical visits mean fewer
charged visits, record that as a work-accounting decision and update exact
budget-boundary tests; do not silently preserve or silently change the count.

### P1 measurement — Separate useful XPath work from control traffic

**Observed mechanism.** Every charged XPath node visit, string-value access,
and arithmetic operation calls `InvocationControl::charge`. The call checks a
shared cancellation atomic, selects a work-domain limit and remaining counter,
checks exhaustion, and decrements it. `for-004` performs this in its per-item
hot loop.

**Candidate experiment.** First add production-shaped attribution of time and
charge counts by domain. If control traffic is material after path fusion,
compare specialized domain methods or a bounded loop-local charging mechanism.
Exact budgets must still reject before unpermitted work, and cancellation must
remain observable within a documented bound. A lower atomic polling frequency
is a semantic policy decision, not a free micro-optimization.

Do not optimize from the 1.479 ns test measurement alone, and do not remove
charges merely because the common request uses unbounded limits.

### P2 — Build result sequences into caller-owned destinations

**Observed mechanism.** `execute_sequence` allocates a result vector and many
instruction helpers return their own `Vec<ResultNode>` for the caller to
`extend`. Loops, template application, source copying, and conditional
execution repeat this shape. Literal-element construction first builds its body
result, then iterates it into a second `children` vector while separating
pending attributes. Literal and computed attributes are likewise built in
separate vectors before concatenation.

**Candidate experiment.** Introduce a private append-oriented `ResultBuilder`
or `&mut Vec<ResultNode>` execution contract. Retain separate child buffers only
at actual element-nesting boundaries. Give the literal-element builder an
attribute-open/children-open state so pending attributes are classified during
execution instead of in a second pass. Reserve capacity only where the compiled
plan provides a safe, bounded output-multiplicity estimate.

**Conservation requirements.** Preserve adjacent-text coalescing, duplicate and
late-attribute errors, budget-before-mutation behavior, recursion failure
unwinding, result ownership, and the architectural separation between semantic
result construction and serialization. Do not turn the builder into a direct
serializer merely to make this benchmark faster.

**Admission evidence.** Use a new result-heavy fixture with hundreds or
thousands of constructed elements, attributes, and mixed text—not `for-004`.
Measure allocation requests/bytes, peak live bytes, semantic execution,
serialization separately, and host-visible throughput.

### P2 — Write serializer-safe text in chunks

**Observed mechanism.** Attribute and text escaping iterate Unicode scalar by
scalar. Ordinary characters call `BudgetedString::push`, which routes every
character through `push_str`, a work charge, checked length addition, byte-limit
test, and `String` growth. C1 numeric references and non-ASCII URI bytes use
`format!` in the inner loop. `BudgetedString` starts without capacity and owns a
copy of the request identifier.

**Candidate experiment.** Scan for escape, character-map, and normalization
boundaries, then append maximal safe UTF-8 runs with one checked write. Format
numeric references and percent escapes into fixed stack buffers. Evaluate a
bounded initial capacity derived from a cheap compiled/result estimate; avoid a
second full result-tree walk merely to compute an exact capacity.

**Conservation requirements.** Charge exact serialized bytes, enforce the same
limit before each mutation, preserve normalization and character-map order, and
retain bounded cancellation latency for long safe runs. Cover XML, HTML, XHTML,
text, URI escaping, C1 controls, multibyte boundaries, and byte-limit edges.

### P2 — Replace per-element namespace-scope cloning with a stack

**Observed mechanism.** `element_namespace_scope` clones the entire inherited
namespace slice for every element. Each local binding linearly searches the
scope, linearly retains by prefix, and clones into the new scope. Element and
attribute prefix selection then linearly scan that scope again. Cost grows with
both tree depth and in-scope namespace count.

**Candidate experiment.** Give the serializer an invocation-owned namespace
stack with a push/pop change log and prefix/namespace lookup state. Siblings
must see the restored parent scope. Prefer a safe implementation and measure
whether maps outperform short linear slices before admitting extra machinery.

**Admission evidence.** Deep and wide result trees with shadowing, default
undeclarations, multiple prefixes per URI, namespaced attributes, and sibling
scope changes. Report allocation and latency scaling against depth and binding
count as well as byte-for-byte output parity.

### P3 measurement — Scope cloning outside the atomic frame

`execute_sequence` clones `RuntimeVariables` on entry. Atomic variables use the
invocation-private copy-on-write scheme required by ADR-0014, but other maps and
vectors in the runtime frame may still make nested sequences or template calls
pay ownership costs. This is a plausible result-execution cost, not a finding
for `for-004`. Instrument clone counts and bytes by field and call-site before
extending sharing. Any change must remain invocation-private and preserve the
complete-clone oracle.

### P3 — Leave plan/dispatch and registries unchanged without new evidence

The broad plan-dispatch share is unmeasured here, and the current 500-item phase
probe leaves only about 2.4% for all non-decimal semantic work combined. The
native registry operations are already measured at roughly 0.035–0.062 us each
in the cited sequential probe. Even eliminating a hypothetical 5% plan/dispatch
share entirely would cap speedup at about 1.05x. Template dispatch and registry
contention deserve their own fanout/concurrency workloads, but neither is a
priority inferred from `for-004`.

## Scenario value of the proposed 45/30/20/5 split

If a future representative workload actually measures the proposed split, the
following Amdahl limits help rank experiments. They are scenario calculations,
not FastXSLT observations.

| Hypothetical improvement  | Removed total time | Maximum speedup |
| ------------------------- | -----------------: | --------------: |
| Halve XPath evaluation    |              22.5% |           1.29x |
| Halve result construction |              15.0% |           1.18x |
| Halve serialization       |              10.0% |           1.11x |
| Eliminate plan/dispatch   |               5.0% |           1.05x |

That scenario would support simultaneous XPath, result-builder, and serializer
experiments. The current `for-004` measurement supports XPath work first and
requires separate workloads before the other two can make consumer-visible
claims.

## Recommended experiment sequence

1. Add bounded path-shape observations: axis, steps, candidates scanned,
   matches returned, ordering repair performed, and transient allocation
   requests/bytes.
2. Prototype the monotonic child-path loop behind compile-time eligibility and
   compare it differentially with the complete evaluator.
3. Add one-pass paired attribute resolution and separately account for any
   deliberate change in work charges or cancellation points.
4. Re-run Rust phase attribution and allocation observation at 50, 500, and
   5,000 items; require an end-to-end ASP.NET gain before retaining the path.
5. Establish result-heavy and serialization-heavy fixtures. Only then prototype
   the append-oriented builder and chunked writer.
6. Measure namespace-stack and runtime-frame candidates only when their targeted
   fixtures show material time or retained-memory pressure.

For every retained optimization, record preparation cost, warm allocation
requests and bytes, peak and retained memory, p50/p95/p99 latency, throughput,
failure and budget parity, and concurrency behavior. Preserve safe complete
paths as differential oracles where practical. No candidate in this review
admits unsafe code, ambient caches, cross-invocation state, an ABI change, or a
public representation contract.

## Disposition

### Follow-up status

The first P1 candidate has been measured and closed without retention. A safe
compile-selected monotonic child scan removed 18 of the complete path's 20
allocation requests at 500 items and was faster in its focused loop, but a
same-machine ASP.NET A/B did not show a repeatable host-visible gain. Native x4
was effectively unchanged and native sequential 500-item behavior was worse and
bimodal. The complete production evaluator was restored.

[Experiment evidence](../Evidence/for-004-monotonic-child-path-experiment-2026-09-04.md)

The paired-attribute P1 experiment is also complete without retention. It
removed one redundant visit per selected item and improved its focused loop by
about 5%, but the 500-item native sequential A/B was flat and native x4 was
slower. The complete separate lookup was restored.

[Paired-attribute evidence](../Evidence/for-004-paired-attribute-lookup-experiment-2026-09-04.md)

Both implementation P1 candidates are therefore closed. Production-shaped
work-control inventory confirms `8N + 1` charge calls for the measured evaluator.
Its replay timing is test-build-biased by test-only observation bookkeeping, so
production cost attribution remains unresolved. A no-charge production build
was rejected before execution because it would globally weaken cancellation and
budget enforcement. No weakened artifact ran.

[Work-control shape evidence](../Evidence/for-004-work-control-shape-2026-09-04.md)

Further work-control measurement does not authorize weaker budget or
cancellation semantics. Result construction, namespace-scope, frame-cloning,
plan/dispatch, registry, and unsafe changes remain unadmitted by this follow-up.

The review's workload-separation gate is now satisfied. The result-heavy probe
observed roughly eight allocation requests per constructed item and nominates,
but does not admit, the append-oriented builder. The text-heavy probe justified
one retained safe implementation: XML-safe text without character maps or
normalization is emitted in at most 4 KiB writes. Exact byte-limit failure,
charged bytes, partial output, and bounded cancellation observation are
differentially conserved. Five-process .NET 10 A/B medians improved by
1.33-2.75x through isolated workers and 2.84-3.75x through native hosting.

[Result and text fixture evidence](../Evidence/result-and-text-heavy-performance-fixtures-2026-09-05.md)

The two implementation-specific P1 experiments are closed without retention.
Of the P2 experiments, bounded safe-text serialization is retained, result
construction is now a measured candidate, and namespace-scope work still lacks
its required targeted fixture. No follow-up accepts a new architecture or
authorizes a shortcut around resource accounting.
