# OASIS XSLT 1.0 Key Match Patterns

Date: 2026-09-23  
Status: Verified semantic and compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing bounded XSLT 1.0 `xsl:key` and `key()` machinery support
static `key()` match patterns, including a relative child or descendant tail,
without introducing a second index, ambient resource access, or an uncharged
source scan?

## Implemented slice

Yes. XSLT 1.0 template compilation now retains a typed static `key()` match
pattern when both arguments are string literals. An optional location-path tail
uses the existing bounded key-tail grammar. Dynamic key names/values and broader
match-pattern syntax remain outside this slice.

Execution evaluates the declared key definitions against the current source
document, charges source traversal and key-use evaluation, restores document
order for a tail, and records the selected node identities in the existing
bounded invocation-owned document-rooted membership cache. The complete scan
remains the fallback if that cache declines admission. No state crosses an
invocation, source, snapshot, worker, or generation boundary.

## Corpus result

Six unchanged cases move from initialization rejection directly to exact XML
comparison passes:

- `Lotus/idkey_idkey03#1`;
- `Lotus/idkey_idkey35#1`;
- `Lotus/idkey_idkey46#1`;
- `Lotus/idkey_idkey47#1`;
- `Lotus/idkey_idkey48#1`; and
- `Microsoft/XSLTFunctions_KeyFuncTestDescendantsNodeset#1`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,200 | 2,206 | +6 |
| Executed successfully | 2,147 | 2,153 | +6 |
| Expected-result XML matches | 1,999 | 2,005 | +6 |
| XML comparison mismatches | 64 | 64 | 0 |
| Execution failures | 53 | 53 | 0 |
| Comparator unsupported | 75 | 75 | 0 |

The strict complete-catalog lower bound becomes
`2,005 / 3,173 = 63.19%`.

## Boundaries

- This is a static XSLT 1.0 match-pattern form, not a general function-call
  pattern evaluator.
- It reuses declared key definitions and the ordinary charged key-use/path
  machinery; it does not add a global or cross-invocation key index.
- Membership caching remains bounded, private, and invocation-owned.
- Temporary-tree key matching, dynamic key-pattern arguments, DTD `id()`, and
  live resource acquisition are not admitted.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_static_key_match_patterns
cargo test -p fastxslt --all-features xslt10_key_match_patterns
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/idkey_idkey03#1'
./scripts/verify.ps1
```
