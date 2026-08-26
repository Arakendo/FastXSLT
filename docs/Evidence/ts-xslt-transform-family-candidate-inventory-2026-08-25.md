# TS XSLT Transform-Family Candidate Inventory

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Peer workspace | `F:\LocalSource\TS XSLT` |
| Observed commit | `9c48142adb00f9808f9e8029b57ffe8bc1b477f3` |
| Worktree state | Modified; observations are a local peer snapshot, not immutable provenance |
| Decision pressure | AR-0001 representative transform families and AR-0002 host workload shape |
| Claim | Candidate questions from a peer project; not first-consumer requirements |

## Observed family ladder

The peer's five first-party golden families and workbench presets form a useful
progression:

1. literal result construction;
2. `xsl:value-of` over a source path;
3. `xsl:apply-templates` over repeated elements with built-in traversal and
   element-name template matching;
4. competing match patterns and declaration-order/default-priority behavior;
5. stylesheet parameters, variables, conditionals, named templates, and calls.

Its curated XSLT30 MVP slice names 73 cases. Case-name families include 23
string, 13 expected-error, 11 parameter, 10 variable, five call-template,
three choose, two conflict-resolution, two literal-result, and individual
node-test, position, template, and XPath-default-namespace cases. These counts
describe the peer's deliberate subset, not FastXSLT applicability or support.

## Resource and host pressure

The peer's .NET fixtures expose a second, later boundary:

- `xsl:include` and `xsl:import` compose a stylesheet graph;
- `document()` loads a secondary source through an explicit URI resolver;
- build-time compilation and request-time execution have different host and
  deployment costs; and
- generated or compiled artifacts must be replaceable by identity without
  moving semantic ownership into MSBuild or C#.

The peer's synthetic compile fixture uses 24 wrapper imports, 72 generated
match templates, named-template calls, parameters, variables, conditionals,
attributes, and repeated apply-template selection. Its recorded S1000D work
found 754 template rules, 5,972 XPath parses, 719 match-pattern parses, and
6,145 QName resolutions after duplicate-composition and source-location fixes.
This is valuable future scale pressure, but it is not an initial-profile target.

## Implications for FastXSLT

The strongest next private semantic candidate is element-name template dispatch
through `xsl:apply-templates`. It occurs in the peer's first non-trivial golden,
workbench demonstration, curated standards strategy, and real-world workload.
It also pressures engine-owned node identity, document order, dynamic context,
compiled-versus-invocation state, and instruction accounting without requiring
imports, modes, packages, schema awareness, or a public profile claim.

The candidate should be staged narrowly:

- exact unprefixed element-name match patterns;
- one required root template;
- explicit child-name selection first;
- stable source document order and isolated invocation context;
- unsupported classification for priorities, modes, named templates, and
  broader pattern grammar until their cases justify admission.

## Consumer questions still required

Before AR-0001 closes, an intended consumer must confirm or replace this ladder
with representative evidence:

- stylesheet versions and processors currently relied upon;
- common instruction, match-pattern, XPath, function, and serialization use;
- imports/includes, `document()`, extension functions, messages, and parameters;
- source/stylesheet/result size distributions and batch shapes;
- compile/update frequency, concurrency, latency, throughput, and memory limits;
- diagnostics and compatibility behavior the application depends upon.

No peer observation satisfies that first-consumer requirement by itself.
