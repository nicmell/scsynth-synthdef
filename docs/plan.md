# The plan of scsynth-synthdef

Version 0.14 is done. As its plan said, its decisions and facts moved to
the tests, the README and the CHANGELOG. This file keeps only what is still
open. The history of git holds the plans of versions 0.1 to 0.14.

## Next

The work after version 0.14, in the order of its value for sc-app3. The
README lists the same item under "Left out".

1. The bodies of the other pseudo-UGens of sc3-plugins, in version 0.15.
   - The 30 classes outside `UGen` that build UGens with the arithmetic of
     the language:
     - biquad filters on `SOS`: `HighShelf`, `LowShelf`, `Notch` and the
       three RBJ classes
     - other filters: `ZLPF`, `ZHPF`, `BLPF`, `BHPF`, `LRLPF` and `LRHPF`
     - filters on `FOS`: `BhobHiShelf` and `BhobLoShelf`
     - reverbs: `FreeVerb1`, `BFFreeVerb` and `BFGVerb`
     - analysis and synthesis: `Vocode`, `VocodeBand`, `Vocoder`,
       `WarpOverlap`, `BtoUHJ` and `UHJtoB`
     - the others: `AutoTrack`, `DelayWr`, `Feedback`, `TapN`, `Phaser`,
       `PulseDPW` and `StkBowedI`
   - The methods of UGen classes that `ugen` refuses because they do more
     than `new1`, but `OnsetsDS.kr` and the 7 classes without a unit. FM7
     also needs a rule for its count of inputs, and `StkFlute.kr` a body
     that applies `madd` twice, as sclang does.
   - `UGen:circleRamp` for `send`.

## sc-app3

sc-app3 takes version 0.14 by a move of its submodule
`crates/scsynth-synthdef` to the tag `v0.14.0`, on a branch of sc-app3.
The feature `sc3-plugins` stays off there. Turning it on needs a plan of
sc-app3:

- sc-expression finds a class with `catalog::lookup` and
  `catalog::pseudo_lookup`, and lists `catalog::all` in an error. It needs
  the lookups of `sc3_plugins` too.
- The scsynth that sc-app3 starts needs the plugins of sc3-plugins.
- The feature adds 258 KB to a release wasm32 build, and 28.5 KB after
  gzip.

The expression functions of the methods of `UGen`, such as
`range(x, lo, hi)`, still wait for a plan of their own in sc-app3.
