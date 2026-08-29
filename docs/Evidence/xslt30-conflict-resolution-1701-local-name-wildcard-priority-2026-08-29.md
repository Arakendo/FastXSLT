# XSLT30 `conflict-resolution-1701` Local-Name Wildcard Priority

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1701`
- Stylesheet: `conflict-resolution-1701.xsl`
- Source: `conflict-resolution-17.xml`

## Trigger and representation review

The prior template-pattern decomposition review required reopening when
namespaces or another wildcard grammar expanded the private owner. This case
introduces `*:NCName` and requires its implicit priority of `-0.25` to compete
with expanded-name rules carrying explicit priorities `-0.1` and `-0.4`.

The existing doubled-integer priority representation could express integers
and half steps but not quarter steps or the corpus decimals. It is replaced by
an exact signed fixed-point domain storing millionths. Standard `0.5`, `0`,
`-0.25`, and `-0.5` defaults and explicit values with at most six fractional
digits compare as integers; binary floating point does not enter selection.
Lexically valid values outside the bounded precision/range remain structured
unsupported outcomes (`FXST1025`), while malformed lexicals remain invalid
(`FXST0030`).

## Method

The pattern compiler lowers `*:b` to a typed local-name element pattern. The
source-XDM selector compares the local component while deliberately ignoring
namespace identity. Expanded-name patterns still compare both namespace and
local name. Retained exact priorities and source order select among applicable
rules.

The first-party helper reads the file named by the pinned environment, closes
that handle, admits the bytes to a bounded sealed snapshot, and performs no
ambient file access during compilation or execution.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1701` | `,Fully-qualified-1:ed-b,Localnamed-b:co-b,Localnamed-b:bz-b` | equal | passed |

The `ed:b` expanded-name rule at `-0.1` outranks the local wildcard; the local
wildcard at `-0.25` outranks the `co:b` rule at `-0.4` and also matches `bz:b`.

## Ownership and claim boundary

The 248-line private pattern owner retains lexical recognition, normalization,
default-priority assignment, bounded explicit-priority parsing, and invalid
versus unsupported classification. Runtime selection retains applicability and
ordering only. No parser, runtime, host, resource, or public boundary moved.

This evidence admits ASCII `*:NCName`, implicit namespace/local wildcard
quarter-step priority, and bounded six-place explicit decimals on non-root
patterns. It does not admit arbitrary-precision decimals, wildcard attributes,
EQNames, Unicode QName grammar, union patterns, document kind tests,
import/package precedence, or ambiguity policy. Root-pattern priority is
evidenced separately by `conflict-resolution-1601`.
