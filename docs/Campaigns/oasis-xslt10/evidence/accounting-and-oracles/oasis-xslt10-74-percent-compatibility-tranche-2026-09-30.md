# OASIS XSLT 1.0 74% Compatibility Tranche

## Purpose

Record the implementation and complete-catalog measurement that moves the
strict OASIS XSLT/XPath 1.0 Committee Draft 04 compatibility lower bound from
73.00% to 74.00%. The archive remains a local, hash-verified, non-redistributed
input and the result remains compatibility evidence rather than a conformance
claim.

## Implemented boundary

This tranche adds four bounded pieces of shared or XSLT-1.0-specific behavior:

- text sorting now retains explicit language and case-order plan facts, supports
  the measured English, Spanish, French, Swedish, and Turkish compatibility
  collations, accepts bounded dynamic language/case controls, and preserves the
  existing charged stable-sort path;
- a local XSLT 1.0 variable whose select is one literal `document()` call may
  retain an invocation-owned prepared document and supply it to `xsl:copy-of`;
  resolution remains relative to the binding stylesheet module and restricted
  to the sealed resource snapshot;
- empty `xsl:use-attribute-sets` lists are admitted only in the XSLT 1.0
  compatibility path, while modern stylesheets retain `XTSE0020`;
- the existing XSLT 1.0 recovery seam ignores a bounded set of known misplaced
  XSLT control attributes. Modern stylesheets still report `FXST1009`.

The shared XML-name validator now recognizes the XML 1.0 Fifth Edition NCName
character repertoire rather than ASCII alone. The Japanese/Chinese computed
element corpus case consequently executes, but its archived expected file
contains replacement question marks and remains an uncredited comparator gap.
Semantic execution is not counted as an exact result when the reference cannot
verify it.

## Ownership and safety

- Prepared external documents are stored only in the local runtime-variable
  frame for one invocation. They are not added to compiled state, a global
  cache, another request, or the sealed snapshot.
- Binding performs the existing controlled XML parse/XDM preparation and uses
  the existing dynamic-document cache and work accounting.
- Copying a prepared variable document reuses the ordinary source-document copy
  implementation, including namespace fixup, result budgets, cancellation, and
  unattached-attribute recovery.
- Sort collation behavior is a compiled activated path; unrelated transforms do
  not acquire runtime language-policy branches.
- No unsafe code, ambient I/O, live resolver, or new public API is introduced.

## Conserved measurement

The complete hash-verified denominator remains 3,173 catalog cases.

| Counter | Result |
| --- | ---: |
| Exact expected-result matches | 2,348 / 3,173 (74.00%) |
| Initialized | 2,537 |
| Successfully executed | 2,491 |
| Initialization failures | 633 |
| Execution failures | 46 |
| Visible comparison mismatches | 7 |
| Comparator gaps | 4 |
| Expected-error credit | 423 / 431 (98.14%) |

Relative to the preceding 73.00% snapshot, 32 additional catalog cases compare
exactly, 38 additional cases initialize, and 38 additional cases execute. Five
newly visible mismatches and one new comparator gap remain uncredited. This is
intentional: the tranche exposes later serialization/reference limitations
rather than hiding them behind an earlier unsupported result.

The exact lower bound is computed as `2348 / 3173 = 74.00%` rounded to two
decimal places. All 3,173 catalog identities retain one visible disposition.

## Verification

- Focused compiler tests cover static/dynamic sort controls, XSLT 1.0 empty
  attribute-set lists, compatibility-only misplaced-control recovery, and
  modern rejection of the same constructs.
- Focused runtime tests cover language/case collation behavior and a local
  literal-document variable copied from a sealed snapshot.
- XML-name unit tests cover non-ASCII XML 1.0 names and invalid NCName syntax.
- The complete local OASIS sweep produced the conserved counters above without
  editing the upstream archive or expected results.

## Non-claims

- This does not establish XSLT 1.0 conformance or complete locale collation.
- This does not admit global `document()` variables, a public prepared-document
  value, live resource acquisition, or cross-invocation prepared state.
- This does not treat archival replacement characters or whitespace/layout
  differences as exact matches.
- This does not broaden XSLT 1.0 recovery behavior beyond the tested private
  compatibility seam.
