# OASIS XSLT 1.0 `xsl:key/@use` `concat()` -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic primitive and corpus-frontier movement  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft `Keys_PerfRepro2` stylesheet declares a key whose
`use` expression is
`concat(PlanLevelDescription, ':', BusTeamDescription)`. FastXSLT already had
typed XSLT 1.0 concat and key plans, but key declaration compilation treated
the function call as a location path and rejected the valid expression.

## Change

XSLT 1.0 key declarations now retain a typed concat use expression when every
argument is a string literal or a location path. Key construction evaluates
each path relative to the matched candidate node, applies XPath 1.0 first-node
string conversion, charges the existing XPath work domains, and emits one key
value. Variable and recursive `key()` dependencies remain statically forbidden.

The plan is immutable stylesheet state. At this tranche, evaluation remained a
complete invocation-owned scan with no resource acquisition or mutation of
prepared XDM. The later invocation-index and Muenchian tranche supersedes the
no-cache observation while retaining this scan as its differential oracle.
Sum-path and variable-dependent concat arguments were not admitted by this
slice.

## Corpus result

`Microsoft/Keys_PerfRepro2#1` now passes key-use compilation and reaches its
next independent frontier: the Muenchian grouping predicate
`count(. | key(...)[1]) = 1`. The complete conserved sweep remains:

- 2,212 / 3,173 exact expected-result matches (69.71%);
- 2,422 initialized cases;
- 2,372 successfully executed cases;
- 748 initialization failures and 50 execution failures;
- nine visible XML mismatches and 52 comparator gaps;
- 423 / 431 expected-error credits.

The archive contains no second `xsl:key/@use` concat declaration, so this
standards primitive earns frontier movement rather than pass credit. The later
grouping predicate is not inferred from key construction and remains separately
unsupported.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_key
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys_PerfRepro2#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 12.2, Keys](https://www.w3.org/TR/1999/REC-xslt-19991116#keys)
- [XPath 1.0 section 4.2, String Functions](https://www.w3.org/TR/1999/REC-xpath-19991116/#section-String-Functions)
