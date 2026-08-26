# Private XPath Boundary Expansion

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Decision pressure | M2 XPath parsing/evaluation and AR-0004 failure provenance |
| Scope | Private relative unprefixed child-name path experiment |
| Claim | Boundary evidence only; no XPath version or conformance claim |

## Question

Can the private XPath slice distinguish its deliberately small supported grammar
from invalid input without rejecting valid name punctuation, and can its current
evaluator preserve the semantic ordering and expanded-name behavior required by
the implemented XSLT instructions?

## Experiment

The focused tests now cover:

- relative child paths containing the ASCII NCName punctuation `.`, `-`, and
  `_`;
- invalid ASCII child names separately from valid-but-unimplemented descendant,
  context-step, and non-ASCII name behavior;
- unchanged logical source location across invalid and unsupported failures;
- context-item selection;
- ordered selection of repeated unnamespaced children;
- exclusion of a same-local-name element in a non-empty namespace; and
- an empty result when no child matches.

The parser no longer rejects `item.name` merely because the expression contains
a dot. Non-ASCII names are conservatively classified as unsupported until an
accepted standards profile and a standards-capable name classifier define the
required editions; the private ASCII helper does not claim they are malformed.
Likewise, `.` remains supported only as the complete context-item expression,
while `catalog/..` remains visibly outside this slice.

## Result

The parser and evaluator retain the current private grammar while making the
invalid-versus-unsupported boundary more honest. Evaluation returns matching
unnamespaced elements in document order, ignores a namespaced element with the
same local name, and returns an empty sequence for a missing child.

This evidence does not implement XPath tokenization, general expressions,
Unicode NCName validation, namespaces in path expressions, axes, predicates,
functions, operators, or sequences. Those features remain standards-directed
work after AR-0001 selects the initial profile.
