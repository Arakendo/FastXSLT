# OASIS XML indentation frontier checkpoint

## Scope

Resume ordinary corpus breadth toward 80% without changing upstream expectations,
the sealed-resource boundary, production DTD denial or compilation-profile ownership.
The input remains OASIS `XSLT-testsuite-04.ZIP`, SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`,
with 3,173 catalog entries. Imported fixtures remain in the local workbench;
no fixture content or expected output was copied into the crate.

## Experiment and disposition

`Microsoft/Completeness__84358#1` differs on indentation around an element's sole
comment child. An initial XML-only candidate indented comments and processing
instructions even without element children. Its full modern replay gained one
exact comparison: 2,501 passes and 13 mismatches.

That gain was **withdrawn**. The selected shared modern serializer must follow
its own constraints, not use archival whitespace as a semantic oracle.
[Serialization 3.1 section 5.1.4](https://www.w3.org/TR/xslt-xquery-serialization-31/#XML_INDENT)
permits indentation subject to content, suppression and preservation constraints.
The retained conservative eligibility requires at least one element child and
otherwise only elements, comments or processing instructions. Comment-only and
PI-only content remains inline, as does content containing text (including
whitespace-only text and disable-output-escaping text).

HTML/XHTML eligibility is unchanged. The semantic result is not mutated.
The iterative writer and independently expressed shallow recursive oracle adopt
the same deliberate formatting policy. Focused tests cover exact output,
exact/one-less byte and work limits, cancellation and namespace-scope unwind.
No public API, wire format, authority or unsafe surface changed.

## Conserved replays

| Preprocessing profile | Exact comparisons | XML mismatches | Expected-error credits |
| --- | ---: | ---: | ---: |
| Modern baseline | 2,500 / 3,173 (78.79%) | 14 | 423 / 431 |
| Strict-1.0 experiment | 2,497 / 3,173 (78.70%) | 17 | 423 / 431 |

The rejected gain is not included in either numerator. Strict preprocessing
selection does not by itself select a separate serialization edition.
80% requires 2,539 exact cases: 39 more modern matches or 42 more strict matches.

The refreshed modern standard-operation frontier contains 84 initialization
failures and 11 execution failures. Inspection finds invalid XML/reserved namespace
bindings, unavailable dependencies, deferred external/parameter entities, legacy
recovery and version-sensitive unsupported behavior among these entries. Do not
advertise their count as a promised pass tranche or relax XML validity to admit them.
Unknown XSLT fallback behavior needs the proper static compatibility owner;
the preprocessing selector is not a substitute for that decision.

## Validation and reproduction

```powershell
cargo test -p fastxslt --all-features xml_indentation
cargo test -p fastxslt --all-features xml_comment_indentation
& './scripts/measure-oasis-xslt10.ps1' -TraceCase 'Completeness__84358'
& './scripts/measure-oasis-xslt10.ps1' -PreprocessingProfile strict-xslt10
& './scripts/verify.ps1'
cargo check -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

All verification gates passed: formatting, strict Clippy, workspace tests,
documentation, Markdown links, unsafe-surface checks and pinned conformance
inventory. Core tests: 1,735 passed, 60 ignored; native adapter: 22 passed,
3 ignored; WASM adapter: 4 passed; worker: 18 passed. Official WASM target check
passed. Local hard-link cache warnings fell back to copying; they are not test
failures or a reason to weaken warning policy.

This checkpoint improves shared formatting behavior and narrows the frontier;
it does not increase exact coverage or establish XSLT 1.0 conformance.

## Follow-up: actionable frontier versus archival boundaries

Detailed `frontier-trace` records now carry `expected-error=true/false`, taken
from the same catalog classification that feeds the standard-operation counters.
This does not reclassify any test. It allows a complete standard-case trace
without accidentally prioritizing expected-error examples from the all-case
frontier. A focused test conserves both phase totals while excluding expected
errors from both standard-frontier maps.

```powershell
& './scripts/measure-oasis-xslt10.ps1' -TraceFrontier '/' |
    Select-String 'expected-error=false'
```

The following are implementation candidates, **not promised exact passes**:

| Candidate | First unmet shared responsibility | Important downstream pressure |
| --- | --- | --- |
| `Lotus/variable_variable40#1` | Execute a global content constructor through template dispatch | Global focus, dependency/cycle handling, budgets and temporary-tree ownership; not a compiler-only fallback |
| `Microsoft/Sorting_SortExprWithCurrentInsideForEach1#1` | Retain a global `document('parts.xml')/parts` selection | Qualified global values plus nested `current()` filters and sort focus; the existing document-root-only binding is insufficient |
| `Microsoft/Miscellaneous__84475#1` | Composed string-valued match predicates | Descendant first-node conversion, document-rooted paths, substring/concat and a second attribute predicate; no fixture-shaped recognizer |
| `Microsoft/Keys_PerfRepro2#1` | Dynamic `format-number` numeric operands | Nested translate/string/sum/key expressions and temporary node-set arithmetic, not merely another picture syntax |
| `Microsoft/XSLTFunctions_DocumentFunctionNoArg#1` and `DocumentInUnion#1` | Qualified union values | Explicit cross-owner identity/order and retained value ownership; preserve existing unsupported boundaries until integrated |

Prioritize a small shared global-constructor execution slice next, beginning
with the unchanged `variable40` case and independent focus/dependency controls.
Do not silently teach the static tree builder template execution or store
source-dependent values in compiled artifacts. Determine whether existing
accepted owners suffice before implementing the runtime seam.

The `Sorting__91753` trace has matching sort order; its difference is whitespace
inside the first empty `br`. It is not evidence of a sorting defect. The 11
shared ISO-2022-JP source failures reproduce the previously documented byte-994
redundant transition, so the
[existing negative decoder disposition](oasis-xslt10-iso2022jp-source-frontier-2026-09-28.md)
is retained, not rediscovered as a new parser feature. Invalid namespace bindings,
missing intranet resources and higher-version fallback/profile choices likewise
remain explicit rather than being turned into compatibility heuristics.

### Harness cohesion and verification

ADR-0004 review: the approximately 3,250-line measurement owner has size pressure.
This narrow trace annotation and counter-conservation test are retained in its
existing OASIS reporting responsibility; they introduce no engine dependency,
runtime policy or public abstraction. The dense runtime directory is not given
another generic helper file for this field. Before another substantive reporting
responsibility is added, checkpoint extraction of frontier reporting from probe
orchestration should be evaluated separately with the same conserved replay.

The full verification rerun passes with 1,736 core tests and 60 ignored, and
unchanged adapter/worker counts. No transformation code changed in this
follow-up. The earlier serializer changes and pending preprocessing work remain
separate, uncommitted worktree changes.
