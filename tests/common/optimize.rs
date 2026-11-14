//! The graphs of the golden files of 0.7, which `finishBuild` rewrites.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{SynthDef, SynthDefBuilder, UGenInput};

use super::by_name;
use super::pseudo::{def, input, out, sine};

/// The definitions of a golden file of 0.7.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "pvcopy" => pvcopy(),
        _ => panic!("no golden graph of 0.7 is named {name}"),
    }
}

/// `FFT(buffer, SinOsc.ar)`.
fn chain_of(b: &mut SynthDefBuilder, buffer: UGenInput) -> UGenInput {
    let sound = sine(b, "ar", 440.0);
    by_name(
        b,
        "FFT",
        "new",
        &[("buffer", input(buffer)), ("in", input(sound))],
    )
}

/// `FFT(LocalBuf(512), SinOsc.ar)`.
fn local_chain(b: &mut SynthDefBuilder) -> UGenInput {
    let buffer = by_name(b, "LocalBuf", "new", &[("numFrames", input(512.0))]);
    chain_of(b, buffer)
}

/// A PV UGen of `chain` with a threshold of 1, as `PV_MagAbove(chain, 1)`.
fn pv(b: &mut SynthDefBuilder, class: &str, chain: UGenInput) -> UGenInput {
    by_name(
        b,
        class,
        "new",
        &[("buffer", input(chain)), ("threshold", input(1.0))],
    )
}

fn ifft(b: &mut SynthDefBuilder, chain: UGenInput) -> UGenInput {
    by_name(b, "IFFT", "ar", &[("buffer", input(chain))])
}

fn binary(b: &mut SynthDefBuilder, op: &str, a: UGenInput, c: UGenInput) -> UGenInput {
    b.binary_op(op, a, c).expect("the operator adds")
}

fn pvcopy() -> Vec<SynthDef> {
    vec![
        def("pvcopy_three", vec![], |b| {
            let chain = local_chain(b);
            let pvs: Vec<UGenInput> = ["PV_MagAbove", "PV_MagBelow", "PV_MagClip"]
                .iter()
                .map(|class| pv(b, class, chain.clone()))
                .collect();
            let sounds = pvs.into_iter().map(|pv| ifft(b, pv)).collect();
            out(b, "ar", UGenInput::Array(sounds));
        }),
        def("pvcopy_slots", vec![], |b| {
            let chain = local_chain(b);
            let product = by_name(
                b,
                "PV_Mul",
                "new",
                &[
                    ("bufferA", input(chain.clone())),
                    ("bufferB", input(chain.clone())),
                ],
            );
            let sounds = vec![ifft(b, product), ifft(b, chain)];
            out(b, "ar", UGenInput::Array(sounds));
        }),
        def("pvcopy_deep", vec![], |b| {
            let first = local_chain(b);
            let chain = pv(b, "PV_MagAbove", first);
            let left = ifft(b, chain.clone());
            let right = ifft(b, chain);
            let sum = binary(b, "+", left, right);
            out(b, "ar", sum);
        }),
        def("pvcopy_analysis", vec![], |b| {
            let chain = local_chain(b);
            let above = pv(b, "PV_MagAbove", chain.clone());
            let sound = ifft(b, above);
            let centroid = by_name(b, "SpecCentroid", "kr", &[("buffer", input(chain))]);
            let product = binary(b, "*", sound, centroid);
            out(b, "ar", product);
        }),
        def("pvcopy_buffer", vec![], |b| {
            let buffer = by_name(b, "LocalBuf", "new", &[("numFrames", input(16.0))]);
            let chain = chain_of(b, 0.0.into());
            let above = pv(b, "PV_MagAbove", chain.clone());
            let left = ifft(b, above);
            let below = pv(b, "PV_MagBelow", chain);
            let right = ifft(b, below);
            let play = by_name(
                b,
                "PlayBuf",
                "ar",
                &[("numChannels", input(1.0)), ("bufnum", input(buffer))],
            );
            out(b, "ar", UGenInput::Array(vec![left, right, play]));
        }),
    ]
}
