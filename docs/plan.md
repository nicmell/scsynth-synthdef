# The plan of scsynth-synthdef

Version 0.1 is done. As its plan said, its decisions and facts moved to the
tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plan of version 0.1.

## Next

The work after version 0.1, in the order of its value for sc-app3. The
README lists the same items under "Left out".

1. `mul` and `add`, with the folds of `BinaryOpUGen:new1`
2. multichannel expansion
3. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains
4. what rate methods do besides `new1`: the Lag family,
   `asAudioRateInput`, `LocalBuf`, `Poll` and the zeros of `Out`
5. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs` and the
   specs of the metadata
6. `Env`, and the pseudo-UGens such as `Mix`, `Splay` and `SoundIn`
7. the rules of `checkInputs` that classes define for themselves
8. sc3-plugins

## sc-app3

sc-app3 takes the crate: the submodule `crates/scsynth-synthdef` with a
relative URL, a workspace member, and the CI step that fetches it. That
work is a branch of sc-app3, with its own plan.
