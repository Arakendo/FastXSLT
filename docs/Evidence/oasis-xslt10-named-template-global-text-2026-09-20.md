# OASIS XSLT 1.0 Named-Template Global Text

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can a bounded XSLT 1.0 global temporary-text constructor obtain its value from
a simple named template without introducing a general global template executor
or retaining invocation state in compiled artifacts?

## Method

- Recognize a global content constructor containing exactly one
  parameter-free `xsl:call-template`.
- Resolve that call only to a named template in the same stylesheet module.
- Require the named template body to contain exactly one `xsl:value-of` whose
  selection is a direct global-variable reference.
- Lower the reference into the existing typed global temporary-text plan so it
  participates in normal dependency ordering, invocation-owned conversion,
  XDM-node charging, and cancellation.
- Retain ordinary named-template compilation and reference validation as the
  independent validation path.
- Add a focused runtime regression and measure unchanged Lotus `variable31`
  from the hash-verified OASIS XSLT 1.0 CD04 archive.

## Result

The focused regression compiles `$input` before `$result` and serializes
`<out>value</out>`. Unchanged Lotus `variable31` resolves the top-level
parameter through `set-tata` and exactly matches `<out>titi</out>`.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,100 | 2,101 | +1 |
| Initialization failures | 1,035 | 1,034 | -1 |
| Executed successfully | 2,013 | 2,014 | +1 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,882 | 1,883 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The exact compatibility lower bound is now `1,883 / 3,173 = 59.34%`. The
generic `FXST1015` initialization frontier falls from 11 to 10 cases. This is
local compatibility evidence against the hash-verified, non-redistributed
OASIS archive; it is not a broad conformance claim.

## Boundaries

- The call has no parameters and the named template has no template parameters.
- The named template contains only one direct-variable `xsl:value-of`.
- Resolution is confined to the current stylesheet module; imported and
  included named-template precedence remains outside this specialization.
- Source-focus-dependent instructions, mixed content, recursion, and general
  template execution in global constructors remain unsupported.
- The compiled generation retains only stylesheet-derived identity and a typed
  variable reference. Materialized text remains invocation-owned.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_text_can_use_a_bounded_named_template_constructor
./scripts/measure-oasis-xslt10.ps1 -TraceCase variable_variable31
./scripts/verify.ps1
```
