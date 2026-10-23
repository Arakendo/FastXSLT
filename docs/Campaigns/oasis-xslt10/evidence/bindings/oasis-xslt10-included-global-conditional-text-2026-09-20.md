# OASIS XSLT 1.0 Included Global Conditional Text

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can an XSLT 1.0 global temporary text value choose between two static branches
using an included global dependency, without admitting a general global
instruction executor?

## Method

- Compile one exact `xsl:choose` shape: a single `xsl:when` comparing a direct
  variable to a string literal, followed by `xsl:otherwise`, with one static
  `xsl:text` in each branch.
- Include the referenced variable in global topological ordering.
- Repeat dependency ordering after an included stylesheet program is merged,
  rejecting cross-module dependency cycles at the include location.
- Evaluate the comparison through controlled global string conversion and
  materialize one charged invocation-owned temporary text node.
- Add a focused forward-dependency regression and run the hash-verified OASIS
  CD04 measurement against unchanged Lotus `variable70`.

## Result

The focused regression orders `$input` before `$result` and produces
`<out>matched</out>`. The unchanged OASIS case resolves `$foo` from its included
module and exactly matches `<out>the value of bar is bar</out>`.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,099 | 2,100 | +1 |
| Initialization failures | 1,036 | 1,035 | -1 |
| Executed successfully | 2,012 | 2,013 | +1 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,881 | 1,882 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The exact compatibility lower bound is now `1,882 / 3,173 = 59.31%`. The
generic `FXST1015` initialization frontier falls from 12 to 11 cases. This is
local compatibility evidence against the hash-verified, non-redistributed
OASIS CD04 archive; it is not a broad conformance claim.

## Boundaries

- The test is one direct-variable equality against one string literal.
- Each branch contains exactly one static `xsl:text` instruction.
- General boolean expressions, multiple `xsl:when` branches, nested sequence
  constructors, and global template execution remain unsupported.
- Module merging changes dependency order only; compiled values remain
  stylesheet-derived and materialized dynamic values remain invocation-owned.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_conditional_text_uses_an_ordered_global_dependency
./scripts/measure-oasis-xslt10.ps1 -TraceCase variable_variable70
./scripts/verify.ps1
```
