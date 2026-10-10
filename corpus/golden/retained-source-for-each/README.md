# Retained-source for-each golden fixture

First-party, hand-authored FastXSLT fixture, distributed under the repository's
MIT license. No external corpus content is copied here. This is regression
evidence, not an XSLT conformance claim.

The host seals `secondary.xml` as `extra.xml` beside the stylesheet. A retained
selection is aliased, sorted numerically, and iterated against its effective
owner. Nested iteration returns to the retained principal document; outer focus
is restored afterward. A key lookup must use each active document's own index.
Stripped whitespace, position/size and colliding local
IDs must not change the result. The exact fixture is exercised by the private
`retained_source_iteration` runtime tests.
