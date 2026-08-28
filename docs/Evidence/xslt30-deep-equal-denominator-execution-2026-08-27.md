# XSLT30 `fn/deep-equal` Denominator Execution

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/fn/deep-equal/_deep-equal-test-set.xml` |
| Cases | `deep-equal-001`, `deep-equal-002` |
| Dependency | `XSLT20+` |
| Outcome | Two selected native passes; no exclusions or hidden cases |

## Executed behavior

FastXSLT reads the complete pinned two-case test-set metadata, retains the
shared inline source environment and each native XML assertion, imports the
referenced stylesheet bytes into a bounded sealed snapshot, and executes both
cases through the ordinary principal-source lifecycle.

The admitted expression slice provides:

- positioned descendant selection for unprefixed elements and comments;
- attribute selection from the positioned element without treating attributes
  as children;
- document-order traversal with each inspected node charged to the XPath-node
  visit budget;
- pairwise sequence-length and item comparison;
- attribute equality by node kind, expanded name, and value; and
- comment equality by node kind and value.

A focused control compares separately owned attribute nodes with equal names
and values. They are deeply equal even though their XDM node identities are
distinct. Another comparison retains equal lexical values under different
attribute names and returns false, preventing string equality from standing in
for node deep equality.

## Claim boundary

This evidence establishes the two native suite cases, not general
`fn:deep-equal`. The parser deliberately accepts only the positioned descendant
attribute and comment selector forms used by this denominator. It does not yet
claim element/document deep equality, namespace-node handling, typed values,
maps, arrays, functions, collations, NaN rules, arbitrary sequences, or the
complete XPath grammar.

The implementation remains a private XPath semantic path. It does not publish
its selector representation, make a conformance percentage claim, or alter the
host resource and invocation contracts.

## QT3 typed-integer extension

The same private function parser now also executes the complete five-case QT3
`fn-deep-equalint2args` group from `fn/deep-equal.xml` at pinned QT3 revision
`83993587711dbd5c18ed846385ec37d079d6e492`. The group covers equal lower-bound
`xs:int` values and unequal lower-, middle-, and upper-bound combinations in
both argument orders. All five native boolean assertions pass.

These source-free expressions parse each `xs:int` constructor into its checked
32-bit numeric value and charge one XPath operation for the comparison; they do
not perform node visits. This extends the evidence from node comparison to one
typed atomic family without claiming arbitrary atomic sequences, numeric type
promotion, cross-type comparison, float/NaN behavior, or the other 258 cases in
the QT3 `fn-deep-equal` test set.

The adjacent complete five-case `fn-deep-equalintg2args` group also passes.
Those `xs:integer` operands include positive and negative 18-digit values that
do not fit in `xs:int`; FastXSLT parses them through a checked `i128` path and
again preserves equal and unequal results in both argument orders. This is an
admitted implementation subset of XPath's arbitrary-precision `xs:integer`,
not evidence that values outside `i128` are supported.

## QT3 exact-decimal extension

The complete five-case `fn-deep-equaldec2args` group from the same pinned QT3
test set also passes. FastXSLT parses each admitted `xs:decimal` lexical value
into a checked `i128` coefficient and a decimal scale, removes insignificant
trailing coefficient zeros, and compares the normalized values exactly. No
binary floating-point conversion participates in these results.

This evidence covers the group's equal negative lower-bound values and its
unequal lower-, middle-, and upper-bound combinations in both argument orders.
It does not establish arbitrary-precision decimal support, values whose
coefficient exceeds `i128`, numeric type promotion, cross-type comparison,
float or double semantics, NaN behavior, or the remainder of the QT3
`fn-deep-equal` test set.

## QT3 bounded-long extension

The adjacent complete five-case `fn-deep-equallng2args` group also passes.
Each admitted `xs:long` constructor is range-checked by parsing through Rust's
signed 64-bit integer representation before comparison. A focused boundary
test accepts `9223372036854775807` and rejects the next positive integer rather
than silently widening an invalid `xs:long` lexical value.

This establishes the five native assertions and the constructor boundary used
by them. It does not add numeric promotion or general constructor-function
semantics, and it does not broaden the bounded `xs:integer` claim above.

## QT3 unsigned-short extension

The complete five-case `fn-deep-equalusht2args` group also passes through an
`xs:unsignedShort` constructor backed by a checked `u16` parse. The group covers
equal lower-bound values plus unequal lower-, middle-, and upper-bound
combinations in both argument orders. Focused controls accept `65535` while
rejecting `-1` and `65536`, so the derived type's value boundary is executable
rather than inferred from the selected fixtures.

This evidence is limited to this constructor and these scalar comparisons. It
does not establish the remaining XML Schema derived-integer families, lexical
whitespace normalization, cross-type promotion, or general sequence equality.

## QT3 negative-integer extension

The adjacent complete five-case `fn-deep-equalnint2args` group also passes.
FastXSLT parses each `xs:negativeInteger` constructor through the checked
`i128` integer path and then enforces the derived type's strictly-less-than-zero
value space. The group covers an equal negative pair and unequal negative
lower-, middle-, and upper-magnitude combinations in both argument orders.

Focused controls accept `-1` while rejecting both `0` and `1`; the constructor
therefore cannot silently collapse into the broader admitted `xs:integer`
subset. This evidence does not establish arbitrary-precision integers, values
outside `i128`, the other derived-integer families, lexical whitespace
normalization, cross-type promotion, or general sequence equality.

## QT3 positive-integer extension

The adjacent complete five-case `fn-deep-equalpint2args` group also passes.
FastXSLT parses each `xs:positiveInteger` constructor through the checked
`i128` integer path and enforces the derived type's strictly-greater-than-zero
value space. The group covers an equal lower-bound pair and unequal lower-,
middle-, and upper-magnitude combinations in both argument orders.

Focused controls accept `1` while rejecting both `0` and `-1`; the constructor
cannot silently collapse into the broader admitted `xs:integer` subset. This
evidence does not establish arbitrary-precision integers, values outside
`i128`, the other derived-integer families, lexical whitespace normalization,
cross-type promotion, or general sequence equality.

## QT3 unsigned-long extension

The adjacent complete five-case `fn-deep-equalulng2args` group also passes.
Each admitted `xs:unsignedLong` constructor is parsed through checked `u64`
before conversion to the evaluator's `i128` comparison representation. The
group covers equal zero values and unequal lower-, middle-, and suite-described
upper-fixture combinations in both argument orders.

The suite's upper fixture is not the actual XML Schema upper boundary, so a
focused control separately accepts `18446744073709551615` and rejects both
`-1` and `18446744073709551616`. This establishes the fixed unsigned 64-bit
value boundary for this private constructor path, but does not establish
lexical whitespace normalization, cross-type promotion, or general sequence
equality.

## QT3 non-positive-integer extension

The adjacent complete five-case `fn-deep-equalnpi2args` group also passes.
FastXSLT parses each `xs:nonPositiveInteger` constructor through checked `i128`
and enforces the derived type's less-than-or-equal-to-zero value space. The
group covers an equal negative pair and unequal negative lower-, middle-, and
zero-boundary combinations in both argument orders.

Focused controls accept both `-1` and `0` while rejecting `1`, distinguishing
this inclusive boundary from both `xs:negativeInteger` and unrestricted
`xs:integer`. This is still a bounded subset of the arbitrary-precision type;
values outside `i128`, lexical whitespace normalization, cross-type promotion,
and general sequence equality remain unclaimed.

## QT3 non-negative-integer extension

The adjacent complete five-case `fn-deep-equalnni2args` group also passes.
FastXSLT parses each `xs:nonNegativeInteger` constructor through checked `i128`
and enforces the derived type's greater-than-or-equal-to-zero value space. The
group covers equal zero values and unequal zero, middle, and upper-fixture
combinations in both argument orders.

Focused controls accept both `0` and `1` while rejecting `-1`, distinguishing
this inclusive boundary from both `xs:positiveInteger` and unrestricted
`xs:integer`. This is still a bounded subset of the arbitrary-precision type;
values outside `i128`, lexical whitespace normalization, cross-type promotion,
and general sequence equality remain unclaimed.

## QT3 bounded-short extension

The adjacent complete five-case `fn-deep-equalsht2args` group also passes.
Each admitted `xs:short` constructor is range-checked through Rust's signed
16-bit integer representation before comparison. The upstream group exercises
equal lower-bound values and unequal lower-, middle-, and upper-bound
combinations in both argument orders.

Focused controls accept both `-32768` and `32767` while rejecting `-32769` and
`32768`, making both fixed boundaries executable. This evidence does not add
numeric promotion, lexical whitespace normalization, general constructor
semantics, or general sequence equality.

## QT3 float and double extensions

The complete five-case `fn-deep-equalflt2args` and five-case
`fn-deep-equaldbl2args` groups now pass through the private atomic owner. Each
group exercises an equal finite lower-bound pair plus unequal zero, lower, and
upper fixture combinations in both argument orders. The harness asserts all
five upstream cases and all five first-party overlay records for each group, so
neither denominator can shrink silently.

The constructors parse the suite's scientific-notation lexicals into Rust
`f32` and `f64` values and retain their IEEE bit patterns for typed comparison.
Each singleton comparison charges one XPath operation for sequence length and
one for the reached item, with no XDM node visits. These groups establish the
selected finite fixtures only; they do not claim the complete XML Schema
float/double lexical spaces, whitespace normalization, every rounding edge,
cross-type promotion beyond the separately admitted mixed group, or general
floating-point arithmetic.

## QT3 static-arity error tranche

`K-SeqDeepEqualFunc-1` through `-3` now pass as an explicit three-case error
tranche. They exercise zero-, one-, and four-argument calls and require the
XPath static error `XPST0017`. The local deep-equal parser owns that standards
identity and retains the supplied expression resource and span; it does not
encode the standards code into display text.

At stylesheet compilation, the focused local error translates to the private
`FXXP0005 / invalid` identity while retaining the stylesheet location. A valid
three-argument call remains `FXXP1010 / unsupported` because FastXSLT has not
admitted collation semantics. This is evidence for one standards/private-code
mapping under AR-0004, not a public diagnostic catalog or a claim that all
function-signature errors are implemented.

## QT3 codepoint-collation and paired-NaN tranche

The explicit second K-family tranche selects `K-SeqDeepEqualFunc-6` and
`K-SeqDeepEqualFunc-8` through `-11`. Case 6 compares equal strings under the
standard Unicode codepoint collation. The other four cases verify paired NaN
for float/float, double/double, float/double, and double/float argument orders.
Every singleton comparison retains the existing two-operation charge and
performs no node visits.

Only the exact standard codepoint collation URI is admitted. Focused controls
keep an unknown collation URI and an empty collation argument unsupported,
despite the suite permitting an optimized `true` alternative for those equal
operands. FastXSLT therefore does not claim collation resolution, fallback, or
function-conversion semantics that it has not implemented. The following
checkpoint treats outer boolean composition separately from collation behavior.

## QT3 boolean-composition tranche

A private boolean-composition owner now wraps the existing deep-equal function
with only identity, `not(...)`, and `eq true()/false()` projections. It delegates
function parsing, standards/private diagnostics, node and atomic comparison,
and all work charging to the existing owners. The stylesheet compiler/runtime
and direct QT3 harness use the same composed representation; the harness does
not strip operators or manufacture expected booleans.

This admits `K-SeqDeepEqualFunc-7`, `-12` through `-16`, and `-18` through
`-20`. The nine cases cover empty-sequence equality, negated float/double NaN
versus zero in both orders, decimal-versus-URI type inequality, and first-,
second-, or third-position mismatches in three-item sequences. Exact operation
counts prove that the outer boolean projection adds no invented function work
and preserves inner early-exit behavior.

The upstream cases exposed and now control two atomic parser gaps: quoted
`xs:decimal`/`xs:integer` constructor arguments normalize through the same
lexical path as direct constructor arguments, and parenthesized sequences parse
arbitrary top-level item tails recursively rather than stopping at two items.
Comma-separated items outside sequence parentheses remain rejected. QName,
binary, `index-of`, and broader boolean-expression composition remain outside
this tranche.

## QT3 ordered and empty-sequence tranche

The next explicit K-family tranche selects `K-SeqDeepEqualFunc-25` through
`-31` and `K-SeqDeepEqualFunc-36` through `-46`. These 18 cases reuse the same
boolean-composition and atomic-sequence owners to cover equal ordered
three-item sequences, mismatches reached at the first, second, or third item,
strings in each sequence position, and empty items flattened out of
parenthesized sequence construction.

Each case asserts its exact local work count. Equal three-item sequences charge
one length decision and all three item comparisons. Mismatches stop after the
first unequal item. Unequal lengths after empty-item flattening charge only the
length decision and do not compare unreachable items. No XDM node visits occur.

This tranche adds no new public behavior and does not widen the expression
grammar beyond forms already admitted by the preceding checkpoint. QName cases
17 and 21, binary cases 22 through 24, and `index-of` cases 32 through 35 remain
unselected pending their own semantic work.

## QT3 mixed atomic-sequence tranche

FastXSLT now executes the complete 31-case
`fn-deep-equal-mix-args-*` group, 001 through 031, without denominator loss.
The admitted expressions cover:

- ordered one- and two-item integer sequences;
- string constructors, string literals, and parenthesized singleton strings;
- case-sensitive string value comparison;
- empty strings as one-item sequences; and
- empty sequences under ordinary, nested, and whitespace-bearing parentheses;
- `xs:anyURI` comparison against equal string literals and `xs:string`; and
- exact `xs:integer`/`xs:decimal` equality without binary floating point;
- integer and decimal promotion to float or double;
- float-to-double promotion that preserves the float's rounded value; and
- positive infinity, negative infinity, and the `fn:deep-equal` paired-NaN
  rule; and
- boolean constructors using `1`, `0`, `true`, or `false`, plus `true()` and
  `false()` function values; and
- distinct typed `xs:date`, `xs:dateTime`, and `xs:time` values compared
  against equal lexical strings.

The parser finds the function's argument separator at parenthesis depth zero,
so a comma inside an operand sequence cannot be mistaken for the separator.
The private representation flattens the admitted parenthesized sequences and
compares equal-length items in order. Evaluation charges one XPath operation
for the length decision and one for every item comparison reached; it performs
no node visits.

The atomic representation retains URI, string, integer, and decimal type
identity. The admitted comparison step applies string-like equality between
URI and string values and exact integer/decimal equality only when the
normalized decimal has no fractional component.

Float and double values retain their IEEE bit patterns in the private atomic
representation. Comparison reconstructs the typed value, promotes float to
double without reparsing its original lexical form, and treats two NaN values
as deep-equal while leaving ordinary numeric equality rules unchanged.

The boolean parser retains a typed boolean value, normalizes only the four XML
Schema boolean lexical forms, and rejects other strings rather than applying
host-language truthiness.

The admitted calendar parser validates four-digit positive years, real month
and day combinations including leap years, and whole-second clock fields from
00:00:00 through 23:59:59. It retains date, date-time, and time type identity,
so lexical equality with a string does not become deep equality.

Completing this group does not claim general XPath sequence parsing, escaped
string literals, collations, broader typed-value promotion, every
floating-point lexical form, timezone-bearing or fractional calendar values,
24:00:00, negative/expanded years, or general `fn:deep-equal` semantics across
the rest of the 263-case QT3 test set.
