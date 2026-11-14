# The plan of scsynth-synthdef

Version 0.5 is done. As its plan said, its decisions and facts moved to the
tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.5.

## Next

The work after version 0.5, in the order of its value for sc-app3. The
README lists the same items under "Left out".

1. nested arrays in array arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains. Each file of
   `tests/fixtures/before_optimize` and the file of its name in
   `tests/fixtures` are a graph before and after `optimizeGraph`.
3. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, the
   specs of the metadata, and `LagControl`
4. `Env`, `VarLag`, and the pseudo-UGens that wait for items 3 and 4:
   `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
5. sc3-plugins

## sc-app3

sc-app3 takes version 0.5 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.5.0`, on a branch of sc-app3. Its
core then adds `BufWr` through `ugen`.
