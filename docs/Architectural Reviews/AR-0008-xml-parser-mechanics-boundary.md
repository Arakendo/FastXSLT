# AR-0008: XML Parser Mechanics Boundary

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-25 |
| Last reviewed | 2026-09-27 |
| Scope | XML byte decoding, tokenization, namespaces, provenance, limits, and XDM handoff |
| Trigger | M1's private transform slice needs to turn admitted bytes into FastXSLT-owned document semantics |
| Related ADRs | ADR-0001, ADR-0002, ADR-0003 |
| Related evidence | `docs/Evidence/rust-xml-parser-candidate-review-2026-08-25.md`, `docs/Evidence/owned-xdm-tree-experiment-2026-08-25.md`, `docs/Evidence/w3c-xml-conformance-suite-candidate-review-2026-08-25.md`, `docs/Evidence/oasis-xslt10-declared-legacy-input-encoding-2026-09-26.md`, and `docs/Evidence/oasis-xslt10-bom-utf16-input-2026-09-27.md` |

## Architectural question

Which replaceable parser should supply XML mechanics for the first private
slice, and what validation and ownership must remain inside FastXSLT so the
dependency does not define XDM semantics, diagnostics, authority, or a public
contract?

## Trigger and evidence

The bounded resource experiment can seal caller-supplied bytes without retaining
file handles. M1 now needs an XML event-to-XDM seam. The parser must operate on
those bytes without opening paths or resolving external identifiers, preserve
provenance, support namespaces, report malformed input, accept explicit limits,
and fit Rust 1.85 and the MIT distribution posture.

The candidate review compared `quick-xml` 0.40.1, `xmlparser` 0.13.6, and
`roxmltree` 0.21.1. A private `quick-xml` adapter passed focused namespace,
malformed-input, authority, provenance, and limit tests. No representative
parser benchmark, XML conformance subset, non-UTF-8 policy, fuzz result, or
production dependency audit exists yet.

## Ownership and constraints

- The host and resource layer own which bytes and logical identity are admitted.
  The parser receives bytes; it receives no path, network client, or ambient
  resolver.
- The XML boundary owns supported XML editions, decoding, well-formedness,
  namespace constraints, source spans, parser limits, and translation of
  dependency failures into structured engine diagnostics.
- XDM owns nodes, expanded names, identity, order, typed and string values, and
  navigation. Parser event or node types may not escape into XDM or public APIs.
- XPath and XSLT own language semantics. Parser convenience behavior cannot
  decide whitespace stripping, QName interpretation, base URI, or stylesheet
  rules.
- DTDs and external entities are denied by the current experiment. Any later
  support requires explicit host authority and security-limit review; the parser
  must never resolve them implicitly.
- ADR-0002 forbids retained source handles, reopening paths, temporary spill,
  and hidden disk caches.
- AR-0007 permits a concrete first tree but forbids spreading a dependency's
  random-access tree assumption through semantic layers.
- ADR-0003's first-party `unsafe` prohibition remains. Transitive unsafe and
  feature changes require dependency evidence rather than being hidden by the
  safe adapter.

## Alternatives

### A. `quick-xml` pull events behind a FastXSLT adapter

This supplies slice-based parsing, namespaces, byte positions, common
well-formedness checks, and a low-allocation path. FastXSLT must add document,
namespace, entity, limit, and standards validation where the crate does not.
It keeps XDM construction and ownership explicit.

### B. `xmlparser` tokens plus FastXSLT structural validation

This minimizes dependency and allocation pressure and exposes spans. Its stated
omission of nesting and duplicate-attribute validation increases correctness
work before the first transform and risks inventing commodity XML mechanics.

### C. Parse with `roxmltree` and copy into owned XDM

This gives a convenient validated source tree and positions. It adds a second
tree, source-lifetime coupling, peak-memory pressure, and temptation to let
dependency nodes become semantic identity. Copying could still make it a useful
temporary oracle.

### D. Adopt a parser tree as the XDM representation

This is fast to start but delegates FastXSLT's defining data-model ownership,
leaks dependency lifetimes, and closes representation options prematurely. It
conflicts with the SDD and AR-0007.

### E. Write or bind a complete XML parser now

This offers maximum control but creates substantial correctness, security,
portability, FFI, or maintenance burden before engine semantics exist. Current
evidence does not justify it.

## Findings and uncertainties

- Alternative A is the best private experiment direction.
- Version 0.40.1 is MIT, fits the MSRV, and can remain isolated as a dev-only
  exact pin while its behavior is evaluated.
- Parser byte offsets can be wrapped with snapshot identity without exposing
  filenames or dependency errors.
- DTD denial at the event boundary preserves the current no-ambient-authority
  rule.
- FastXSLT must validate duplicate expanded attribute names; raw lexical
  duplicate checking is insufficient when two prefixes bind the same namespace.
- Comments and processing instructions cannot be discarded merely because the
  first golden output does not use them; they exert XDM construction pressure.
- Complete XML name and namespace validation, declaration rules, character and
  encoding support, source line mapping, entity policy, performance, and
  adversarial robustness remain uncertain.
- A private `quick-xml` encoding-feature experiment decodes declared
  ASCII-compatible single-byte input while retaining original-byte source
  offsets. A private BOM-selected UTF-16LE/UTF-16BE adapter also preserves
  original-byte offsets across strict transcoding. These experiments do not
  settle the supported-encoding profile, transitive unsafe review,
  vulnerability review, or production dependency admission.
- The official W3C XML 20130923 archive supplies 2,586 catalog cases across XML
  editions, namespaces, validity, well-formedness, entity modes, and canonical
  output. It is suitable pressure but requires dependency-aware selection.
- The archive's root catalog composes 21 fragments through DTD entities. A
  harness must inventory those local fragments explicitly rather than granting
  ambient entity resolution.
- Older mixed contributor notices remain in the archive and the current W3C
  dual-license policy is not retroactive by default. The archive is therefore a
  locally inspected candidate, not an admitted redistributable corpus.

## Disposition

**Under Review. Continue Alternative A only as a private experiment.** Do not
stabilize `quick-xml` types, promote the crate to a production dependency, or
describe the experiment as XML conformance. The adapter is the replaceable
boundary and FastXSLT remains responsible for accepted XML behavior.

## Required follow-up

- [x] Pin `quick-xml` 0.40.1 as a dev-only dependency compatible with Rust 1.85.
- [x] Demonstrate in-memory parsing, namespace expansion, logical provenance,
  DTD denial, unknown-entity denial, malformed structure, and explicit limits.
- [x] Select XML 1.0 Fifth Edition and Namespaces in XML 1.0 Third Edition as
  the reference semantics through ADR-0007; parser and encoding admission remain
  unresolved here.
- [ ] Run a standards-derived XML well-formedness and namespace subset,
  especially names, namespace declarations, declarations, characters, and
  document structure.
- [x] Inventory the W3C XML 20130923 candidate archive, case classes, metadata,
  entity/catalog mechanics, SHA-256, and redistribution uncertainty.
- [ ] Decide local-only acquisition versus repository/CI admission after a
  focused rights review, preserving the exact archive digest.
- [ ] Decide UTF-8-only versus explicit supported encodings and test declaration
  mismatches and byte-order marks. A focused ASCII-compatible single-byte
  experiment and BOM-selected UTF-16LE/UTF-16BE experiment are complete; the
  supported public profile remains open.
- [ ] Define offset-to-line/column indexing without copying parser types or
  repeatedly scanning large resources.
- [x] Build the first owned XDM document without retaining parser events or
  dependency node handles.
- [ ] Fuzz malformed input and measure allocation, latency, and peak memory on
  representative source and stylesheet sizes.
- [ ] Complete license, feature, vulnerability, and transitive unsafe review
  before production admission.
- [ ] Propose an ADR only if production admission or a stable parser boundary is
  justified by that evidence.

## Reopening triggers

Reassess the leading candidate if XML cases expose material correctness gaps,
required encodings are impractical, source positions cannot support diagnostics,
parser allocations dominate consumer-visible work, dependency policy changes,
or another physical input strategy requires a different event seam.

## Review history

- 2026-09-27 -- Threaded the host-supplied XML event and depth envelope through
  principal and dependency stylesheet parsing as well as source preparation.
  Exhaustion is a structured limit outcome, and two unchanged large OASIS
  stylesheets become exact under the measurement host's larger bounded policy.
  Production defaults and parser admission remain open.
  [Evidence](../Evidence/oasis-xslt10-host-owned-stylesheet-xml-limits-2026-09-27.md)
- 2026-09-27 -- Added strict BOM-selected UTF-16LE and UTF-16BE decoding with
  parser offsets mapped back to the immutable admitted bytes. Twenty-two OASIS
  standard-operation cases leave the generic stylesheet-XML frontier, six
  become exact, and the remainder expose named numbering/runtime/mismatch
  frontiers. The public encoding profile and production parser admission remain
  open.
  [Evidence](../Evidence/oasis-xslt10-bom-utf16-input-2026-09-27.md)
- 2026-08-25 -- Opened Under Review with a dev-only `quick-xml` experiment.
- 2026-08-25 -- Confirmed the adapter can feed an owned private XDM tree after
  source bytes are released; production admission gates remain open.
- 2026-08-25 -- Inspected the official W3C XML 20130923 archive and retained it
  as a non-admitted candidate. Edition-aware selection, secure acquisition, and
  rights disposition remain before subset execution.
- 2026-08-26 -- ADR-0007 selected the XML and Namespaces reference editions.
  Encoding coverage, parser production admission, corpus rights, and executable
  subset selection remain open.
- 2026-08-26 -- XSLT30 `template-002` exposed and verified adapter-owned PI
  target/data separator normalization. The parser remains private and
  production admission gates remain open.
- 2026-09-26 -- Enabled the private parser candidate's encoding decoder for
  declared ASCII-compatible single-byte XML while preserving raw-byte source
  offsets. Seven OASIS cases initialize, three become exact, and two lossy
  archival include references receive explicit exclusions. UTF-16 provenance,
  the supported encoding profile, dependency audit, and production admission
  remain open.
  [Evidence](../Evidence/oasis-xslt10-declared-legacy-input-encoding-2026-09-26.md)
