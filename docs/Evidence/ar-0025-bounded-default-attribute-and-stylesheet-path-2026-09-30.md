# AR-0025 Bounded Default-Attribute and Stylesheet Path

## Purpose

Record the next bounded AR-0025 tranche: literal/fixed declaration-derived
attributes, plus the same explicitly selected internal-subset parser on
stylesheet modules. Production source parsing and stylesheet compilation still
deny every DTD by default.

## Implemented reference slice

The private internal-subset parser now retains bounded literal and `#FIXED`
defaults for the already admitted `CDATA` and tokenized attribute types. On an
element start, the XML adapter:

- preserves an authored attribute instead of replacing it with its default;
- injects a missing declaration-derived attribute before XDM construction;
- applies XML whitespace normalization according to the declared type;
- charges repeated injected bytes against the existing cumulative DTD
  replacement-byte limit; and
- rejects a missing declaration-derived namespace binding at the current
  post-tokenization seam instead of constructing a wrongly namespaced tree.

The measurement-only workbench can select the bounded parser independently for
stylesheet compilation as well as source preparation. Ordinary constructors
continue to select complete DTD denial. Stylesheet dependencies still come
only from the sealed resource snapshot; no resolver or ambient acquisition was
added.

## Corpus result

The complete hash-verified OASIS XSLT/XPath 1.0 CD04 denominator remains 3,173
catalog cases. Three additional unchanged cases initialize, execute, and
compare exactly:

- `Microsoft/Keys__91858`, whose source combines internal entity replacement
  with an explicit attribute covered by a default declaration;
- `Microsoft/Keys__91860`, whose missing source attribute is injected from the
  internal subset and retains the owning element as its XDM parent; and
- `Microsoft/BVTs_bvt004`, whose simplified stylesheet authors the namespace
  and version attributes named by its fixed declarations while using an
  internal entity in an ordinary result attribute.

The strict lower bound moves from 2,358 to **2,361 exact comparisons out of
3,173 (74.41%)**. Initialization reaches 2,552, successful execution reaches
2,506, and initialization failures fall to 618. The seven visible XML
mismatches, four comparator gaps, and 423 / 431 expected-error credit remain
unchanged. Reaching 75.00% now requires 19 additional exact cases.

The original direct DTD frontier remains conserved at 117 cases with its
87-source/30-stylesheet role split. The private reference parser now reports 15
parsed resources and 102 explicit unsupported declaration/external outcomes.
Reference-parser admission is not itself pass credit; the measurement runner
continues to require complete engine execution and expected-result comparison.

## Focused verification

Focused tests prove authored-over-default precedence, missing default/fixed
injection, cumulative replacement-byte pressure, unchanged production denial,
and a simplified stylesheet whose internal entity expands through the private
bounded compilation route. The complete OASIS sweep proves the declaration-
derived attribute participates as an ordinary child of its source element.

## Remaining seam

The remaining external-identifier-free stylesheet cases depend on namespace or
version attributes that exist only in the DTD. Namespace declarations must be
available before the XML tokenizer resolves element and attribute names. The
current adapter deliberately rejects that situation; post-tokenization
injection would be semantically false. That seam requires a separate parser-
boundary experiment or a decision to keep those cases unsupported.

## Non-claims

- Production DTD support is not enabled.
- The reference parser is non-validating and does not claim the complete XML
  attribute-declaration grammar.
- DTD-derived namespace declarations are not supported.
- External subsets/entities and parameter entities remain denied.
- This compatibility evidence does not establish XSLT 1.0 conformance.
