//! The graphs of the golden files of 0.7, which `finishBuild` rewrites.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, SynthDef, SynthDefBuilder, UGenInput};

use super::pseudo::{def, input, out, sine};
use super::{arg, array, by_name};

/// The definitions of a golden file of 0.7.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "optimize" => [mul_adds(), sums()].concat(),
        "deadcode" => deadcode(),
        "pvcopy" => pvcopy(),
        _ => panic!("no golden graph of 0.7 is named {name}"),
    }
}

/// `SinOsc.ar` of each of these frequencies, in order.
fn sines(b: &mut SynthDefBuilder, freqs: &[f64]) -> Vec<UGenInput> {
    freqs.iter().map(|&freq| sine(b, "ar", freq)).collect()
}

fn neg(b: &mut SynthDefBuilder, a: UGenInput) -> UGenInput {
    b.unary_op("neg", a).expect("the operator adds")
}

/// The left fold of `+` over these inputs, as `a + b + c` adds them.
fn plus_all(b: &mut SynthDefBuilder, inputs: Vec<UGenInput>) -> UGenInput {
    let mut inputs = inputs.into_iter();
    let first = inputs.next().expect("an input");
    inputs.fold(first, |sum, input| binary(b, "+", sum, input))
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

fn mul_adds() -> Vec<SynthDef> {
    vec![
        def("optimize_muladd", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let product = binary(b, "*", sound, 0.5.into());
            let sum = binary(b, "+", product, 0.25.into());
            out(b, "ar", sum);
        }),
        def("optimize_muladd_swap", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let product = binary(b, "*", 0.5.into(), sound);
            let sum = binary(b, "+", product, 0.25.into());
            out(b, "ar", sum);
        }),
        def("optimize_muladd_right", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let product = binary(b, "*", sound, 0.5.into());
            let sum = binary(b, "+", 0.25.into(), product);
            out(b, "ar", sum);
        }),
        def("optimize_args", vec![Arg::new("amp", 0.1)], |b| {
            let amp = arg(b, "amp");
            let sound = sine(b, "ar", 440.0);
            let product = binary(b, "*", sound, amp);
            let sum = binary(b, "+", product, 0.5.into());
            out(b, "ar", sum);
        }),
        def("optimize_muladd_kr", vec![], |b| {
            let sound = sine(b, "kr", 440.0);
            let noise = by_name(b, "LFNoise1", "kr", &[]);
            let product = binary(b, "*", sound, noise);
            let sum = binary(b, "+", product, 2.0.into());
            out(b, "kr", sum);
        }),
        def("optimize_shared", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let product = binary(b, "*", sound, 0.5.into());
            let sum = binary(b, "+", product.clone(), 0.25.into());
            out(b, "ar", UGenInput::Array(vec![sum, product]));
        }),
        def("optimize_kr_audio", vec![], |b| {
            let slow = sine(b, "kr", 440.0);
            let product = binary(b, "*", slow, 2.0.into());
            let sound = sine(b, "ar", 440.0);
            let sum = binary(b, "+", product, sound);
            out(b, "ar", sum);
        }),
    ]
}

fn sums() -> Vec<SynthDef> {
    vec![
        def("optimize_sum3", vec![], |b| {
            let first = sines(b, &[1.0, 2.0]);
            let pair = plus_all(b, first);
            let third = sine(b, "ar", 3.0);
            let sum = binary(b, "+", pair, third);
            out(b, "ar", sum);
        }),
        def("optimize_sum3_right", vec![], |b| {
            let sounds = sines(b, &[1.0, 2.0, 3.0]);
            let inner = binary(b, "+", sounds[1].clone(), sounds[2].clone());
            let sum = binary(b, "+", sounds[0].clone(), inner);
            out(b, "ar", sum);
        }),
        def("optimize_sum4", vec![], |b| {
            let sum = sum_of_sines(b, 4);
            out(b, "ar", sum);
        }),
        def("optimize_sum5", vec![], |b| {
            let sum = sum_of_sines(b, 5);
            out(b, "ar", sum);
        }),
        def("optimize_twice", vec![], |b| {
            let sounds = sines(b, &[1.0, 2.0]);
            let pair = plus_all(b, sounds);
            let sum = binary(b, "+", pair.clone(), pair);
            out(b, "ar", sum);
        }),
        def("optimize_addneg", vec![], |b| {
            let sounds = sines(b, &[1.0, 2.0]);
            let negated = neg(b, sounds[1].clone());
            let sum = binary(b, "+", sounds[0].clone(), negated);
            out(b, "ar", sum);
        }),
        def("optimize_negadd", vec![], |b| {
            let first = sine(b, "ar", 1.0);
            let negated = neg(b, first);
            let second = sine(b, "ar", 2.0);
            let sum = binary(b, "+", negated, second);
            out(b, "ar", sum);
        }),
        def("optimize_subneg", vec![], |b| {
            let sounds = sines(b, &[1.0, 2.0]);
            let pair = plus_all(b, sounds);
            let third = sine(b, "ar", 3.0);
            let negated = neg(b, third);
            let difference = binary(b, "-", pair, negated);
            out(b, "ar", difference);
        }),
        def("optimize_demand", vec![], |b| {
            let first = dseq(b, &[1.0, 2.0]);
            let second = dseq(b, &[3.0, 4.0]);
            let pair = binary(b, "+", first, second);
            let third = dseq(b, &[5.0]);
            let sum = binary(b, "+", pair, third);
            let trigger = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
            let demand = by_name(
                b,
                "Demand",
                "kr",
                &[
                    ("trig", input(trigger)),
                    ("reset", input(0.0)),
                    ("demandUGens", input(sum)),
                ],
            );
            out(b, "kr", demand);
        }),
    ]
}

/// `SinOsc.ar(1) + SinOsc.ar(2) + ...`, each `SinOsc` built before its `+`.
fn sum_of_sines(b: &mut SynthDefBuilder, count: u32) -> UGenInput {
    let first = sine(b, "ar", 1.0);
    (2..=count).fold(first, |sum, freq| {
        let sound = sine(b, "ar", f64::from(freq));
        binary(b, "+", sum, sound)
    })
}

fn dseq(b: &mut SynthDefBuilder, list: &[f64]) -> UGenInput {
    by_name(
        b,
        "Dseq",
        "new",
        &[
            ("list", input(array(list))),
            ("repeats", input(f64::INFINITY)),
        ],
    )
}

fn deadcode() -> Vec<SynthDef> {
    vec![
        def("deadcode_chain", vec![], |b| {
            let slow = sine(b, "ar", 1.0);
            let freq = binary(b, "*", slow, 100.0.into());
            sine(b, "ar", freq);
            let sound = sine(b, "ar", 440.0);
            out(b, "ar", sound);
        }),
        def("deadcode_proxy", vec![], |b| {
            let dc = by_name(b, "DC", "ar", &[("in", input(1.0))]);
            binary(b, "*", dc, 2.0.into());
            let sound = sine(b, "ar", 440.0);
            out(b, "ar", sound);
        }),
        def("deadcode_detectsilence", vec![], |b| {
            let quiet = sine(b, "ar", 2.0);
            by_name(b, "DetectSilence", "ar", &[("in", input(quiet))]);
            let sound = sine(b, "ar", 440.0);
            out(b, "ar", sound);
        }),
        def("deadcode_arg", vec![Arg::new("freq", 440.0)], |b| {
            let freq = arg(b, "freq");
            binary(b, "*", freq, 2.0.into());
            let sound = sine(b, "ar", 440.0);
            out(b, "ar", sound);
        }),
        def("deadcode_constant", vec![], |b| {
            sine(b, "ar", 1234.0);
            let sound = sine(b, "ar", 440.0);
            out(b, "ar", sound);
        }),
        def("deadcode_revisit", vec![], |b| {
            let (product, sum) = revisit(b);
            binary(b, "*", product, sum.clone());
            out(b, "ar", sum);
        }),
        def("deadcode_revisit_swapped", vec![], |b| {
            let (product, sum) = revisit(b);
            binary(b, "*", sum.clone(), product);
            out(b, "ar", sum);
        }),
        def("deadcode_readers", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let product = binary(b, "*", sound.clone(), 2.0.into());
            let sum = binary(b, "+", product, 1.0.into());
            for factor in [3.0, 4.0] {
                binary(b, "*", sound.clone(), factor.into());
            }
            out(b, "ar", sum);
        }),
    ]
}

/// `m = SinOsc.ar(1) * 2` and `a = m + 1`.
fn revisit(b: &mut SynthDefBuilder) -> (UGenInput, UGenInput) {
    let sound = sine(b, "ar", 1.0);
    let product = binary(b, "*", sound, 2.0.into());
    let sum = binary(b, "+", product.clone(), 1.0.into());
    (product, sum)
}
