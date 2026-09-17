# OASIS XSLT 1.0 Attribute-Set Global Variable Scope -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a static attribute-set plan admit one global atomic-variable value without
allowing a same-named local binding at the consuming instruction to capture it?

## Implemented slice

Yes. A computed attribute inside a top-level `xsl:attribute-set` may now contain
one `xsl:value-of` that references a declared top-level variable or parameter.
Compilation records that reference as explicitly global rather than reusing the
ordinary lexical-variable value kind. Runtime materialization therefore reads
the invocation's evaluated global frame directly, including a host-supplied
global parameter value, and cannot observe a same-named template-local binding.

Undeclared names are rejected during compilation with `XPST0008`. Other dynamic
attribute-set value constructors remain explicitly unsupported; this slice does
not introduce a general constructor executor or runtime attribute-set registry.

## Corpus result

Unchanged Lotus `attribset44` becomes an exact XML-semantic expected-result
pass. Its global `$foo` has value `correct`; a consuming template shadows that
name with local value `incorrect`; the produced attribute remains `correct`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,817 | 1,818 | +1 |
| Initialization failures | 1,318 | 1,317 | -1 |
| Executed successfully | 1,627 | 1,628 | +1 |
| Execution failures | 190 | 190 | 0 |
| Expected-result XML matches | 1,482 | 1,483 | +1 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,483 / 2,742 = 54.08%` for standard-operation cases and
`1,483 / 3,173 = 46.74%` for the complete catalog.

## Verification

A focused production-lifecycle test uses the same global/local shadowing shape
and asserts the exact serialized result. The unchanged complete sweep conserves
all 3,173 identities and adds no XML mismatch, execution failure, or panic.
