# XSLT30 `conflict-resolution-1602`–`1603` Document-Element Pattern Priority

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Cases: `conflict-resolution-1602`, `conflict-resolution-1603`
- Stylesheets: `conflict-resolution-1602.xsl`, `conflict-resolution-1603.xsl`
- Source: inline `conflict-resolution-16` environment

## Representation and execution

The pattern compiler retains the exact admitted forms as one typed
`DocumentElement` pattern:

- `document-node(element(doc))` stores the expanded element name and default
  priority `0`;
- `document-node(element(*))` stores wildcard document-element applicability
  and default priority `-0.5`.

Runtime matching first requires a document node, then performs a bounded,
charged scan to its element child. Exact-name matching compares expanded names;
the wildcard form requires only element kind. Duplicate pattern/mode shapes
with distinct exact priorities reach ordinary selection; equal-rank duplicates
remain rejected. The more general next-match use of that rule is evidenced by
`conflict-resolution-1201`.

## Results

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1602` | `<out>big</out>` | semantically equal | passed |
| `conflict-resolution-1603` | `<out>big</out>` | semantically equal | passed |

For `1602`, explicit `0.1` outranks the exact-name default `0` and explicit
`-0.1`. For `1603`, explicit `-0.4` outranks wildcard default `-0.5` and
explicit `-0.6`.

## Claim boundary

This evidence admits ASCII unprefixed exact-name and wildcard element tests
nested in `document-node()` within one stylesheet module. It does not admit
typed/schema-aware element tests, prefixed or EQName forms, other document-node
content tests, equal-rank duplicate-pattern policy, include/import/package
precedence, or ambiguity recovery behavior.
