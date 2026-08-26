# Private Host-Owned Two-Stage Workflow

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Fixture | `corpus/golden/host-owned-two-stage` |
| Decision | ADR-0005 unordered transform sets and host-owned workflow |
| Claim | Private executable workflow-boundary evidence; no public orchestration API |

## Executed flow

The first sealed snapshot contains only the stage-one source and stylesheet.
Stage one produces an identified serialized result:

```xml
<message>Hello, FastXSLT!</message>
```

A separately sealed stage-two snapshot contains only the stage-two stylesheet.
Even after stage one completes, adding a stage-two request whose source identity
matches the intermediate result fails with structured missing-resource code
`FXRS0001`. Result identity does not mutate the earlier snapshot or create
resource authority.

The host then copies the selected serialized result bytes into a new resource
builder under the intermediate logical identity, admits the stage-two
stylesheet, and seals a later snapshot. Stage two consumes that explicitly
admitted resource and produces:

```xml
<stage-two>Hello, FastXSLT!</stage-two>
```

## Conservation

- Both stages use the same private compile, transform-set, semantic-result, and
  serialization paths as a batch of one.
- The intermediate result retains explicit logical identity independent of a
  path or worker.
- The earlier snapshot remains immutable and cannot observe later host state.
- No sibling-result lookup, implicit promotion, filesystem publication,
  workflow graph, or evolving resource universe is introduced.
- The host owns the copy, selection, admission, ordering, and later snapshot.

## Limitations

The fixture is sequential and in-process. It does not select a public host API,
zero-copy transfer representation, transactional publication model, retry
policy, result-retention policy, process boundary, ASP.NET adapter, or graph
facility. Those mechanisms must preserve the same explicit admission semantics
if later justified.
