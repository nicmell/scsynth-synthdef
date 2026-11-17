# The plan of scsynth-synthdef

Version 0.15 is done. As its plan said, its decisions and facts moved to
the tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.15.

## Next

No version is planned after 0.15. The README lists what the crate leaves
out, and why, under "Left out".

## sc-app3

sc-app3 is at version 0.13 of the crate. When the owner asks, it takes
0.15 by a move of its submodule `crates/scsynth-synthdef`. The feature
`sc3-plugins` stays off there. Turning it on needs a plan of sc-app3:

- sc-expression finds a class with `catalog::lookup` and
  `catalog::pseudo_lookup`, and lists `catalog::all` in an error. It needs
  the lookups of `sc3_plugins` too, and `sc3_plugins::ugen_method` for
  `circleRamp`.
- The scsynth that sc-app3 starts needs the plugins of sc3-plugins.
- In a release wasm32 build that calls `ugen`, the feature adds 433 KB,
  and 82 KB after gzip.

The expression functions of the methods of `UGen`, such as
`range(x, lo, hi)`, still wait for a plan of their own in sc-app3.
