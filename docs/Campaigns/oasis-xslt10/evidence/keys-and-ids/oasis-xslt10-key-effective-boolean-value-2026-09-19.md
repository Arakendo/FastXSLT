# OASIS XSLT 1.0 Key Effective Boolean Value

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an already typed XSLT 1.0 `key()` lookup participate directly in a
conditional test without adding a second key evaluator or weakening the
existing resource controls?

## Implemented slice

Yes. In XSLT 1.0 compatibility mode, a structurally complete `key()` call used
as an `xsl:if` or `xsl:when` test now compiles to a typed key-lookup
effective-boolean-value plan. Runtime evaluation delegates to the existing
charged key selector and returns true exactly when its node-set is non-empty.

The lookup retains the existing rules for static or admitted dynamic key names,
atomic/node-set lookup values, document order, predicates, path tails,
diagnostics, budgets, and cancellation. Modern static context remains
explicitly unsupported for this private compatibility form. This does not
admit general function-call EBV, a second key index, or a public node-set type.

## Corpus result

The unchanged Lotus `impincl17` case now evaluates `key('annid', @id)` in an
imported template and matches its expected result exactly. Against the
conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,051 | 2,052 | +1 |
| Initialization failures | 1,084 | 1,083 | -1 |
| Executed successfully | 1,949 | 1,950 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,820 | 1,821 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

A focused regression exercises both non-empty and empty key results from two
source items and proves the exact modern expression remains unsupported. The
unchanged corpus case adds imported-module composition, attribute-derived
lookup values, and `xsl:choose` coverage. Full corpus accounting remains
conserved without a new mismatch, execution failure, or panic.
