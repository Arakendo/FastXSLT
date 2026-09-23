# OASIS XSLT 1.0 Dynamic Computed-Attribute Namespace

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can path-valued XSLT 1.0 `xsl:attribute` namespace AVTs share the compiled
namespace-value plan introduced for computed elements while retaining the
prefix binding required by the semantic result tree?

## Method

- Generalize the private namespace value from an element-only type to a static
  string or compiled location path shared by computed constructors.
- Keep path-valued namespace AVTs limited to one braced location path in an
  XSLT 1.0 static context.
- Evaluate the path with ordinary work charging and XSLT 1.0 first-node string
  conversion.
- Preserve the element and attribute QName rules as separate runtime
  validators.
- When a runtime attribute namespace lacks a prefixed binding on its owning
  result element, generate a deterministic private `nsN` binding after all
  attributes have been assembled. Reuse the immutable stylesheet-derived
  namespace slice unchanged when no dynamic binding is required.
- Include the compiled namespace path in known prepared-program retention
  accounting.

## Result

The focused regression constructs one dynamically named attribute in a
source-derived namespace and one static-name attribute whose source-derived
namespace is empty. It proves the generated prefix is retained for
serialization and the empty namespace produces an ordinary unprefixed
attribute. Controls retain unsupported diagnostics for non-path XSLT 1.0 AVTs
and for the same path form in an XSLT 3.0 static context.

The unchanged OASIS case `Lotus/attribset_attribset16#1` leaves `FXST1061`,
executes, and compares XML-semantically exactly. The complete sweep now
initializes 2,194 cases, reports 941 initialization failures, executes 2,137
successfully, and reports 57 execution failures. Exact XML-semantic matches
rise from 1,985 to 1,986; mismatches remain 67 and comparator-unsupported
results remain 75. The strict compatibility lower bound is
`1,986 / 3,173 = 62.59%`.

## Boundaries

- General namespace AVTs and modern expression semantics remain unsupported.
- Runtime prefix generation is result-owned. It does not mutate or extend the
  immutable stylesheet-derived namespace slice shared across invocations.
- The namespace expression acquires no resource authority and consumes only
  the current invocation's admitted source.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_path_valued_attribute_namespace_uses_first_node_string_value
cargo test -p fastxslt --all-features computed_attribute_retains_static_namespace_and_literal_text
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/attribset_attribset16#1'
./scripts/verify.ps1
```
