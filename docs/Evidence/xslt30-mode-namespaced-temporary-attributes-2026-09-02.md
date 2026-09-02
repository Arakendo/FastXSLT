# XSLT30 namespaced temporary attributes and mode policies -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `mode-0016` case. A source-free
invocation constructs a namespaced temporary element with one unqualified and
one namespace-qualified literal attribute, applies the three declared
on-no-match policies, and matches the native XML assertion.

## Implemented semantics

- Literal result and temporary-tree attributes retain their expanded names;
  namespace-qualified attributes are no longer rejected merely because the
  private representation does not retain lexical prefix identity.
- The temporary element retains the literal namespace bindings needed by
  shallow-copy serialization, including otherwise unused bindings required by
  the unchanged expected result.
- The unchanged `@*` rule in `mode="#all"` selects both temporary attributes.
- The exact `{local-name()}` literal attribute value template reads the active
  source or temporary node's expanded-name local part. It does not introduce
  general function dispatch or string-expression evaluation.
- Shallow-copy and shallow-skip apply the attribute rule in attribute order;
  text-only-copy ignores attributes and copies the temporary text child.

## Boundary

This slice retains namespace bindings as element metadata but does not expose
namespace nodes for selection, matching, identity, or standalone copying.
General attribute value templates, prefix-sensitive functions, computed
namespace constructors, and schema typing remain unsupported.

## Denominator movement

The complete 169-case `attr/mode` denominator moves from 81 to 82 passes and
from 43 to 42 visible default not-run cases; its 45 profile exclusions are
unchanged. Across the eleven conserved XSLT30 denominators, the total moves
from 401 to 402 passes and from 76 to 75 visible default not-run cases, with
three engine-unsupported cases and 51 profile exclusions unchanged.
