# QT3 `Axes074`–`Axes079` Internal-Descendant Child Expansion

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes074` through `Axes079` groups (23 cases)
- Environments: `TreeTrunc`, `Tree1Text`, `Tree1Child`, `TreeCompass`,
  `TreeStack`, and `TreeRepeat`
- Forms: explicit and abbreviated element, named-element, and node child tests
  following an internal `//`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all 23 cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

The parser now lowers one isolated internal `//` separator to the typed
`descendant-or-self::node()` step defined by the XPath abbreviation. The
following written child step remains independently typed. Explicit and
abbreviated child wildcards, names, and node tests therefore share evaluator
semantics rather than using a string-level descendant shortcut. Malformed
repeated separators remain unsupported.

The evaluator already retains one result per XDM identity in document order
when descendant-or-self expansions from nested contexts overlap. A focused
source selects nested `center` elements, expands both contexts, and returns the
same three descendant elements for explicit and abbreviated child syntax. The
exact 14-node work charge retains repeated traversal work even though duplicate
result identities are removed.

## Result

| Group | Cases | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes074` | 4 | 4 | passed |
| `Axes075` | 4 | 4 | passed |
| `Axes076` | 4 | 4 | passed |
| `Axes077` | 3 | 3 | passed |
| `Axes078` | 4 | 4 | passed |
| `Axes079` | 4 | 4 | passed |
| **Complete denominator** | **23** | **23** | **passed** |

The admitted `Axes001` through `Axes079` selections now contribute 164 passing
location-path cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed child-axis forms after one internal
descendant abbreviation. It does not establish attribute, self, parent, or
other axes after that abbreviation; repeated separators; internal-descendant
forms composed with the current existence-predicate grammar; generalized path
normalization; namespace-sensitive name tests; or a general XPath parser.
