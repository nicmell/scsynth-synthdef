# The plan of scsynth-synthdef

Version 0.12 is done. As its plan said, its decisions and facts moved to
the tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.12.

## Next

The work after version 0.12, in the order of its value for sc-app3. The
README lists the same item under "Left out".

1. sc3-plugins

## sc-app3

sc-app3 takes version 0.12 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.12.0`, on a branch of sc-app3.
Its markup reaches classes, not methods. So the expression functions of
the new methods, such as `range(x, lo, hi)`, get a plan of their own in
sc-app3.
