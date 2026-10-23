# OASIS XSLT 1.0 Detached Cross-Document Node Values

- Date: 2026-09-29
- Status: Verified private runtime and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Four unchanged cases bind source nodes from the principal input and then enter
a literal `document()` root before consuming the binding through XSLT 1.0
scalar semantics:

- `Lotus/idkey_idkey18#1`;
- `Lotus/idkey_idkey50#1`;
- `Microsoft/Keys__91834#1`; and
- `Microsoft/Keys__91835#1`.

The runtime previously rejected all four with `FXRT1017`. Its invocation frame
stored source variables as bare `NodeId` values, so carrying those identifiers
into another document would have confused identity and ownership.

## Bounded implementation

Before a literal-document `xsl:for-each` switches its source context, the
runtime now derives an invocation-owned detached view of each local source-node
binding. The view retains ordered string values and cardinality while the
principal document still owns the node identifiers. Raw `NodeId` values do not
cross the document boundary.

The detached view supports the XSLT 1.0 scalar operations required by this
slice: first-node string conversion, node-set effective boolean value,
`count()`, scalar comparisons, and all ordered lookup values supplied to
`key()`. A path or identity operation still requires an owned source-node
sequence and therefore cannot accidentally navigate the detached values as
nodes in the external document.

Global source-node bindings and source-node template parameters remain outside
the admitted multi-document slice. Resource acquisition remains limited to the
sealed invocation snapshot, and derivation charges the existing controlled
string-value traversal.

## Corpus result

All four unchanged cases now compare exactly. The conserved sweep changes as
follows:

- exact matches rise from 2,279 to **2,283 / 3,173 (71.95%)**;
- successful executions rise from 2,411 to **2,415**;
- execution failures fall from 48 to **44**;
- initialized cases remain **2,459**;
- initialization failures remain **711**;
- visible mismatches and comparator gaps remain **zero**; and
- expected-error credit remains **423 / 431**.

## Boundaries

This evidence does not establish portable cross-document node identity,
general XPath data-model provenance, source-node globals or parameters across
document switches, live resource acquisition, or a public multi-document
value representation. The complete owning document remains required for node
navigation and identity-sensitive operations.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_literal_document_root_
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'unsupported/FXRT1017/FXRT1017'
./scripts/verify.ps1
```
