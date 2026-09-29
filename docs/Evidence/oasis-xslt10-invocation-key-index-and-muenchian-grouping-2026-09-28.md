# OASIS XSLT 1.0 Invocation-Owned Key Index and Muenchian Grouping -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic slice, differential oracle, and corpus pass  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Microsoft `Keys_PerfRepro3` case applies the standard XSLT 1.0
Muenchian form `count(. | key(...)[1]) = 1` to a 3,534,521-byte source. The
first complete implementation was semantically correct but rebuilt a complete
key lookup for every candidate and exhausted the existing ten-million
`xpath-node-visit` budget. Reusing a key table then exposed a second quadratic
cost: generic evaluation of the relative match pattern
`Firm_Subsidiary_Relationship/Parent` scanned the large sibling set for every
candidate.

## Change

FastXSLT now compiles the exact first-by-key grouping form as a private typed
XSLT selection. Candidate input may be an admitted location path or key lookup;
the grouping predicate retains node identity and document order.

The runtime lazily builds one source-derived key index for each referenced
expanded key name within an invocation. Same-name declarations compose into
the index, each value maps to document-ordered source nodes, and ordinary
`key()` calls reuse it. The index:

- is mutable invocation state, not compiled or prepared-input state;
- is never shared across invocations, workers, snapshots, or generations;
- performs no resource acquisition and does not mutate source XDM;
- records its known retained string/vector payload in test observations; and
- retains the complete charged document scan as a selectable differential
  oracle.

Predicate-free relative paths containing only child-name steps now match by
walking candidate ancestry directly. Paths with predicates, other axes, or
document origins retain the complete evaluator.

## Verification

The focused Muenchian test executes the same stylesheet and source through the
index and complete-scan paths, compares semantic result trees, observes one
index build with nonzero attributable payload, and observes no index retention
on the reference path. Existing key selection tests remain exact.

The unchanged OASIS case now completes within its existing work envelope and
matches its XML reference exactly. The conserved sweep is:

- 2,213 / 3,173 exact XML expected-result matches (69.74%);
- 2,423 initialized cases;
- 2,373 successfully executed cases;
- 747 initialization failures and 50 execution failures;
- nine visible XML mismatches and 52 comparator gaps; and
- 423 / 431 expected-error credits.

`Microsoft/Keys_PerfRepro2#1` also advances past its path-based Muenchian form
and key-selected grouping base. At this evidence checkpoint it stopped
explicitly at a different lookup value,
`concat(PlanLevelDescription, ':', BusTeamDescription)`. The subsequent
[focus-dependent concat key-value tranche](oasis-xslt10-focus-concat-key-value-2026-09-28.md)
admits that typed value shape. Neither tranche infers general key-value
expressions, general XPath union/count compilation, or cross-invocation key
retention.

```powershell
cargo test -p fastxslt --all-features xslt10_key
cargo test -p fastxslt --all-features xslt10_muenchian_key_group
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys_PerfRepro3#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Keys_PerfRepro2#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 12.2, Keys](https://www.w3.org/TR/1999/REC-xslt-19991116#keys)
- [XSLT 1.0 section 5.2, Patterns](https://www.w3.org/TR/1999/REC-xslt-19991116#patterns)
- [XPath 1.0 section 3.3, Node-sets](https://www.w3.org/TR/1999/REC-xpath-19991116/#node-sets)
