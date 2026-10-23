# XSLT30 `template-002/003` Node-Kind Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identities | test set `template`, cases `template-002` and `template-003` |
| Dependencies | `XSLT10+` |
| Result assertions | native `assert-xml` |
| Outcome | Both passed through the private reference path |

## Executed behavior

The metadata-driven test resolves and imports each case's principal source and
stylesheet through a bounded sealed snapshot, then compares the serialized
result element name and string value with the native `assert-xml`.

The two cases extend the node-kind and mode seam with:

- `processing-instruction()` child selection and pattern matching;
- `node()` child selection and pattern matching across text and element nodes;
- named-mode isolation from deliberately failing default-mode rules;
- exact element-pattern precedence over a generic `node()` rule; and
- built-in document dispatch when no explicit `/` template exists.

The PI case exposed a parser-adapter issue: `quick-xml` retained the XML syntax
whitespace separating the PI target from its data. FastXSLT now removes that
separator while constructing the owned parser event, so the XDM PI string value
matches the standards case. A focused parser test retains this normalization.

The `node()` case also establishes that the child-pattern form does not match
the root document node. Treating it as a universal node predicate would execute
the deliberately failing default-mode rule before built-in document dispatch.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 4 |
| Engine unsupported and not run | 2 |
| Total | 6 |

`template-001`, `template-002`, `template-003`, and `template-006` pass.
Attribute selection/patterns in `template-004` and named-template recursion in
`template-005` remain explicitly unsupported.

## Claim boundary

This is evidence for the two native cases and implemented node tests. It is not
a claim of the complete XPath kind-test grammar, pattern grammar, priority
rules, modes, XSLT 1.0, or XSLT 3.0 conformance.
