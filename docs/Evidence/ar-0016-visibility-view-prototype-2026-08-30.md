# AR-0016 Visibility-View Prototype

Date: 2026-08-30

## Scope

This record evaluates one private, safe visibility-view prototype against
AR-0016's complete-derived-document semantic reference for exact
`xsl:strip-space elements="*"`. It does not select a public source-view API,
general whitespace-declaration semantics, a retained cache, or a performance
guarantee.

## Representation

Prepared `Document` node storage is now immutable shared storage. A stripping
invocation scans the source under its existing `XdmNode` budget and
cancellation control, then retains only replacement child sequences for
elements that contained strip-eligible whitespace text. Every visible node
continues to use the prepared document's `NodeId`, payload, parent, document
order, namespaces, and source location. The view is invocation-owned and is
dropped after execution.

The private `Document` access surface remains concrete. XPath, template
selection, built-in rules, values, and copying did not acquire a generalized
provider trait. This is the smallest complete seam identified by the AR-0016
source-access inventory.

## Differential controls

- Every current `Document` accessor is compared node-by-node between the
  complete clone and the visibility view, including relationships and
  containing string values.
- The first comparison found that controlled string-value recursion still read
  physical child storage directly. That leak was moved behind the effective
  `children` accessor before the view entered production execution.
- One full stripping transform executes through both representations and
  produces the same semantic result.
- The unchanged pinned XSLT30 `mode-1301` case passes through the visibility
  path.
- One prepared source is executed 100 times concurrently under preserving and
  stripping stylesheets. Results remain stable and the prepared source remains
  unchanged afterward.
- A focused runtime stylesheet proves that whitespace-only siblings do not
  contribute to `position()` or `last()` and that `xsl:copy` traverses the same
  effective children. The complete reference and view produce byte-identical
  serialized results with positions 1 through 3 rather than the physical
  whitespace-interleaved positions.
- A descendant `node()` selection independently proves that stripped text is
  removed before focus positions are assigned. The five visible element/text
  descendants report positions 1 through 5 and `last() = 5` through both the
  reference and view.
- Old stripping and replacement preserving stylesheet generations execute
  concurrently against the same prepared source. Each retains its own policy
  and result while the prepared source remains unchanged.
- Zero-node budgets and deterministic cancellation stop view construction at
  real `XdmNode` charge points.

No sibling axis is implemented in the current XPath surface; a future sibling
axis must receive the same effective-sequence control when admitted.

## Preliminary local measurement

Command:

```text
cargo test --release -p fastxslt --lib measures_whitespace_reference_against_visibility_view -- --ignored --nocapture
```

Toolchain: Rust 1.95.0, `x86_64-pc-windows-msvc`, LLVM 22.1.2.

The generated source contains 500 item elements separated by indentation. Each
sample performs 2,000 complete warm transforms, and seven samples are reduced
to the median:

| Candidate | Median invocation | Additional owned-capacity estimate |
| --- | ---: | ---: |
| Complete reference | 160,725.0 ns | 574,479 bytes |
| Visibility view | 33,076.4 ns | 4,072 bytes |

For this private workload, the view was 4.86 times faster and its attributable
additional-capacity estimate was about 141 times smaller. Both candidates scan
the source once per invocation and execute the same value-producing transform.

These byte values are implementation-owned capacity estimates, not allocator
or process peak-memory measurements. The view figure excludes prepared node
storage because that storage already belongs to the reusable prepared input;
the reference figure is the additional full clone. The timing is one local
microprobe, not a CI threshold, host-boundary result, or general XSLT
performance claim.

## Disposition

At this checkpoint, use the visibility view as the private executable candidate
for the admitted strip-all policy while retaining the complete safe clone as a
differential oracle. Do not retain views across invocations or generations.

The subsequent
[decision measurement matrix](ar-0016-decision-measurement-matrix-2026-08-30.md)
completed the source-shape, concurrency, and allocator-requested memory work.
AR-0016 was then accepted through ADR-0012 for the exact strip-all policy;
broader whitespace semantics remain deferred.
