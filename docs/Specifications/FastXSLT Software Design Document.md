# FastXSLT Software Design Document

| Field | Value |
| --- | --- |
| Status | Draft, pre-stability |
| Last updated | 2026-08-26 |
| Applies to | FastXSLT workspace |

## 1. Product intent

FastXSLT is a Rust-native engine for compiling and executing XSLT transforms
inside other applications. A motivating consumer is a performance-sensitive
ASP.NET application that needs to compile stylesheets once and execute them many
times. The product should be embeddable, observable, secure by explicit policy,
and capable of measuring standards conformance without substituting host-library
behavior for FastXSLT semantics.

The engine is distributed under the MIT License so consuming applications can
embed and redistribute it with minimal licensing friction. Dependency and corpus
admission must preserve that distribution model and retain all required notices.

ADR-0007 selects XSLT 3.0, XPath/XDM 3.1, Serialization 3.1, XML 1.0 Fifth
Edition, and Namespaces 1.0 Third Edition as reference semantics for a staged
modern profile. Support remains feature-enumerated and incomplete; the decision
does not imply general XSLT 1.0, 2.0, or 3.0 conformance.

### Goals

- Correct, testable XSLT and XPath semantics for an explicitly selected profile.
- Reusable compiled stylesheets separated from per-transform dynamic context.
- Structured, source-located diagnostics through all engine phases.
- Explicit host resource and security policy suitable for library embedding.
- A layered verification model spanning unit, golden, conformance, differential,
  integration, and benchmark evidence.
- Performance work driven by profiles and representative transforms.
- Low-overhead host integration whose end-to-end latency and throughput can be
  measured from real consumers, including ASP.NET.

### Non-goals for the scaffold

- Selecting an XML parser or tree representation.
- Claiming a broad standards version or conformance level beyond implemented,
  ledger-accounted features.
- Streaming, schema-aware processing, packages, or extension functions.
- A command-line interface, service, browser binding, or alternate backend.
- Stable public APIs before an end-to-end transform exercises them.

A CLI is not the primary product boundary. Host adapters are expected later,
but their concrete mechanisms are not selected by the scaffold.

## 2. Initial logical layers

FastXSLT begins as one library crate with private logical layers:

```text
Host application
      |
      v
Public facade and explicit host policies
      |
      +--------------------------+
      v                          v
Stylesheet compilation       Transformation runtime
      |                          |
      +------------+-------------+
                   v
              XSLT semantics
                   v
              XPath semantics
                   v
               XDM meaning
                   v
        Replaceable XML boundary
```

The diagram describes ownership, not one-way runtime calls in every case. XML
mechanics may be supplied by a dependency, but engine-visible node identity,
sequences, XPath, template selection, and transformation meaning belong to
FastXSLT.

Host-specific adapters, including a future ASP.NET/.NET adapter, sit above the
public facade. They translate host values, lifetime, cancellation, diagnostics,
and output without reimplementing engine semantics.

### Architectural invariants

These constraints apply even while their concrete Rust representations remain
open:

- XML dependencies provide parsing or serialization mechanics, never
  FastXSLT-visible XPath or XSLT semantics.
- A compiled stylesheet contains stylesheet-derived static state only. Source
  documents, invocation parameters, messages, clocks, resolver state, budgets,
  and other per-transform mutable state belong to a runtime invocation.
- A compiled global variable or parameter declaration may retain its
  stylesheet-defined default expression or value, but each invocation owns the
  resulting binding value and any host-supplied parameter override. Compilation
  must not turn an invocation parameter value into shared mutable state.
- Source-derived global values retain identities only within the prepared input
  and invocation that produced them. A compiled stylesheet must not retain node
  identities from one principal source for reuse by another invocation.
- Runtime atomic-variable frames may share immutable bindings only within one
  invocation. Frame mutation uses safe copy-on-write isolation, and no runtime
  binding map crosses an invocation, prepared-input, worker, snapshot, or
  generation boundary. The complete-clone frame remains a private test oracle
  under ADR-0014.
- For the exact admitted `xsl:strip-space elements="*"` policy, runtime composes
  compiled stylesheet policy with immutable prepared XDM through a private
  invocation-owned visibility view. Every source-semantic consumer observes
  the effective relationships, visible nodes retain prepared identity and
  provenance, and the view is never retained across invocations or
  generations. ADR-0012 does not admit broader whitespace declarations or a
  public source-view abstraction.
- The invocation explicitly selects its standards-defined entry, such as a
  principal source in the default or a named initial mode, or a named initial
  template. A source-free initial-template
  invocation must not acquire a fabricated source document or context item;
  the selected entry remains invocation state while template definitions remain
  immutable compiled state.
- The private initial-mode evidence supports both admitted-document focus and
  one explicitly supplied expanded-name document-element focus. Element
  resolution occurs after bounded source preparation and remains invocation
  state. This exact corpus-driven seam does not select a public context-node,
  selector-expression, borrowing, or prepared-input contract.
- Engine-visible nodes provide stable identity and document order for the
  lifetime required by the accepted standards profile. Neither property may be
  inferred accidentally from Rust object identity or allocation order.
- Engine layers depend on the semantic navigation and retention capabilities
  they require, not on a permanent assumption that every source is a fully
  materialized random-access tree. A concrete tree implementation remains valid
  for the first evaluator; this rule does not require speculative provider
  traits or a streaming backend.
- Resource access occurs only through explicitly supplied host capabilities.
- A sealed resource snapshot has stable identity and content for the lifetime
  of compilation and transformations that use it; host files changing beneath
  the snapshot do not mutate engine-visible inputs.
- After a host adapter admits bytes and closes its source handle, core
  compilation and execution are memory-resident by default and never reopen a
  provenance path or create implicit disk artifacts.
- Resource authority and execution budgets remain explicit even when a host
  chooses permissive policies.
- Structured diagnostics and sufficient source provenance survive parsing,
  semantic normalization, lowering, and optimization until a presentation
  boundary deliberately formats them.
- A transformation result model remains conceptually distinct from
  serialization into text, bytes, or an output sink.
- Optimization preserves every observable behavior required by the accepted
  standards profile and reference semantic tests.
- Instrumentation does not change transformation semantics or introduce ambient
  global state.

### Conceptual dependency rules

The starting dependency direction is:

```text
facade
  |---> compilation ----+
  `---> runtime --------+---> XSLT ---> XPath ---> XDM
                                                ^
XML parsing mechanics --------------------------+

diagnostics <--- emitted by every phase
host policy ---> consumed only where authority or budgets are required
```

This is an ownership rule rather than a commitment to exact Rust modules or
traits. Lower semantic layers do not invoke higher ones: XDM does not select
templates, XPath does not own stylesheet compilation, and XML parser convenience
APIs do not become query semantics. XSLT may use XPath semantics; shared helpers
must not become an unnamed alternate owner of either language. Compilation and
runtime may coordinate the layers through private representations until an
accepted ADR establishes a stable boundary.

### XML boundary

The XML layer adapts parser events or nodes and serialization mechanics. Parser
selection, DOM wrapping versus an owned arena, DTD support, and entity behavior
remain open. External entities and ambient network/filesystem access default to
unavailable until explicit policy exists.

### XDM

The XDM layer will own the engine-visible data model: node identity and order,
atomic values, sequences, names, namespaces, and conversions required by the
selected standards profile. Without knowing the physical representation, XPath
and XSLT may rely only on the semantic operations admitted by that profile,
expected to include node kind, identity, expanded name, string value,
relationships, root/document membership, and document order. Attributes,
namespaces, and typed values depend on the selected profile.

Expanded-name identity remains independent of lexical prefix spelling. The
owned XML/XDM boundary nevertheless retains an element's authored prefix when
result namespace fixup or serialization must preserve a selected qualified
name; two prefixes bound to one namespace therefore compare as one expanded
name without becoming indistinguishable serialization choices.

The physical representation is not yet decided. In particular:

```text
node identity  != Rust object identity
node equality  != value equality
document order != allocation order
```

### XPath

The XPath layer will own lexing, parsing, static context, dynamic context,
evaluation, type behavior, and the supported function library. It must not
delegate semantics to a host query engine whose behavior cannot be inspected or
tested against the chosen profile.

### XSLT and compilation

Compilation is conceptually a pipeline:

```text
XML stylesheet
    -> stylesheet syntax and structure
    -> static resolution and validation
    -> semantic normalization
    -> executable and optionally optimized representation
```

These stages need not become separate public types or multiple IRs. The
distinction prevents parsed syntax, semantic meaning, and a current execution
strategy from becoming architecturally identical. The reusable compiled result
must retain sufficient source provenance for diagnostics and must not capture
per-transform source documents, parameters, messages, clocks, resolver state,
budgets, or host resources as hidden global state.

Compiled template rules retain stylesheet-module import precedence separately
from template priority and declaration order. Ordinary dispatch ranks all three;
`xsl:next-match` continues to a lower-ranked applicable rule, while
`xsl:apply-imports` considers only applicable rules at lower import precedence
before using the built-in rule. Equal-ranked ordinary dispatch uses the later
declaration in the admitted recovery path. The private transform-set path may
instead request error-on-multiple-match; it reports concrete dynamic error
`XTDE0540` when ordinary or `xsl:next-match` selection encounters more than one
applicable rule at the highest eligible import precedence and priority. A tie
below a unique higher-ranked rule is not an error unless continuation reaches
that tied rank. Corpus harnesses verify each upstream recover/error dependency
before selecting the path, but this does not select a general legacy
compatibility profile or expose a public or host-configurable policy. The
current executable slice proves only the bounded module topologies below; it
does not define a public module graph representation.

The same policy seam has native mode-suite evidence for document-rooted match
paths. An admitted absolute path is compiled through the existing typed
location-path representation and evaluated from the source document node;
relative patterns retain their candidate-relative matching behavior. This does
not admit leading descendant patterns such as `//name`, warning delivery, or a
general pattern grammar.

The private compiler admits named `xsl:mode` declarations only when
`warning-on-multiple-match` is absent or uses a warning-disabled lexical. The
declaration is validated but adds no runtime state because the property is
semantically inert in that form. Invalid boolean lexicals report `XTSE0020`;
warning-enabled values remain explicitly unsupported until warnings have an
owned, bounded delivery channel. This does not admit other mode properties or
define a public diagnostic event representation.

Each source-node template invocation retains its position and size within the
sequence selected by the applying instruction, including non-element child
nodes. The bounded positional-pattern slice evaluates `position()` against the
matching named-element sibling sequence, while the exact `{position()}` and
`{last()}` literal-result AVTs observe the invocation focus. This does not admit
general focus functions, positional expressions, or arbitrary AVTs.

Compiled mode names use expanded QName identity. An unprefixed lexical mode is
in no namespace; a prefixed mode is resolved against the namespace context of
the containing stylesheet instruction and retained in canonical expanded form.
Template declarations and apply-templates instructions therefore do not equate
names merely because their prefixes or local parts match. `#default`, `#all`,
and `#current` retain their separate control meanings. This private identity
model does not yet define a public initial-mode QName type.

The private temporary-tree path distinguishes its document focus from the
principal source document. A bare variable selected by `xsl:apply-templates`
may resolve an invocation-local or global temporary tree, with local lexical
state taking precedence. Nested apply-templates with no explicit selection
continues from that temporary document or node focus and does not silently fall
back to the principal source. Each materialized temporary tree also receives a
first-class identity from its invocation control. Copies of the private tree
handle preserve that identity, independently materialized trees within the
same invocation receive distinct identities, and the identity domain is not
shared across invocations, workers, snapshots, or generations. The identifier
spelling remains private and opaque; allocation addresses and variable names
are not semantic identity. The current representation admits attribute-free
literal element trees with non-whitespace text children, preserving mixed child
order and accounting separately for retained XDM nodes and result text bytes.
Top-level text, attributes, comments, processing instructions, and general
temporary-tree paths require separate evidence.

The private temporary selection path also admits an exact child-element path
whose origin is one temporary-tree variable and whose steps are lexical QNames
resolved to expanded names during stylesheet compilation. Execution starts at
the temporary document roots, preserves stored child order, and charges each
inspected node as XPath work. Wildcards, predicates, descendant axes, mixed
node tests, and arbitrary expressions are not implied by this bounded path.

The private pattern slice admits a pipe-separated union of exact
multi-step element paths. Each branch is compiled to expanded-name steps and
matched leaf-to-ancestor using representation-owned parent identity. Source and
temporary trees share the same rule-ranking contract: import precedence and
compiled priority determine the winning semantic rank, with later declaration
order used only for tied recovery where policy permits it. Wildcards,
predicates, axes, and general union operands remain unsupported.

`xsl:next-match` preserves temporary focus, current mode, and matched-template
identity. It selects the highest eligible lower-ranked applicable rule using
the same ranking and ambiguity policy as source-tree continuation; exhaustion
uses the temporary-tree built-in rule and never silently changes focus to the
principal source.

The same private path admits `xsl:for-each` only when its selection is one bare
temporary-tree variable. That form establishes the selected temporary document
as focus for the instruction body without changing the surrounding current
template rule or current mode. This is not a general `xsl:for-each` or sequence
iteration contract.

The bounded include slice also admits one three-module include chain in which a
simple fragment selects exactly one embedded stylesheet by `xml:id`. Resource
bytes are acquired under the fragmentless identity before fragment semantics
are applied, and inherited `xml:base` determines the embedded module's nested
reference base. General XPointer, DTD-typed IDs, and arbitrary nested or mixed
module graphs remain outside the slice.

One additional bounded topology admits a principal module with exactly one
leading import followed by one include. Included declarations share principal
import precedence; imported declarations remain lower. This permits
`xsl:apply-imports` from an included rule to select its imported counterpart
without treating inclusion as a separate precedence level. Other mixed,
repeated, or nested import/include graphs remain outside the executable slice.

The largest admitted private topology contains two principal includes, each
with one leaf import. Included rules share principal precedence and declaration
order provides use-last recovery for the selected same-precedence conflict;
each imported leaf remains lower for `xsl:apply-imports`. This is corpus-bound
evidence, not a general precedence-graph or configurable conflict-policy model.
The private path admits both the suite's explicit recover request for its
XSLT 1.0/2.0 variant and its XSLT 3.0+ positive variant. The same private
policy seam now has corpus evidence for error-on-multiple-match in both the
apply-templates set and the exact five-module `include-0702b` graph, but it is
not exposed as a host-selectable contract.

A second five-module topology admits two leading imports from the principal,
each with one leaf import. The first leaf, first branch, second leaf, second
branch, and principal occupy distinct increasing import-precedence strata.
This preserves declaration-order precedence between sibling import subtrees
and lets `xsl:apply-imports` select the highest applicable lower stratum. The
topology is bounded evidence, not a general recursive precedence-graph model.

Module assembly resolves statically known cross-module declarations before
whole-program validation and runtime materialization. Within the bounded
bounded import slice, a principal global binding shadows an imported binding of
the same supported binding name, and a later sibling import shadows an earlier
one. Shadowed defaults are not retained for runtime evaluation. Imported named
templates are linked before reference validation;
duplicate named-template precedence remains outside the current slice. An
imported simplified stylesheet's implicit template is normalized into an
ordinary lower-precedence document-matching rule. It therefore participates in
normal template selection and `xsl:apply-imports`, rather than acquiring the
principal module's direct root-template execution privilege.

The prepared instruction representation may fold a statically validated result
construction into its owning literal element when the operation cannot be
observed independently. The current bounded example is a leading,
statically-named `xsl:attribute` with a variable value: compilation retains it
as a distinct computed-attribute feature, while execution materializes it before
children without exposing a mutable result-node API.

Compilation may eventually attach required navigation, retention, buffering,
or evaluation capabilities to normalized expressions and templates. This is a
reserved ownership seam, not an accepted metadata schema or requirement to
implement formal streamability analysis in the first profile. One semantic
compiler should remain authoritative if later tree, streaming, or hybrid
execution strategies are admitted.

### Runtime

The runtime owns per-transformation evaluation context and result production.
Resource resolution, extension behavior, cancellation, limits, and output sinks
are host-supplied capabilities rather than ambient authority.

Result production has two conceptual phases:

```text
transformation result model -> serialization -> bytes, text, or output sink
```

The first slice may implement them together, but tests and future APIs must be
able to distinguish semantic result correctness from serialization correctness.
The private string result remains UTF-8-only. A separate bounded byte lane
admits UTF-8, including an explicitly requested three-byte UTF-8 byte-order
mark, deterministic BOM-prefixed UTF-16BE for the `UTF-16` label, the ASCII
subset of ISO-8859-1, and one bounded US-ASCII XHTML CDATA profile. The
US-ASCII profile closes CDATA around each nonrepresentable
character, emits an uppercase hexadecimal character reference, and reopens the
section; non-ASCII content elsewhere remains unsupported. It charges marks,
declarations, body bytes, and encoding expansion and rejects non-ASCII
ISO-8859-1 output rather than substituting characters or returning a mislabeled
UTF-8 string. This is executable
serialization evidence, not selection of a public byte-result contract or
general encoding support. Compiled output metadata may explicitly retain
`normalization-form="none"`, which preserves result characters byte-for-byte,
or `normalization-form="NFC"`/`"NFD"`, which uses an exact-pinned UAX #15
implementation during private XML, XHTML, HTML, and text serialization.
Character mapping precedes requested normalization and mapped replacement
strings bypass it; CDATA-selected text normalizes before CDATA construction.
NFKC, NFKD, and fully-normalized output remain unsupported.
The XML-compatible lane also retains canonical standalone `yes`, `no`, and
`omit` metadata; `yes` and `no` become declaration pseudo-attributes, while
`omit` emits none. An explicit serialization version is retained separately
from the stylesheet language version; the current bounded lane admits and emits
only XML `1.0`. XHTML-only content-type metadata remains inert for XML output.
An unused, uniquely named output declaration may be validated separately from
the unnamed principal declaration and has no effect on implicit final-result
serialization. Duplicate named-declaration merging, named character-map
resolution, a retained named-format table, and `xsl:result-document` remain
outside the private compiled slice.
An explicit XML, XHTML, or HTML output declaration may carry a valid
`escape-uri-attributes` boolean, which the compiled output settings retain.
The property has no effect on XML output. The bounded XHTML and HTML lanes
recognize only an unnamespaced `href` on an XHTML or null-namespace element as
URI-valued: when enabled or defaulted, the complete value first normalizes to
NFC, then each non-ASCII character becomes its uppercase percent-escaped UTF-8
bytes while existing ASCII percent sequences remain unchanged; when disabled,
ordinary XML-compatible attribute escaping applies. Character maps do not
rewrite an enabled URI-expansion path. The property remains unsupported for an
absent output method. The wider HTML/XHTML URI-attribute vocabulary remains
outside this slice. A separate bounded XPath `escape-html-uri()` path
constant-folds one single-quoted literal used by a leading computed attribute.
It preserves printable ASCII and percent-escapes every other character's UTF-8
bytes without Unicode normalization. This remains distinct from
serializer-owned URI attribute escaping and does not admit dynamic arguments or
general function dispatch.
Invalid boolean values on admitted `xsl:output` properties are static
stylesheet errors reported as `XTSE0020`, with the structured invalid category
and stylesheet source location preserved. XSLT 2.0 accepts only `yes`/`no`;
XSLT 3.0 additionally accepts the exact lower-case `true`/`false` and numeric
`1`/`0` forms after whitespace normalization.
Requested indentation currently adds newline plus two-space depth prefixes only
around non-empty element-only child sequences. Text-only and mixed-content
elements remain inline so indentation does not alter their string values; wider
pretty-printing choices remain implementation-defined and unclaimed.
The private HTML 5 lane admits one no-namespace document element with a bounded
HTML/SVG/MathML vocabulary and a fixed set of non-URI attributes. Its standard
void-element list serializes without end tags. Known XHTML, SVG, and MathML
element prefix bindings normalize to the required default namespace. A
qualified attribute retains the exact prefix binding it consumes, and an
admitted arbitrary foreign namespace retains its prefix. HTML URI attributes, raw-text
handling, arbitrary element and attribute vocabularies, general namespace
fixup, and other HTML versions remain outside this successful slice.
An independent HTML 5 character-map slice admits only the unchanged
`doc/a/@value` corpus shape and applies the compiled map to both text and
attribute values; it does not widen the ordinary HTML 5 element vocabulary.
The explicit HTML serialization version 5 path emits C1 controls (`#x7F` through
`#x9F`) as hexadecimal numeric character references for the bounded corpus
shape. A separate exact source-free HTML 5 lane admits one empty `input` with
the corpus `type` and `value` attributes, emits void-element syntax, and keeps
the non-URI value out of URI percent encoding. HTML 4 serialization remains
excluded from the selected profile, and an
environment-supplied default HTML version is not inferred by this evidence.
`suppress-indentation` is retained as a list of expanded element names in
compiled output settings and merged by expanded-name identity. When indentation
is enabled, the serializer does not add indentation inside a matching element
or any descendant in that element's subtree. The current unchanged corpus
evidence covers an unnamespaced `p` in bounded HTML 5 and XML-compatible XHTML
results and both unprefixed `p` and namespace-qualified `z:p` in XML output; it
does not claim a general pretty-printing or word-wrapping algorithm.
XML-compatible attribute serialization emits C1 controls as hexadecimal numeric
character references. XHTML evidence covers a bounded non-URI `accesskey`
attribute independently of the inert URI-escaping property; it does not admit
URI attribute escaping.
The predefined XML namespace serializes attributes with the reserved `xml`
prefix without requiring an authored declaration. Other namespaced literal
result attributes remain outside the current private compilation slice.
Static `xsl:comment` content may be literal sequence-constructor text or a
compile-time concatenation of single-quoted strings and single-integer
`codepoints-to-string()` calls. Generated codepoints must be XML 1.0
characters; comment data containing `--` or ending in `-` remains unsupported
rather than receiving implicit lexical recovery.
For XHTML output, `include-content-type` defaults to enabled. An XHTML `head`
receives one serializer-owned empty `meta` whose content combines the explicit
media type or `text/html` default with UTF-8; an existing Content-Type meta is
replaced for serialization rather than mutating the semantic result tree.
The bounded HTML lane applies the same enabled-by-default policy to its exact
text-only `HTML/HEAD/BODY` corpus shape, but emits the injected `meta` using
HTML void-element syntax rather than XHTML empty-element syntax. Explicit false
lexicals suppress injection without altering the semantic result tree. That
shape may contain one existing two-attribute Content-Type `meta`; serialization
replaces it with one UTF-8 meta, discarding the authored charset or additional
content parameters without mutating the result tree.
Disabling the property retains authored metadata. This does not extend the
private lane to general HTML serialization. A separate bounded HTML lane admits
only the exact attribute-free `html/head/body/p/del/ins` hierarchy needed to
verify preservation of significant `del` and `ins` text under `indent="no"`;
other HTML hierarchies remain unsupported. Another exact HTML lane admits one
`html/head/script/body` result and emits the manually escaped script value as
raw text. Its bounded extension admits the corpus hierarchy containing one
script and style in the head plus one pre/b and textarea in the body, preserving
their significant whitespace under `indent="no"`; this does not admit arbitrary
script/style or preformatted structures. XHTML `script` and `style` text
continues to use XML escaping. Selected XHTML CDATA elements use the same
expanded-name matching and terminator-splitting behavior as XML output, so a
literal `]]>` becomes adjacent CDATA sections without changing result text.
DOCTYPE system and public identifiers are retained as compiled output metadata.
The current serializer emits SYSTEM or paired PUBLIC declarations only for an
XHTML `html` document element; a public identifier alone is inert. An emitted
identifier containing both quote forms, or any other result shape, remains
explicitly unsupported. Emitted bytes use the normal serialization budget.
Empty XHTML `area`, `base`, `basefont`, `br`, `col`, `frame`, `hr`, `img`,
`input`, `isindex`, `link`, `meta`, and `param` elements use spaced
empty-element syntax. Other empty XHTML elements retain paired tags, and
attribute values are never minimized merely because their names resemble HTML
boolean attributes.
The private sequence-constructor slice admits `xsl:processing-instruction`
with a static NCName target other than reserved `xml` and literal character
data that excludes `?>`. It produces a distinct semantic result node rather
than markup-shaped text, participates in result-node and retained-text work
accounting, serializes as `<?target data?>`, and contributes no characters to
text-method output. Processing instructions do not select an inferred output
method or disqualify an otherwise valid XHTML document element for bounded
DOCTYPE emission. Computed targets/content and PI-terminator recovery remain
explicitly unsupported.
When no output method is declared, an XHTML-namespaced `html` document element
selects the XHTML serializer and its content-type behavior. A null-namespace
`html` selects the still-unsupported HTML method; the two inference rules must
not be conflated.
Unnamed output declarations may now merge only when their scalar properties do
not overlap; repeated scalar properties remain explicitly unsupported until
precedence and conflict semantics are implemented. `cdata-section-elements`
is unioned by expanded QName and affects only serialization of selected result
element text. Named output definitions remain outside the private slice.

### Resource snapshots and volume execution

FastXSLT's preferred volume path is conceptually:

```text
host files, buffers, or application resources
                |
                v
      bounded resource loading
                |
                v
       sealed resource snapshot
          |              |
          v              v
 stylesheet compile   source parse/cache
          |              |
          +------+-------+
                 v
    independent transform requests
                 v
         result set / output sink
```

The host or an adapter owns filesystem, network, database, upload, and refresh
mechanisms. The engine owns qualified resource lookup within the admitted
snapshot and the interpretation of XSLT resource references. A display name,
host path, and content fingerprint are not resource identity.

Under ADR-0002, an adapter finishes host I/O before sealing: it copies admitted
bytes into owned/shared memory and releases source streams and file handles. A
path retained for diagnostics is inert provenance, not a lazy loading token.
Core parsing, compilation, execution, intermediate representations, and default
results do not use memory maps, temporary files, spill files, or persistent disk
caches.

Loading and mutation may occur while constructing a resource set. Compilation
and execution consume a sealed snapshot so repeated requests observe stable
bytes and deterministic lookup. The snapshot must enforce caller-selected entry,
per-entry byte, and aggregate-byte budgets. Physical byte sharing, parse caches,
and content fingerprints are provider optimizations and diagnostics; they do not
merge distinct logical resources.

A sealed snapshot owns immutable admitted resources; it does not imply that
every source is parsed into and retained as a complete tree before a batch
begins. The execution strategy may choose tree construction for the initial
engine. Any future forward-only or hybrid strategy must account for selective
materialization through explicit memory budgets and may not spill to disk
silently.

A closed snapshot cannot satisfy an unadmitted resource discovered dynamically
through `document()`, `unparsed-text()`, collections, includes/imports, or future
extensions. The engine reports an explicit missing/denied-resource condition.
An optional live resolver may be studied later, but must be an explicit
capability and cannot become ambient disk or network access.

Under ADR-0005, a transform set contains only independently executable requests.
Submission, start, execution, and completion order have no semantic meaning;
results correlate by logical request/result identity. One request cannot observe
a sibling result, and producing a result does not mutate or admit a resource.
The host sequences dependent stages and explicitly admits selected prior results
into a later snapshot. Failure collection versus fail-fast behavior,
cancellation, concurrency limits, result retention, and executor mechanics
remain open. Single-transform convenience is semantically a batch of one over
the same resource, compilation, runtime, diagnostic, and result boundaries.

### Diagnostics

Diagnostics cross every layer. They will distinguish phase, stable identity or
standards code where applicable, severity, primary source location, related
locations, and contextual details. Human formatting is an adapter over the
structured form. Lowering and optimization must retain enough provenance to
associate diagnostics with relevant stylesheet, source-document, and
host/resource locations. One event may use one as its primary location and the
others as related locations.

Boundary-facing failures should eventually preserve a stable machine identity,
a small policy category, a human-readable message, structured details, and an
underlying cause where one exists. The concrete vocabulary must be derived from
implemented failure owners rather than declared speculatively. Local modules
may retain focused error types; a single repository-wide mega-enum is not a
goal.

A reportable semantic outcome is not necessarily an operation failure. A
compilation that can reliably report unsupported syntax, or a batch that can
return independently classified request results, should preserve those
findings in its structured result where the public contract permits. Failure to
parse enough input to produce a trustworthy result, denied resource authority,
budget exhaustion, host cancellation, and internal invariant failure remain
distinguishable boundary failures. Presentation adapters must not recover this
meaning by parsing display strings.

### Observability

Observability means that a host can understand engine work without parsing
human diagnostic strings or relying on global hooks. Candidate events and
measurements include compilation and evaluation timing, resource resolution,
template invocation, expression evaluation, message/diagnostic emission, and
budget consumption.

The event vocabulary, cost model, and public interface remain open. Any design
must be opt-in or explicitly supplied, bounded where necessary, and unable to
change template selection, evaluation order where observable, results, or error
semantics.

Hosts also need read-only inspection and explainability without depending on
private engine representation. Candidate semantic snapshots include admitted
resource identities and sizes, the selected standards profile, compiled
stylesheet dependencies, static diagnostics, capability requirements, and
bounded execution summaries. Such a surface must describe supported meaning,
not stabilize a parser AST, arena index, optimizer IR, cache layout, or internal
module boundary. Exact fields and compatibility rules remain open until an
implemented slice and a real consumer exercise them.

## 3. Security and resource policy

XML and XSLT process attacker-controlled recursive structures, names, strings,
regular expressions, URIs, and imported resources. Resource exhaustion and
authority escalation are design inputs, not later hardening tasks.

Capabilities and authority determine what a transform may access:

- document, include, import, collection, and unparsed-text resolution;
- external entities and DTD processing;
- extension functions and any host objects they can access;
- diagnostic access to source text, URIs, and host paths.

Budgets and limits determine how much work it may perform:

- maximum bytes, depth, nodes, sequence growth, recursion, output, and elapsed
  work where enforceable;
- cancellation and host termination signals;
- storage retained by diagnostics, messages, tracing, and result construction.

No resolver should be interpreted as permission to use an ambient default.
Authority failures and budget exhaustion are distinct diagnostic conditions.
Whether each limit is a deterministic semantic counter, a best-effort host
safeguard, or both remains an explicit open decision.

Execution supervision cannot substitute for those local checks. An in-process
dispatcher may coordinate cooperative cancellation and bounded work but cannot
safely terminate and repair an arbitrary Rust thread. Any hard-termination
guarantee requires a discardable process or stronger isolation boundary; the
mode and evidence are tracked by AR-0010.

## 4. Public boundary

The public API remains intentionally empty during M0. The first API must be
shaped by an executable vertical slice and should make these phases visible:

1. create or supply explicit host policy;
2. compile a stylesheet and receive structured diagnostics;
3. reuse the compiled stylesheet with source input and parameters;
4. receive a semantic result plus messages or diagnostics without hidden side
   channels;
5. serialize or otherwise consume that result through an explicit boundary.

Sync versus async execution, input/output ownership, and compiled artifact
thread-safety are open decisions. Thread-safety, reentrancy, and concurrent use
must account for both compiled state and supplied host capabilities; a
thread-safe compiled representation does not make a resolver, message sink, or
output sink thread-safe automatically.

The engine boundary must be usable from Rust without an interop tax. Host
adapters may expose a narrower representation appropriate to their runtime, but
must preserve compiled-stylesheet reuse, explicit ownership, cancellation,
structured failures, and bounded resource access. ASP.NET integration must be
validated from managed caller to completed result rather than inferred from a
Rust-only benchmark.

An ASP.NET host should be able to create or replace a resource snapshot and
compiled stylesheet set outside the per-request hot path, then execute isolated
dynamic contexts concurrently when the accepted concurrency contract permits.

## 5. Verification

The normative testing tiers are defined in
[Testing Strategy](../testing-strategy.md). Passing selected corpus cases proves
only those cases under the recorded environment. Conformance reports must expose
selection and exclusion policy rather than collapsing results into an
unqualified percentage.

ADR-0006 requires suite-native case identity, an explainable disposition for
every discovered case, separate selection and execution outcomes, and conserved
denominators across filtering, sharding, interruption, retry, and merging. It
does not select a ledger schema or storage format, or wording for a published
conformance claim. ADR-0007 selects the staged modern reference editions and
widening discipline without claiming broad conformance.

## 6. Performance

The name FastXSLT is an objective, not evidence. Performance claims require a
named workload, baseline, hardware/software environment, measurement method,
and correctness gate. Optimize measured costs only after the semantic reference
path is observable. Unsafe code remains forbidden unless a later ADR supplies
invariants, evidence, and focused verification. ADR-0008 admits only the
workbench-native .NET boundary's export and bounded buffer-copy surface; it does
not admit unsafe engine semantics or hot-path optimization.
ADR-0015 adds only four read-only scalar registry-pressure observations to that
unpublished surface. They expose counts and exact outcome bytes without
exporting handles, maps, layouts, prepared-engine estimates, or mutation.

ADR-0016 establishes the default that FastXSLT owns semantics, accounting, and
enforcement while hosts own environment-dependent operational numbers. Fixed
standard semantics, safety invariants, ABI constraints, and representation
ceilings remain engine-owned and cannot be weakened by configuration.

As its first concrete application, ADR-0016 requires the native adapter to use
one explicit host-configured,
process-wide admission policy before producing handles. Separate engine,
control, and outcome counts compose with exact aggregate outcome bytes, private
known prepared-engine capacity, and aggregate accounted bytes. Configuration is
immutable for the process, admission and charge release are atomic, and no live
handle is silently evicted. Quota exhaustion is a versioned tagged scalar that
requires no outcome slot. These limits bound FastXSLT-accounted retained
ownership; they are not a CLR, allocator, construction-peak, or whole-process
memory guarantee. Hard memory ceilings and abandonment reclamation require an
externally limited isolated worker.

ADR-0003 defines the exception policy: tests are necessary but cannot prove an
unsafe invariant. Any first-party unsafe implementation requires its own narrow
ADR, a measured need that safe Rust cannot reasonably meet, a written safety
contract, minimized and reviewable surface, safe reference behavior whenever
practical, specialized verification appropriate to the risk, and explicit
removal criteria. ADR-0008, as narrowly extended by ADR-0015's scalar-only
observation exports and ADR-0016's scalar-only policy configuration and status
surface, is the only current exception.

For embedded consumers, measurements include parsing or marshaling, interop,
stylesheet lookup/compilation policy, execution, result transfer, diagnostics,
and serialization as applicable. A faster engine core does not establish a
faster application when boundary conversion dominates the request.

Memory-first execution is expected to reduce repeated filesystem and transfer
cost for reusable workloads, but that benefit must be measured separately from
OS page caching, XML parsing, stylesheet compilation, allocation, and output
cost. Reports must include preload time, warm and cold paths, retained and peak
memory, cache hit rates, and workload reuse.

Avoiding retained and repeated file access is also an operational requirement:
hosts must be able to replace or remove imported files without engine-held
handles or later path probes. This reduces exposure to file locking and
security-tool contention after admission, while making no claim that the host's
initial read or explicit output publication escapes operating-system scanning.

As the language surface widens, compilation should discover the semantic
features a stylesheet requires and select a reusable execution plan that avoids
per-node runtime feature branching for unrelated facilities. This preserves one
general semantic owner while allowing safe specialization. Any later unsafe
hot-path specialization remains a separate ADR-0003 exception with a safe
reference and differential evidence; ADR-0008 authorizes only native FFI copies.

## 7. Open decisions

### M0/M1 architectural decisions

- XML parser and XDM physical representation.
- Intermediate representation shape and stability.
- Sync/async execution, input/output ownership, and initial public boundary.
- Transformation result model, serialization ownership, and comparison rules.
- Resource capability, cancellation, and deterministic/best-effort limit model.
  Resource reference/base semantics, catalogs, dependency closure, live
  authority composition, and resolution-budget ownership are tracked by
  AR-0014; the private exact snapshot resolver does not settle them.
- Thread-safety, reentrancy, and concurrent execution semantics for compiled
  artifacts and host capabilities.
- Initial observability events, cost constraints, and host boundary.
- Structured error/outcome categories and the read-only semantic inspection
  boundary, tracked by AR-0004 and AR-0005.
- Compatibility domains, version identifiers, and whether FastXSLT ever admits
  persisted compiled artifacts, tracked by AR-0006.
- ASP.NET/.NET integration, deployment, ownership, cancellation, and artifact
  boundary, tracked by AR-0002.
- ADR-0002 fixes logical memory-resident resource admission and sealed snapshot
  authority. ADR-0005 fixes unordered independent execution and host-owned
  workflow ordering; transformation graphs are deferred. Exact public lifecycle
  shapes remain unstabilized, while prepared-input definition, retention, and
  cache lifecycle are tracked by AR-0009.
- Execution supervision, cooperative cancellation/deadline observation, panic
  containment, worker health, and the process boundary required for hard
  termination, tracked by AR-0010.

### Deferred capability decisions

- `no_std` and CLI requirements.
- A presealed, memory-resident WASM embedding profile, target/runtime choice,
  host boundary, operational guarantees, and parity evidence, tracked by
  AR-0015.
- Streaming and incremental execution, including any XSLT streaming-conformance
  claim, tracked as architectural optionality by AR-0007.
- Schema awareness and typed values beyond the initial profile.
- Packages, extension functions, and alternate execution backends.

## 8. Architecture decision policy

Decisions intentionally left open by this document must not be resolved
implicitly through implementation. Material choices affecting semantic
ownership, public boundaries, security authority, concurrency guarantees,
replaceability, or conformance claims require an Architectural Review and an
accepted ADR before they become project contracts.

A private experiment may test an alternative when its scope and reversibility
are explicit. It does not acquire architectural authority merely because it
compiles, performs well, or is convenient to reuse.
