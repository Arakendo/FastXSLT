# Template Priority, Selection, and Pattern Decomposition Review

Date: 2026-08-29

## Trigger

The retained runtime/compiler review required reopening before template
priority or indexing and at 1,200 lines for the top-level compiler. Bounded
explicit priority and additional conflict-resolution patterns have now made
both triggers concrete. Immediately before structural extraction:

| Source unit | Physical lines |
| --- | ---: |
| `runtime/golden_runtime_experiment.rs` | 1,009 |
| `compile/golden_stylesheet_experiment.rs` | 1,309 |

The purpose of this review is responsibility ownership, not reaching a numeric
score.

## Observed seams

Compiled priority removes one former source of mutual recursion: dispatch no
longer needs to rediscover default priority from pattern categories. Source
template selection can therefore own:

- mode eligibility;
- pattern applicability against source XDM nodes;
- charged attribute-predicate and relative-path matching;
- retained-priority comparison; and
- last-declared selection among equal priorities.

It returns one borrowed compiled template or no selection. It does not execute
the template body, recurse through built-in rules, construct results, bind
parameters, serialize, or own invocation/resource policy.

Template pattern compilation now also has an independent normalization seam:

- recognize the admitted pattern grammar;
- preserve typed expanded names and location paths;
- compute default priority once;
- parse bounded explicit priority; and
- classify invalid versus unsupported priority/pattern syntax.

It returns the existing private `MatchPattern` and `TemplatePriority`. It does
not compile template bodies, modes, parameters, globals, named calls, modules,
or whole-program validation.

## Decomposition

| Owner | After extraction | Dependency direction |
| --- | ---: | --- |
| Runtime sequence/template composition | 919 lines | calls selector, executes returned body |
| `runtime/template_selector.rs` | 111 lines | XDM + XPath path seam + compiled semantics + control/failure adapter |
| Single-document stylesheet compiler | 1,213 lines | calls pattern compiler, retains template/top-level assembly and tests |
| `compile/template_pattern_compiler.rs` | 117 lines | stylesheet XDM + path parser + compile failure helpers |

The compiler parent remains above 1,200 because its cohesive integration tests
and top-level/template assembly still belong together. The trigger has been
reviewed and one demonstrated child owner extracted; no line-count-only split
is justified. Reopen the parent at 1,500 lines, when another top-level
declaration/validation owner appears, or when its integration tests develop an
independently reusable harness.

Temporary-tree template selection remains in the runtime parent. That path has
a different representation and deliberately supports only exact/wildcard
selection plus a copy-only body. Moving it into the source-XDM selector would
create a generalized provider abstraction without evidence, contrary to
AR-0007.

## Conservation

The extraction must preserve:

- source node identity and document order;
- mode eligibility and built-in rules;
- explicit/default priority and source-order tie resolution;
- exact work-domain charging and structured control failures;
- compiled/prepared reuse, request isolation, and generation ownership;
- result bytes and structured diagnostics;
- native and isolated host behavior; and
- every existing corpus disposition.

Full workspace formatting, Clippy, tests, documentation, Markdown-link checks,
unsafe-surface enforcement, and pinned-corpus integrity are the closing gate.
No public API, ABI, unsafe surface, resource authority, or semantic type was
introduced by this structural change.

## Disposition

The named reopening triggers are discharged. Retain the two one-way private
owners. Reopen source selection when indexing, import/package precedence,
another execution strategy, or materially different pattern evaluation needs a
new contract. Reopen pattern compilation when namespaces, unions, generalized
predicates, or another pattern grammar phase causes independent pressure.

