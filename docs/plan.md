# The plan of scsynth-synthdef

Version 0.7 is done. As its plan said, its decisions and facts moved to the
tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.7.

## Next

The work after version 0.7, in the order of its value for sc-app3. The
README lists the same items under "Left out".

1. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, the
   specs of the metadata, and `LagControl`
2. `Env`, `VarLag`, and the pseudo-UGens that wait for items 1 and 2:
   `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
3. sc3-plugins

## sc-app3

sc-app3 takes version 0.7 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.7.1`, on a branch of sc-app3. A
new test of sc-expression compiles the markup of `optimize_args` against
the golden file of the crate. The clippy of sc-app3 lints the crate for
`wasm32-unknown-unknown`, which `v0.7.0` fails.
