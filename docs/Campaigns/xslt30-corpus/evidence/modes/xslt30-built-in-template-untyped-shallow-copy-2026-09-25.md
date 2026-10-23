# XSLT30 Built-In-Template Untyped Shallow Copy

Date: 2026-09-25  
Suite: W3C XSLT 3.0 test suite  
Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`  
Case: `misc/built-in-templates/built-in-templates-0301`

## Result

The unchanged non-schema-aware case passes through the normal resource,
compilation, transform-set, and native assertion path. The complete six-case
denominator now records five passes and one explicit schema-aware profile
exclusion.

## Admitted semantics

- A modern local variable without `select` or `as` may execute the admitted
  single-`xsl:apply-templates` sequence constructor and materialize the result
  as an invocation-owned temporary document node.
- Effective `expand-text="yes"` compiles the case's exact text value template
  into the existing typed value-expression path.
- `$copied//empty instance of element(empty, xs:untyped)` performs a bounded,
  work-charged traversal of the invocation-owned temporary tree and requires
  exactly one matching expanded-name element.
- FastXSLT's non-schema-aware source and temporary element nodes are untyped by
  construction. This case therefore does not introduce schema annotations or
  claim schema-aware processing.
- Existing shallow-copy and `xsl:next-match` behavior preserve the copied
  element while the overriding template suppresses the original text and adds
  its sibling result element.

## Boundaries retained

- Other text value template forms remain outside this narrow tranche and retain
  their prior compiler behavior.
- Other modern local-variable sequence-constructor shapes remain outside this
  tranche; their prior diagnostic behavior is preserved.
- General sequence-type parsing and schema-derived type annotations are not
  inferred.
- The schema-aware sibling `built-in-templates-0302` is classified as excluded
  by profile and remains unexecuted.

## Verification

- The unchanged W3C case matches its native `/out='true'` assertion.
- A focused runtime regression proves that an ordinary XSLT 3.0 local sequence
  constructor materializes and exposes its document-node string value.
- The conserved overlay names every case and distinguishes the five passes from
  the one schema-aware exclusion.
