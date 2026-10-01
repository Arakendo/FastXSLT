# AR-0025 Redundant Namespace Default Experiment

## Question

Can the bounded internal-subset path admit the remaining DTD-derived XSLT
namespace/version defaults without rewriting the admitted XML byte stream or
pretending that a post-tokenization namespace binding affected QName parsing?

## Inventory result

The eight internal-only Microsoft `22-8` stylesheet inputs behind the observed
namespace/default seam are narrower than the initial failure suggested:

- every stylesheet authors `xmlns:xsl` on its document element;
- DTD declarations repeat the same fixed XSLT namespace binding on selected
  descendant XSLT elements;
- `Attributes__81545` and `Attributes__81551` omit the document element's
  `version="1.0"`, which their internal subsets supply; and
- none needs a DTD declaration to establish a previously unbound QName prefix.

The shared principal source, `Plants.xml`, independently names the external
subset `plants.dtd`. The complete cases therefore still stop at explicit
external-identifier denial before stylesheet execution. This source boundary
must not be mistaken for a failure of the narrower stylesheet experiment.

## Implemented private rule

The reference parser now accepts a DTD-derived namespace declaration only when
the tokenizer has already resolved the same prefix to the identical normalized
namespace value. The declaration is retained as local XDM namespace metadata,
but it is not credited with establishing or changing the binding used for QName
resolution. Missing, unknown, or different bindings remain rejected.

Ordinary DTD-derived `version` attributes continue through the existing bounded
default-attribute path. A focused compile-and-transform test proves a stylesheet
whose root authors `xmlns:xsl`, whose DTD supplies `version="1.0"`, and whose
template receives the same redundant fixed namespace binding.

This rule avoids a pre-tokenization rewriter for the observed stylesheet shape.
It does not admit general DTD namespace defaulting.

## Conservation and verification

- Production parsing and compilation still deny every DTD.
- No resource acquisition or external-subset resolution was added.
- Default bytes remain charged once through the existing replacement-byte
  accounting.
- A focused parser test proves equivalent-binding admission and local namespace
  retention.
- The same test proves that a missing binding remains an unknown-prefix failure
  and that a different inherited binding remains explicitly unsupported.
- The complete hash-verified 3,173-case OASIS sweep remains unchanged at 2,361
  exact comparisons (74.41%), 2,552 initialized cases, and 2,506 successful
  executions. The direct DTD frontier remains 117 cases: 15 parsed and 102
  explicitly unsupported.

The unchanged numerator is expected: the eight motivating catalog cases share
an external-subset-bearing principal source, which remains outside the bounded
internal-only profile.

## Disposition

Retain the narrow equivalence rule in the private AR-0025 reference path. Do not
build a byte-rewriting pre-tokenization layer for these cases. Reopen that
larger option only when an admitted workload requires a DTD declaration to
establish or change a namespace binding before QName resolution and can justify
the provenance and diagnostic complexity.

