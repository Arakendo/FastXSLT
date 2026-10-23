# OASIS XSLT 1.0 Invocation-Owned Message Evidence

Date: 2026-09-23

## Question

Can the XSLT 1.0 compatibility path execute `xsl:message` without introducing
ambient logging, allowing message content to enter the principal result, or
selecting a public message-sink API before the public Rust lifecycle exists?

## Implementation

- The compiler retains an XSLT 1.0-only typed message instruction containing
  the compiled sequence constructor and the static `terminate` decision.
- The runtime instantiates the message constructor under the current focus and
  variable frame, derives its string value from descendant text, and records
  that value on the invocation control. Message nodes never enter the
  principal result tree.
- `terminate="yes"` records the message first and then returns structured
  dynamic error `XTMM9000`. `terminate="no"` and the absent default continue
  the invocation.
- Message construction uses the existing XSLT-instruction, result-node, and
  result-text-byte charge points. A result-text budget failure occurs before a
  message is recorded, so retained message state cannot bypass invocation work
  accounting.
- Modern-profile `xsl:message`, a public observer/sink spelling, adapter
  delivery, and independent message-retention limits remain unselected.

## Focused verification

Focused compiler/runtime tests establish that:

- absent, `no`, and `yes` terminate values compile only in the XSLT 1.0
  compatibility path;
- invalid terminate values and unknown attributes keep their existing static
  classifications;
- a non-terminating message is retained by its invocation while its content is
  absent from serialized principal output;
- a terminating message is recorded before exact `XTMM9000` failure; and
- result-text exhaustion prevents message publication.

## Unchanged corpus result

The hash-verified OASIS CD04 sweep moved from:

- 2,210 to 2,233 initialized cases;
- 2,157 to 2,175 successful executions;
- 2,009 to 2,026 exact expected-result matches;
- 53 to 58 visible execution failures; and
- 64 to 65 visible comparison mismatches.

The strict exact-match lower bound is now **2,026 / 3,173 (63.85%)**.

All fifteen unchanged Lotus `message01` through `message15` cases become exact.
Unchanged Microsoft `Messages_XmlFragmentInsideMessage` and `91765_1` also
become exact. Four terminating cases now report `XTMM9000`. The unchanged Lotus
`impincl18` case proceeds beyond `xsl:message` and exposes its later
`xsl:apply-imports` context error. Microsoft `Messages__91758` executes message
semantics correctly but remains a visible serializer-indentation mismatch:
`<out></out>` versus an expected line break inside the empty element.

This is compatibility evidence over one pinned archival suite. It is not a
claim of complete XSLT 1.0 message conformance or a stabilized host-facing
message API.
