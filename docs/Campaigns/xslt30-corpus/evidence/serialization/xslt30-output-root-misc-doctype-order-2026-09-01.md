# XSLT30 root miscellaneous nodes and doctype order -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0234` case. A statically
constructed root comment and processing instruction serialize in result order,
followed by the configured XML doctype immediately before the document element.

The implementation adds a private comment result-node kind and charges both its
node and retained text bytes through the existing invocation control. Doctype
emission now occurs when serialization reaches the first top-level element
rather than before all result children.

## Boundary conservation

The admitted `xsl:comment` form has literal text content only. Computed content
and comment data requiring lexical recovery remain explicitly unsupported.
Comments do not contribute to text-output string value. This slice does not
admit HTML serialization or `output-0233`, whose HTML method remains outside the
current bounded character-map-only HTML lane.

## Denominator movement

The complete output denominator moves from 134 to 135 passes and from 98 to 97
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 332 to 333 passes, with 3 engine unsupported cases, 50
profile exclusions, and 145 visible default not-run cases.
