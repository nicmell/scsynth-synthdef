# The plan of scsynth-synthdef

Version 0.8 is done. As its plan said, its decisions and facts moved to the
tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.8.

## Next

The work after version 0.8, in the order of its value for sc-app3. The
README lists the same items under "Left out".

1. `Env`, `VarLag`, and `EnvGate`, which waits for `Env`
2. The `pvcalc` methods of `PV_ChainUGen`, which take a function and need
   `PackFFT`
3. sc3-plugins

## sc-app3

sc-app3 takes version 0.8 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.8.0`, on a branch of sc-app3. A
new test of sc-expression compiles markup with
`<sc-ugen type="NamedControl">` against the golden graph
`namedcontrol_markup`. `<sc-control>` gets a plan of its own in sc-app3.
