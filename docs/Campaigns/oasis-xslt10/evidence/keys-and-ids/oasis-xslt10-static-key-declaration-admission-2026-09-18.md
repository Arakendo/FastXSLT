# OASIS XSLT 1.0 Static Key-Declaration Admission

## Question

Can FastXSLT move the complete 90-case `xsl:key` declaration frontier to its
real XPath and runtime boundaries without prematurely selecting an eager index,
a prepared-input cache, or a public key representation?

## Method

- Retain every admitted key declaration as compiled stylesheet-static state.
- Resolve the key name as an expanded QName at compilation.
- Compile the bounded match pattern and `use` location path once.
- Compose declarations across admitted include/import module graphs; declarations
  sharing one expanded name remain additive.
- Reject variable references and recursive `key()` calls in `use` as static
  `XTSE1205` errors.
- Keep lookup/index construction unimplemented so the next measurement exposes
  the real `key()` expression shapes.
- Run the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The blanket
`FXST1002 / unsupported top-level XSLT declaration: xsl:key` frontier falls
from 90 cases to zero. Three unchanged cases execute and compare exactly,
raising the strict lower bound from 1,569 to 1,572.

The aggregate lifecycle moves from 1,952 initialized / 1,850 executed to 1,955
initialized / 1,853 executed. Initialization failures fall from 1,183 to 1,180;
execution failures remain 102. Expected-error accounting remains conserved at
397 initialization observations, 26 execution observations, and five
unexpected successes.

The dominant newly visible key-related boundary is now 85
`FXXP1001 / location-path-shape:function` cases. Other cases expose existing
predicate, copy, match-pattern, whitespace, module, and static-error boundaries.
This is declaration admission and honest frontier movement, not `key()` support.

## Ownership conclusion

Key declarations belong to immutable compiled stylesheet state. A future lookup
index must be derived from one source document for one invocation (or another
separately reviewed immutable prepared representation); this tranche does not
create an eager index, global cache, cross-snapshot sharing, or ambient resource
access. Known prepared-engine capacity includes the retained names, patterns,
paths, and source locations.

