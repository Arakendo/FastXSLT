# OASIS XSLT 1.0 Static AVT Brace-Escaping Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 724 definite unchanged XML passes; 1,002 initialized cases |
| Result | 728 definite unchanged XML passes; 1,009 initialized cases |
| Disposition | Shared static AVT lexical semantics; not a conformance claim |

## Change

Literal result attributes now recognize doubled left and right braces as the
XSLT attribute-value-template escape for one literal brace. Compilation removes
one brace from each pair and retains the resulting static text in the ordinary
compiled literal-attribute representation. No runtime expression dispatch or
version check is added.

A single or otherwise unpaired brace remains an explicit `FXST1031`
unsupported attribute value template. Dynamic expressions such as `{.}` also
remain unsupported: admitting them correctly requires typed access to the
current node's XDM string value rather than treating the runtime's optional
atomic context value as a substitute.

## Unchanged cases

Four catalog identities move directly to definite XML comparison passes:

- `attribvaltemplate03`, for `{{` becoming `{`;
- `attribvaltemplate04`, for `}}` becoming `}`; and
- both catalog identities for `attribvaltemplate12`, for
  `{{font:helvetica}}` becoming `{font:helvetica}`.

Three additional cases now initialize and reach already-visible execution
failures. They are not credited as compatibility passes.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,002 | 1,009 | +7 |
| Execution succeeded | 805 | 809 | +4 |
| XML comparison passes | 724 | 728 | +4 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 197 | 200 | +3 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **728 / 2,742 = 26.55%** of standard-operation
cases and **728 / 3,173 = 22.94%** of the complete catalog.

## Verification

A focused compiler test covers paired static braces, rejects a dynamic
expression, and rejects an unbalanced sequence. The complete local OASIS sweep
confirms all four intended identities pass without a new mismatch, comparator
gap, unexpected success, or panic. The three newly exposed execution failures
remain separately counted rather than being mistaken for progress.
