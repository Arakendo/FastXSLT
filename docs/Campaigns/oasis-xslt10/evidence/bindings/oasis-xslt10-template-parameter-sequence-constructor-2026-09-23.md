# OASIS XSLT 1.0 Template-Parameter Defaults

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can template-parameter defaults containing instructions or paths rooted at an
earlier parameter reuse existing invocation semantics rather than acquiring a
parameter-specific evaluator?

## Method

- Keep the behavior limited to an XSLT 1.0 static context.
- Compile a non-text `xsl:param` body through the ordinary bounded instruction
  sequence compiler, with a maximum of 64 top-level constructor children.
- Execute the constructor in the template invocation's source context and
  current variable frame.
- Materialize the semantic result as an invocation-owned temporary tree through
  the same charged XDM path already used for content-valued `xsl:with-param`.
- Include the compiled instruction sequence in known prepared-program retention
  accounting.
- Compile `$variable/relative-path` through the shared XSLT 1.0 variable-rooted
  path plan. Normalize `$variable//descendant` to context-descendant navigation
  rather than accidentally treating the second slash as a document-root origin.
- Evaluate defaults sequentially in the invocation frame so a later parameter
  can consume source nodes bound by an earlier parameter.
- Retain the prior explicit `FXST1032` boundary for modern static contexts.

## Result

The focused control constructs `<author>Andy</author>` from a literal result
element and `xsl:value-of`, copies the temporary tree, and proves the equivalent
XSLT 3.0 stylesheet remains unsupported by this compatibility-only path.

The unchanged OASIS case `Microsoft/BVTs_bvt027#1` leaves `FXST1032`, executes,
and compares XML-semantically exactly. Its raw serialization differs only in
indentation and attribute placement that the suite's XML comparator treats as
insignificant.

`Microsoft/BVTs_bvt091#1` also leaves `FXST1032` and executes its chained
`$abs/bookstore[1]`, `$bookstore//magazine`, and later variable-rooted defaults.
It remains an output mismatch because source whitespace copied by built-in text
rules differs from the archival Microsoft expectation; that later behavior is
not credited as a pass or silently changed here.

The complete sweep now initializes 2,188 cases, reports 947 initialization
failures, executes 2,131 successfully, and reports 57 execution failures.
Exact XML-semantic matches rise from 1,980 to 1,981; mismatches rise from 66 to
67 as the second case reaches comparison. The strict compatibility lower bound
is `1,981 / 3,173 = 62.43%`.

## Boundaries

- This is an XSLT 1.0 compatibility path, not a general modern sequence-type
  implementation for parameter defaults.
- The constructor is invocation-owned and is neither retained in prepared input
  nor shared across invocations, workers, snapshots, or generations.
- The existing instruction engine owns semantics, diagnostics, cancellation,
  and work charging; no second temporary-tree executor is introduced.
- Source whitespace behavior exposed after parameter evaluation remains a
  separate semantic question; this tranche does not infer stripping from one
  archival expected result.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_template_parameter_content_builds_an_invocation_owned_temporary_tree
cargo test -p fastxslt --all-features xslt10_template_parameter_default_can_navigate_an_earlier_source_node_parameter
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt027#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt091#1'
./scripts/verify.ps1
```
