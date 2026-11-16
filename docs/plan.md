# The plan of scsynth-synthdef

Version 0.11 is done. As its plan said, its decisions and facts moved to
the tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.11.

## Next

The work after version 0.11, in the order of its value for sc-app3. The
README lists the same items under "Left out".

1. The methods of `UGen` that build graphs, such as `range`, `exprange` and
   `linlin`, which need the `signalRange` of each class, and with them
   `range`, `exprange` and `curverange` of `Env`
2. sc3-plugins

## sc-app3

sc-app3 takes version 0.11 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.11.0`, on a branch of sc-app3. A
new test of sc-expression compiles the markup of `packfft_markup` against
the golden file of the crate. The `pvcalc` methods do not reach the
markup, which has no functions.
