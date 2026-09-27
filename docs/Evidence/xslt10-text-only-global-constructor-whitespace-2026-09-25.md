# XSLT 1.0 text-only global-constructor whitespace

Date: 2026-09-25

## Result

Text-only global variable and parameter constructors now apply the same
stylesheet whitespace rule as other sequence constructors. Whitespace-only
stylesheet text nodes are omitted under the default `xml:space` policy and are
retained when `xml:space="preserve"` is effective.

The compiler derives the retained string from the constructor's direct text
children. It does not trim text nodes that contain non-whitespace characters,
and it does not change source-document whitespace.

## Evidence

The focused runtime case splits text nodes with stylesheet comments so it can
prove both branches independently:

- defaulted whitespace-only nodes are absent from the temporary text value;
- preserved whitespace-only nodes remain byte-for-byte visible; and
- the result travels through the normal global-binding and transform-set path.

The complete archival OASIS XSLT 1.0 measurement remains:

- 3,173 catalog cases;
- 2,328 initialized;
- 2,281 executed successfully;
- 2,111 exact XML-semantic matches (66.53%);
- 47 execution failures;
- 85 visible XML mismatches; and
- 77 comparator-unsupported outcomes.

No archival pass is claimed for this correction. The inspected nearby mismatch
also includes whitespace-only nodes from the source document, which this
stylesheet-constructor rule must not erase.

## Validation

- `cargo test -p fastxslt xslt10_text_only_global_constructor_strips_only_defaulted_whitespace_nodes --all-features`
- `scripts/measure-oasis-xslt10.ps1`

