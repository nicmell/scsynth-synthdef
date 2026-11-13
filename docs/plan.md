# The plan of scsynth-synthdef

Version 0.3 is done. As its plan said, its decisions and facts moved to the
tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1, 0.2 and 0.3.

## Next

The work after version 0.3, in the order of its value for sc-app3. The
README lists the same items under "Left out".

1. nested arrays in array arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains. Each file of
   `tests/fixtures/before_optimize` and the file of its name in
   `tests/fixtures` are a graph before and after `optimizeGraph`.
3. what rate methods do besides `new1`: the Lag family, `LocalBuf`,
   `LocalIn`, `Poll` and the zeros of `Out`
4. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs` and the
   specs of the metadata
5. `Env`, and the pseudo-UGens that wait for items 3 to 6: `Dunique`,
   `PingPong`, `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
6. the rules of `checkInputs` that classes define for themselves
7. sc3-plugins

## sc-app3

sc-app3 takes version 0.3 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.3.0`. That work is a branch of
sc-app3, with its own plan.
