# OASIS XSLT 1.0 Qualified Absolute Match Path

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can an absolute multi-step match pattern mix unprefixed and namespace-qualified
element names while preserving XPath 1.0's separation between literal-result
default namespaces and unprefixed XPath names?

## Method

- Extend the existing qualified child-path parser with a document-node origin
  for a single leading slash.
- Route mixed qualified multi-step match patterns through that parser while
  resolving prefixes from stylesheet static context.
- Keep unprefixed XPath 1.0 steps in no namespace unless an explicit
  `xpath-default-namespace` contract applies.
- Add a complete transform whose literal results have a default namespace but
  whose selection and absolute match use source names independently.
- Recheck unchanged Microsoft `BVTs_bvt075`.

## Result

The complete transform selects and matches the namespaced source element while
constructing results in an unrelated default namespace. The unchanged corpus
case advances past `/bookstore/my:book` and now stops at its independent
`@xmlns:*` namespace-node pattern boundary.

The conserved counters do not change for this tranche; it is foundational
breadth evidence and does not claim the corpus case as a pass.

## Boundaries

- The admitted form contains child element name tests only and at least one
  qualified step; predicates, namespace axes, and qualified wildcards remain
  governed by their existing typed families.
- A leading slash establishes document-node origin. Relative qualified paths
  retain their existing ancestor-suffix match behavior.
- This does not represent namespace nodes as attributes or admit
  `@xmlns:*`.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_default_result_namespace_does_not_change_qualified_path_names
./scripts/measure-oasis-xslt10.ps1 -TraceCase bvt075
./scripts/verify.ps1
```
