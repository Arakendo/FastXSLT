# Private Template-Dispatch Slice

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Golden | `corpus/golden/template-dispatch` |
| Decision pressure | AR-0001 next semantic family and AR-0007 navigation needs |
| Claim | Private exact-name dispatch evidence; no XSLT version or conformance claim |

Current note: the duplicate-pattern rejection recorded by this historical
checkpoint was superseded by the XSLT 3.0 use-last and next-match evidence in
`conflict-resolution-1202c`. The original boundary below remains the evidence
available on the checkpoint date.

## Implemented boundary

The existing reference compiler now retains one required root template and a
list of compiled exact unprefixed element-name template rules. The existing
runtime executes `xsl:apply-templates` with an explicit relative child-name
path, preserves selected source document order, finds the exact expanded-name
rule, and executes that rule with the selected source node as its dynamic
context.

Compiled rules contain only stylesheet-derived state. Selected source nodes,
request identity, invocation controls, results, and work accounting remain in
the transformation invocation. The change extends the same reference backend
and transform-set path used by the original golden; it does not introduce a
second executor.

## Golden result

The source contains two `item` elements with nested names. The root template
selects `catalog/item`; one compiled `match="item"` template produces entries
using the relative `name` value expression. The exact result is:

```xml
<items><entry>alpha</entry><entry>beta</entry></items>
```

The ordering is source document order and is unrelated to transform-set request
completion order.

## Unsupported boundary

This slice deliberately rejects or does not admit:

- duplicate match patterns requiring priority/declaration-order semantics;
- template or apply-template modes;
- predicates, unions, absolute paths, wildcards, namespaces, and generalized
  match-pattern grammar;
- absent `select` and built-in template rules;
- named templates, parameters, imports/includes, and secondary documents.

Unexpected attributes on the admitted XSLT instructions fail visibly as
unsupported rather than being ignored. Focused tests cover the successful
compiled shape, exact execution result, duplicate-pattern rejection, and mode
rejection.

## Result and limitations

The workspace now has 48 tests: 47 pass and one manual accounting-cost probe is
ignored by default. This is enough to make template dispatch the second private
semantic family. It does not satisfy AR-0001's intended-consumer requirement,
establish priority or built-in-rule semantics, or justify a public API.
