# XSLT30 Mode Denominator and QName Identity

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- First-party overlay: `corpus/overlays/xslt30/mode-denominator-v0.toml`
- Selected cases: `mode-0105`, `mode-0106`, `mode-0107`, `mode-0108`

## Denominator

An executable inventory parses the pinned native test-set document, conserves
169 uniquely named cases from `mode-0001` through `mode-1905`, and requires the
overlay's default `harness-unsupported / not-run` disposition. Exactly four
case-specific overrides are currently selected and passed. Unexecuted cases
remain visible and are not classified as engine failures.

## Executable behavior

FastXSLT now resolves lexical QName mode names against the namespace context of
each `xsl:template` or `xsl:apply-templates` instruction. The compiled private
identity uses canonical expanded form; for these cases `foo:a` becomes
`Q{http://foo.com}a`, while unprefixed `a` remains in no namespace.

Both pinned cases use the same source and declare rules for `a`, `foo:a`, and
the default mode:

- `mode-0105` applies `foo:a` and produces `mode-foo:a:a-text`;
- `mode-0106` applies unprefixed `a` and produces `mode-a:a-text`.

Their native `assert-xml` results pass through the sealed in-memory resource,
compile, and transform-set path. The qualified namespace declaration is also
preserved on the literal result element.

`mode-0107` adds a distinct focus/lifecycle slice. The bare `$x` selection
resolves the global temporary-tree variable as its document node, dispatches a
moded document rule, and then applies templates from temporary focus rather
than accidentally returning to the principal source. Local temporary-tree
variables use the same selection form and shadow globals during runtime lookup.

`mode-0108` changes focus through the deliberately bounded
`xsl:for-each select="$x"` form. The variable again identifies a temporary-tree
document, the instruction body retains the surrounding current-template
context, and its explicit mode dispatch reaches the temporary `x` element
rule. Other `xsl:for-each` selections remain outside the private slice.

## Subsequent temporary-text prerequisite

A focused first-party execution test now retains non-whitespace text children
inside attribute-free constructed elements. Invocation materialization charges
each text node as XDM work, built-in temporary-tree traversal preserves mixed
element/text document order, and result construction charges both the result
node and retained UTF-8 text bytes. The exercised tree
`<x>head<y>middle</y>tail</x>` produces `headmiddletail` through built-in rules.

This closes only the retained-text representation prerequisite for
`conflict-resolution-1401`; it does not select that pinned case or change the
four-case mode denominator.

A second focused prerequisite now compiles the pinned case's exact
`$dummy/db:book/db:chapter/db:info/db:title` selection into a variable identity
plus expanded-name steps. Runtime navigation begins at the temporary document's
roots, visits child elements in stored document order, charges every inspected
node as XPath work, and dispatches only the selected `db:title`. The focused
stylesheet produces `ChapterTitle`, excluding the sibling book title.

## Architectural consequence

Mode identity is semantic QName identity rather than raw prefix spelling. This
removes one independent blocker from `conflict-resolution-1401`. The added
temporary-document focus behavior removes another, but does not claim that
case: its union match pattern and temporary-focus `xsl:next-match` still require
dedicated work.

## Claim boundary

Four passes out of 169 are not a mode-conformance percentage. This slice does
not select packages, streamability, typed modes, public initial-mode QName
syntax, or the many remaining mode semantics represented by the denominator.
