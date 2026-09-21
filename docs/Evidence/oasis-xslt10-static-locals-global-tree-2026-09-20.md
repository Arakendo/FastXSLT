# OASIS XSLT 1.0 Static Locals in Global Trees

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can a global XSLT 1.0 temporary tree use lexical local variables without
introducing mutable local frames into compiled state or a general global
instruction executor?

## Method

- Add a private compiler owner for bounded XSLT 1.0 global-constructor
  specializations rather than adding the responsibility to the stylesheet
  compiler's main source unit.
- Admit sequential local `xsl:variable` declarations only when each initializer
  is one string literal selected through `select`.
- Preserve lexical declaration order and reject duplicate local names.
- Fold a visible local value into a later variable-only literal-result AVT or a
  direct-variable `xsl:value-of` nested in an otherwise static literal tree.
- Lower the result to the existing immutable constructed-node representation,
  retaining ordinary result namespaces and invocation-owned temporary-tree
  materialization.
- Add a focused runtime regression and measure the unchanged Microsoft
  `Variables_GlobalVarHaveLocalVarDefinedWithin` case from the hash-verified
  OASIS XSLT 1.0 CD04 archive.

## Result

The focused regression independently folds an attribute value and element text
from prior local declarations. The unchanged OASIS case builds three global
temporary trees and exactly matches its expected result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,101 | 2,102 | +1 |
| Initialization failures | 1,034 | 1,033 | -1 |
| Executed successfully | 2,014 | 2,015 | +1 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,883 | 1,884 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The exact compatibility lower bound is now `1,884 / 3,173 = 59.38%`. The
generic `FXST1015` initialization frontier falls from 10 to 9 cases. This is
local compatibility evidence against the hash-verified, non-redistributed
OASIS archive; it is not a broad conformance claim.

## Boundaries

- Local initializers are string literals; content constructors and dynamic
  expressions remain unsupported in this specialization.
- Only prior lexical bindings are visible. A declaration produces no result
  node and duplicate local names are rejected.
- Consumers are restricted to one variable-only AVT or direct-variable
  `xsl:value-of`; general XPath evaluation is not performed at compile time.
- The result must otherwise be a static literal tree. Source focus, global
  references, template calls, loops, and arbitrary instructions remain outside
  this path.
- Compiled artifacts retain folded stylesheet-derived strings only. Every
  temporary tree remains invocation-owned and charged during materialization.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_tree_can_use_prior_static_local_bindings
./scripts/measure-oasis-xslt10.ps1 -TraceCase Variables_GlobalVarHaveLocalVarDefinedWithin
./scripts/verify.ps1
```
