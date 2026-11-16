# The plan of scsynth-synthdef

Version 0.10 is done. As its plan said, its decisions and facts moved to
the tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.10.

## Next

The work after version 0.10, in the order of its value for sc-app3. The
README lists the same items under "Left out". `PackFFT.new` comes first,
because the markup of sc-app3 reaches it with `UnpackFFT` and expressions.
The markup has no functions, which the `pvcalc` methods take.

1. `PackFFT.new`, whose inputs follow a count, and the `pvcalc` methods of
   `PV_ChainUGen`
2. The methods of `UGen` that build graphs, such as `range`, `exprange` and
   `linlin`, which need the `signalRange` of each class, and with them
   `range`, `exprange` and `curverange` of `Env`
3. sc3-plugins

## sc-app3

sc-app3 takes version 0.10 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.10.0`, on a branch of sc-app3. A
new test of sc-expression compiles the markup of 16 `SinOsc` into an `Out`
against `wirebufs_sixteen`, and counts 16 wire buffers. A pump that refuses
a definition over `-w` gets a plan of its own in sc-app3, because scsynth
does not report its `-w`.
