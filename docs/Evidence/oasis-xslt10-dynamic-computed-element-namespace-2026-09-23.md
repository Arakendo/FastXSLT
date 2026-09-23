# OASIS XSLT 1.0 Dynamic Computed-Element Namespace

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can an XSLT 1.0 `xsl:element` namespace AVT whose expression is an admitted
location path reuse the existing path evaluator and QName constructor without
turning arbitrary AVTs into a second expression engine?

## Method

- Keep the new form limited to an XSLT 1.0 static context and exactly one
  braced location-path expression.
- Compile the expression into the shared `LocationPath` representation.
- Evaluate it with the ordinary charged path evaluator and XSLT 1.0 first-node
  string conversion.
- Feed the resulting namespace string into the existing dynamic QName
  validation and namespace-binding path.
- Route a static `name` through the dynamic constructor only when its namespace
  is actually dynamic; retain the existing static constructor fast path for a
  static name and namespace.
- Include both static and path-valued namespace plans in prepared-program
  retention accounting.

## Result

The focused regression constructs both an unprefixed static name and a
path-valued prefixed name using the namespace URI selected from the source.
Invalid lexical QNames and reserved/empty namespace combinations continue to
use the existing structured dynamic errors.

The complete 3,173-case sweep removes all five `FXST1045` initialization
failures. Four unchanged cases compare XML-semantically exactly:

- `Lotus/lre_lre19#1`
- `Lotus/namespace_namespace52#1`
- `Lotus/namespace_namespace53#1`
- `Lotus/namespace_namespace54#1`

`Microsoft/BVTs_bvt055#1` also initializes and executes, but its result uses
namespace names containing spaces and remains in the comparator-unsupported
bucket rather than being credited as a pass.

The complete sweep now initializes 2,193 cases, reports 942 initialization
failures, executes 2,136 successfully, and reports 57 execution failures.
Exact XML-semantic matches rise from 1,981 to 1,985; mismatches remain 67 and
comparator-unsupported results rise from 74 to 75. The strict compatibility
lower bound is `1,985 / 3,173 = 62.56%`.

## Boundaries

- This does not admit general namespace AVTs, functions such as
  `namespace-uri()`, or the corresponding modern XSLT expression surface.
- The namespace expression acquires no resource authority and uses only the
  current invocation's already-admitted source tree.
- A static computed element retains its current pre-resolved representation;
  this compatibility slice adds runtime work only when the namespace is
  source-derived.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_xsl_element_namespace_path_uses_the_first_node_string
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/namespace_namespace54#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt055#1'
./scripts/verify.ps1
```
