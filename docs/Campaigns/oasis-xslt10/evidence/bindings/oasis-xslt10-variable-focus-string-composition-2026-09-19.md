# OASIS XSLT 1.0 Variable/Focus String Composition

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a recursive XSLT 1.0 transformation use a numeric template parameter to
partition sequence focus and then consume its constructed string parameters
without introducing a general XPath evaluator or a legacy runtime?

## Implemented slice

Yes. Three compile-selected typed forms were added under XSLT 1.0 static
context:

- `position() mod $variable` converts the bound value through the existing
  XPath 1.0 numeric path and applies numeric effective boolean value;
- `string-length($variable)` converts the bound value through the existing
  XSLT 1.0 string path and applies numeric effective boolean value;
- `contains()`, `starts-with()`, `substring-before()`, and
  `substring-after()` accept one bound variable and one string-literal operand.

The runtime retains dynamic focus and variable ownership in the invocation,
charges conversion and function work, and stores only variable names, literals,
and source locations in compiled state. Modern static context continues to
reject these specialized compositions. No second focus, variable-conversion,
or string-function implementation was introduced.

## Corpus result

The unchanged Microsoft `Variables__84438` case advanced successively through
the modulo, variable substring, and variable string-length boundaries and now
matches its XML-semantic expected result exactly. Against the conserved
3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,046 | 2,047 | +1 |
| Initialization failures | 1,089 | 1,088 | -1 |
| Executed successfully | 1,944 | 1,945 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,815 | 1,816 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

Focused runtime tests cover modulo focus with a numeric template parameter,
variable string conversion from a constructed temporary tree, all four binary
string-function kinds, empty delimiters, and variable string-length EBV.
Cross-version regressions retain explicit modern unsupported outcomes. The
complete corpus measurement conserves every catalog identity and adds no
mismatch, execution failure, or panic.
