# OASIS XSLT 1.0 Static Text-Tree Sequences -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

FastXSLT already represented an XSLT 1.0 content-built variable containing one
literal text node as an invocation-owned temporary document. Splitting the same
static value across multiple `xsl:text` constructors fell through to the
literal-element-only constructor and reported `FXST1015`, even though no new
dynamic construction semantics were required.

One Microsoft case exposed a second bounded composition after that repair:
`string-length(string($variable)) * positive-integer`, where the variable holds
the static temporary text tree. The engine already had controlled XSLT 1.0
variable string conversion and codepoint length semantics, but did not compose
that result with the existing multiplication operation.

## Repair

Under XSLT 1.0 static context only, a local or global content-built binding made
entirely from literal text and `xsl:text` instructions is folded into one
temporary text node. Each `xsl:text` still uses the ordinary validator, so
`disable-output-escaping="yes"` remains explicitly unsupported rather than
being erased by folding.

The compiler also admits the exact nested string-length-times-positive-integer
shape. Runtime evaluation uses the existing charged variable string-value and
Unicode-codepoint length operation, charges the multiplication, checks integer
overflow, and emits the result through the ordinary bounded text path. This
does not select general function composition or arithmetic over arbitrary
expressions.

## Corpus result

The `FXST1015` initialization frontier falls from 40 to 36. Three Lotus output
cases now stop at their real `disable-output-escaping="yes"` boundary instead
of the generic constructor boundary. Microsoft `BVTs_bvt085` initializes and
executes, then remains a visible XML mismatch because the archival expected
file contains replacement question marks where the stylesheet and actual UTF-8
result contain supplementary Unicode codepoints.

Accordingly, initialized cases rise from 1,871 to 1,872, executed-successfully
cases rise from 1,672 to 1,673, and visible mismatches rise from 120 to 121. The
exact-result lower bound remains 1,523. No comparator-unsupported outcome,
expected-error disposition, or panic is added. The mismatch is not credited as
a pass and its expected bytes are not rewritten.

## Verification

A focused runtime test covers local and global multi-instruction text trees,
variable conversion, codepoint length, multiplication, and bounded output. The
complete local measurement conserves all 3,173 identities: 1,872 initialize,
1,673 execute successfully, 1,523 compare exactly, 121 remain visible
mismatches, and 23 reach comparator-unsupported outcomes. Expected-error
accounting remains 403 initialization observations, 21 execution observations,
and four doubt-annotated unexpected successes.
