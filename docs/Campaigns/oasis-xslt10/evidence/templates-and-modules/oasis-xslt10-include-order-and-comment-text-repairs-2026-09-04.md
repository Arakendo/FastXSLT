# OASIS XSLT 1.0 Include Order and Comment-Text Repairs

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Triggers | `Lotus/impincl_impincl06#1` and `Lotus/whitespace_whitespace21#1` |
| Input baseline | 462 definite unchanged XML passes; 31 comparison mismatches |
| Result | 464 definite unchanged XML passes; 29 comparison mismatches |
| Disposition | Shared compiler repairs; not an XSLT 1.0 conformance claim |

## Include declaration order

`impincl06` has two included modules, each with one imported leaf. The principal
stylesheet declares its competing `title` template after both `xsl:include`
declarations. Included declarations have the same import precedence as the
including module, so the later principal declaration must win the equal-priority
conflict.

The module compiler previously appended included matched templates after every
principal matched template. That retained imported precedence but lost the
include declaration's textual position, causing the included `title` rule to
win. Included matched-template sequences are now spliced at their include
declaration position relative to declarations from the including resource.
The existing complete XSLT30 `decl/include` denominator remains green, including
the inverse case where a later include must win a conflict with an earlier
principal declaration.

## Text runs separated by ignored comments

`whitespace21` contains literal text separated by stylesheet comments. Comments
and processing instructions in a sequence constructor do not become result
nodes, and removing them makes the adjacent text segments one logical text run
for whitespace stripping. The compiler previously examined each parser text
node independently and discarded a whitespace-only segment after an ignored
comment even though the surrounding run contained non-whitespace text.

The sequence compiler now treats text, comments, and processing instructions as
one run for the decision to retain whitespace-only text. Elements remain run
boundaries, and an all-whitespace run such as the content around an ignored
comment inside an otherwise empty literal element is still stripped.

Focused production-path tests cover both repairs. The unchanged local suite
cases then move from comparison mismatch to pass.

## Measurement

The complete hash-verified local sweep reports:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| XML comparison passes | 462 | 464 | +2 |
| XML comparison mismatches | 31 | 29 | -2 |
| XML comparator gaps | 16 | 16 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **464 / 2,742 = 16.92%** of standard-operation
cases and **464 / 3,173 = 14.62%** of the complete catalog. Raw comparison
mismatches still require individual semantic-versus-comparator classification.

## Boundaries

This work does not select general import/include graph support, broader
precedence depth, a legacy compiler, output-indentation comparison rules, or an
XSLT 1.0 compatibility claim. It preserves the existing bounded dependency
graph and shared modern runtime.
