//! The graphs of the golden files of the pseudo-UGens, built with `ugen`.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, ArgValue, SynthDef, SynthDefBuilder, UGenInput};

use super::{arg, array, by_name};

/// The definitions of a golden file of the pseudo-UGens.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "pmosc" => pmosc(),
        "changed" => changed(),
        "silent" => silent(),
        "dstutter" => dstutter(),
        "tchoose" => tchoose(),
        "twchoose" => twchoose(),
        "tap" => tap(),
        "linlin" => linlin(),
        "soundin" => soundin(),
        "selectx" => selectx(),
        "blowpass4" => blowpass4(),
        "bhipass4" => bhipass4(),
        "sum" => sum(),
        "mix" => mix(),
        _ => panic!("no golden graph is named {name}"),
    }
}

/// A definition with these arguments, which `build` fills.
fn def(name: &str, args: Vec<Arg>, build: impl FnOnce(&mut SynthDefBuilder)) -> SynthDef {
    let mut builder = SynthDefBuilder::new(name, args).unwrap();
    build(&mut builder);
    builder.finish().unwrap()
}

/// `SinOsc` at the rate of `selector`, of `freq`.
fn sine(builder: &mut SynthDefBuilder, selector: &str, freq: impl Into<UGenInput>) -> UGenInput {
    by_name(builder, "SinOsc", selector, &[("freq", freq.into().into())])
}

/// `SinOsc`s of these frequencies, as an array.
fn sines(builder: &mut SynthDefBuilder, selector: &str, freqs: &[f64]) -> UGenInput {
    UGenInput::Array(
        freqs
            .iter()
            .map(|&freq| sine(builder, selector, freq))
            .collect(),
    )
}

/// `Out.ar(0, channels)` or `Out.kr(0, channels)`.
fn out(builder: &mut SynthDefBuilder, selector: &str, channels: UGenInput) {
    by_name(
        builder,
        "Out",
        selector,
        &[("bus", 0.0.into()), ("channelsArray", channels.into())],
    );
}

fn input(value: impl Into<UGenInput>) -> ArgValue {
    ArgValue::Input(value.into())
}

fn pmosc() -> Vec<SynthDef> {
    vec![
        def("pmosc_ar", vec![], |b| {
            let sound = by_name(
                b,
                "PMOsc",
                "ar",
                &[
                    ("carfreq", input(440.0)),
                    ("modfreq", input(220.0)),
                    ("pmindex", input(2.0)),
                ],
            );
            out(b, "ar", sound);
        }),
        def("pmosc_kr", vec![Arg::new("car", 10.0)], |b| {
            let car = arg(b, "car");
            let sound = by_name(
                b,
                "PMOsc",
                "kr",
                &[
                    ("carfreq", input(car)),
                    ("modfreq", input(3.0)),
                    ("pmindex", input(1.5)),
                    ("modphase", input(0.25)),
                    ("mul", input(0.5)),
                    ("add", input(0.5)),
                ],
            );
            out(b, "kr", sound);
        }),
    ]
}

fn changed() -> Vec<SynthDef> {
    vec![
        def("changed_kr", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            let change = by_name(b, "Changed", "kr", &[("input", input(sound))]);
            out(b, "kr", change);
        }),
        def("changed_ar", vec![], |b| {
            let sound = sine(b, "ar", 2.0);
            let change = by_name(
                b,
                "Changed",
                "ar",
                &[("input", input(sound)), ("threshold", input(0.1))],
            );
            out(b, "ar", change);
        }),
        def("changed_array", vec![], |b| {
            let sounds = sine(b, "kr", array(&[1.0, 2.0]));
            let change = by_name(b, "Changed", "kr", &[("input", input(sounds))]);
            out(b, "kr", change);
        }),
    ]
}

fn silent() -> Vec<SynthDef> {
    vec![
        def("silent_1", vec![], |b| {
            let silence = by_name(b, "Silent", "ar", &[]);
            out(b, "ar", silence);
        }),
        def("silent_2", vec![], |b| {
            let silence = by_name(b, "Silent", "ar", &[("numChannels", input(2.0))]);
            out(b, "ar", silence);
        }),
    ]
}

fn dstutter() -> Vec<SynthDef> {
    vec![def("dstutter", vec![], |b| {
        let trig = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
        let seq = by_name(
            b,
            "Dseq",
            "new",
            &[
                ("list", input(array(&[1.0, 2.0]))),
                ("repeats", input(f64::INFINITY)),
            ],
        );
        let dup = by_name(
            b,
            "Dstutter",
            "new",
            &[("n", input(2.0)), ("in", input(seq))],
        );
        let demand = by_name(
            b,
            "Demand",
            "kr",
            &[
                ("trig", input(trig)),
                ("reset", input(0.0)),
                ("demandUGens", input(dup)),
            ],
        );
        out(b, "kr", demand);
    })]
}

fn tchoose() -> Vec<SynthDef> {
    vec![
        def("tchoose_kr", vec![], |b| {
            let trig = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
            let choices = sines(b, "kr", &[1.0, 2.0]);
            let chosen = by_name(
                b,
                "TChoose",
                "kr",
                &[("trig", input(trig)), ("array", input(choices))],
            );
            out(b, "kr", chosen);
        }),
        def("tchoose_ar", vec![], |b| {
            let trig = by_name(b, "Impulse", "ar", &[("freq", input(1.0))]);
            let choices = sines(b, "ar", &[1.0, 2.0, 3.0]);
            let chosen = by_name(
                b,
                "TChoose",
                "ar",
                &[("trig", input(trig)), ("array", input(choices))],
            );
            out(b, "ar", chosen);
        }),
    ]
}

fn twchoose() -> Vec<SynthDef> {
    vec![
        def("twchoose_kr", vec![], |b| {
            let trig = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
            let choices = sines(b, "kr", &[1.0, 2.0]);
            let chosen = by_name(
                b,
                "TWChoose",
                "kr",
                &[
                    ("trig", input(trig)),
                    ("array", input(choices)),
                    ("weights", input(array(&[0.3, 0.7]))),
                ],
            );
            out(b, "kr", chosen);
        }),
        def("twchoose_ar", vec![], |b| {
            let trig = by_name(b, "Impulse", "ar", &[("freq", input(1.0))]);
            let choices = sines(b, "ar", &[1.0, 2.0]);
            let chosen = by_name(
                b,
                "TWChoose",
                "ar",
                &[
                    ("trig", input(trig)),
                    ("array", input(choices)),
                    ("weights", input(array(&[1.0, 3.0]))),
                    ("normalize", input(1.0)),
                ],
            );
            out(b, "ar", chosen);
        }),
    ]
}

fn tap() -> Vec<SynthDef> {
    vec![
        def("tap_1", vec![], |b| {
            let tapped = by_name(
                b,
                "Tap",
                "ar",
                &[
                    ("bufnum", input(0.0)),
                    ("numChannels", input(1.0)),
                    ("delaytime", input(0.2)),
                ],
            );
            out(b, "ar", tapped);
        }),
        def("tap_2", vec![Arg::new("time", 0.5)], |b| {
            let time = arg(b, "time");
            let tapped = by_name(
                b,
                "Tap",
                "ar",
                &[
                    ("bufnum", input(1.0)),
                    ("numChannels", input(2.0)),
                    ("delaytime", input(time)),
                ],
            );
            out(b, "ar", tapped);
        }),
    ]
}

/// `LinLin` of `in` with the ranges, at the rate of `selector`.
fn lin_lin(b: &mut SynthDefBuilder, selector: &str, ranges: [UGenInput; 5]) -> UGenInput {
    let [input_value, srclo, srchi, dstlo, dsthi] = ranges;
    by_name(
        b,
        "LinLin",
        selector,
        &[
            ("in", input(input_value)),
            ("srclo", input(srclo)),
            ("srchi", input(srchi)),
            ("dstlo", input(dstlo)),
            ("dsthi", input(dsthi)),
        ],
    )
}

fn linlin() -> Vec<SynthDef> {
    vec![
        def("linlin_ar", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let mapped = lin_lin(
                b,
                "ar",
                [sound, (-1.0).into(), 1.0.into(), 100.0.into(), 200.0.into()],
            );
            out(b, "ar", mapped);
        }),
        def("linlin_kr", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            let mapped = lin_lin(
                b,
                "kr",
                [sound, (-1.0).into(), 1.0.into(), 100.0.into(), 200.0.into()],
            );
            out(b, "kr", mapped);
        }),
        def("linlin_numbers", vec![], |b| {
            let first = lin_lin(
                b,
                "kr",
                [0.5.into(), 0.0.into(), 1.0.into(), 10.0.into(), 20.0.into()],
            );
            let second = lin_lin(
                b,
                "ar",
                [0.25.into(), 0.0.into(), 1.0.into(), 0.0.into(), 4.0.into()],
            );
            assert_eq!([&first, &second], [&15.0.into(), &1.0.into()]);
            let sound = sine(b, "kr", 1.0);
            let channels = b
                .binary_op("*", UGenInput::Array(vec![first, second]), sound)
                .unwrap();
            out(b, "kr", channels);
        }),
        def("linlin_ugens", vec![Arg::new("lo", 100.0)], |b| {
            let lo = arg(b, "lo");
            let sound = sine(b, "kr", 1.0);
            let mapped = lin_lin(
                b,
                "kr",
                [sound, (-1.0).into(), 1.0.into(), lo, 200.0.into()],
            );
            out(b, "kr", mapped);
        }),
    ]
}

fn sound_in(b: &mut SynthDefBuilder, bus: UGenInput, mul: f64) -> UGenInput {
    by_name(
        b,
        "SoundIn",
        "ar",
        &[("bus", input(bus)), ("mul", input(mul))],
    )
}

fn soundin() -> Vec<SynthDef> {
    vec![
        def("soundin_0", vec![], |b| {
            let sound = sound_in(b, 0.0.into(), 1.0);
            out(b, "ar", sound);
        }),
        def("soundin_01", vec![], |b| {
            let sound = sound_in(b, array(&[0.0, 1.0]), 1.0);
            out(b, "ar", sound);
        }),
        def("soundin_02", vec![], |b| {
            let sound = sound_in(b, array(&[0.0, 2.0]), 1.0);
            out(b, "ar", sound);
        }),
        def("soundin_mul", vec![], |b| {
            let sound = sound_in(b, 2.0.into(), 0.5);
            out(b, "ar", sound);
        }),
        def("soundin_ugen", vec![Arg::new("bus", 1.0)], |b| {
            let bus = arg(b, "bus");
            let sound = sound_in(b, bus, 1.0);
            out(b, "ar", sound);
        }),
    ]
}

fn selectx() -> Vec<SynthDef> {
    vec![
        def("selectx_ar", vec![Arg::new("which", 0.0)], |b| {
            let which = arg(b, "which");
            let choices = sines(b, "ar", &[1.0, 2.0, 3.0]);
            let mixed = by_name(
                b,
                "SelectX",
                "ar",
                &[("which", input(which)), ("array", input(choices))],
            );
            out(b, "ar", mixed);
        }),
        def("selectx_kr", vec![], |b| {
            let choices = sines(b, "kr", &[1.0, 2.0]);
            let mixed = by_name(
                b,
                "SelectX",
                "kr",
                &[("which", input(0.5)), ("array", input(choices))],
            );
            out(b, "kr", mixed);
        }),
    ]
}

/// `class.ar(SinOsc.ar(1), freq, rq, mul)`, for `BLowPass4` and `BHiPass4`.
fn b_pass4(
    b: &mut SynthDefBuilder,
    class: &str,
    freq: UGenInput,
    rq: UGenInput,
    mul: f64,
) -> UGenInput {
    let sound = sine(b, "ar", 1.0);
    by_name(
        b,
        class,
        "ar",
        &[
            ("in", input(sound)),
            ("freq", input(freq)),
            ("rq", input(rq)),
            ("mul", input(mul)),
        ],
    )
}

fn blowpass4() -> Vec<SynthDef> {
    vec![
        def("blowpass4", vec![], |b| {
            let filtered = b_pass4(b, "BLowPass4", 800.0.into(), 0.5.into(), 1.0);
            out(b, "ar", filtered);
        }),
        def(
            "blowpass4_ugens",
            vec![Arg::new("freq", 800.0), Arg::new("rq", 0.5)],
            |b| {
                let (freq, rq) = (arg(b, "freq"), arg(b, "rq"));
                let filtered = b_pass4(b, "BLowPass4", freq, rq, 0.5);
                out(b, "ar", filtered);
            },
        ),
    ]
}

fn bhipass4() -> Vec<SynthDef> {
    vec![def("bhipass4", vec![], |b| {
        let filtered = b_pass4(b, "BHiPass4", 800.0.into(), 0.5.into(), 1.0);
        out(b, "ar", filtered);
    })]
}

fn sum() -> Vec<SynthDef> {
    vec![
        def("sum3_fold", vec![], |b| {
            let (first, second) = (sine(b, "ar", 1.0), sine(b, "ar", 2.0));
            let total = by_name(
                b,
                "Sum3",
                "new",
                &[
                    ("in0", input(first)),
                    ("in1", input(second)),
                    ("in2", input(0.0)),
                ],
            );
            out(b, "ar", total);
        }),
        def("sum4_fold", vec![], |b| {
            let first = sine(b, "ar", 1.0);
            let third = sine(b, "kr", 2.0);
            let fourth = sine(b, "ar", 3.0);
            let total = by_name(
                b,
                "Sum4",
                "new",
                &[
                    ("in0", input(first)),
                    ("in1", input(0.0)),
                    ("in2", input(third)),
                    ("in3", input(fourth)),
                ],
            );
            out(b, "ar", total);
        }),
        def("sum3_array", vec![], |b| {
            let pair = sines(b, "ar", &[1.0, 2.0]);
            let (third, fourth) = (sine(b, "ar", 3.0), sine(b, "ar", 4.0));
            let total = by_name(
                b,
                "Sum3",
                "new",
                &[
                    ("in0", input(pair)),
                    ("in1", input(third)),
                    ("in2", input(fourth)),
                ],
            );
            out(b, "ar", total);
        }),
    ]
}

/// `Mix` at the rate of `selector`, of `array`.
fn mix_of(b: &mut SynthDefBuilder, selector: &str, array: UGenInput) -> UGenInput {
    by_name(b, "Mix", selector, &[("array", input(array))])
}

/// `Out.ar(0, Mix(SinOsc.ar(freqs)))`.
fn mixed_sines(name: &str, freqs: &[f64]) -> SynthDef {
    def(name, vec![], |b| {
        let sounds = sine(b, "ar", array(freqs));
        let mixed = mix_of(b, "new", sounds);
        out(b, "ar", mixed);
    })
}

fn mix() -> Vec<SynthDef> {
    let nine: Vec<f64> = (1..=9).map(f64::from).collect();
    vec![
        def("mix_1", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let mixed = mix_of(b, "new", sound);
            out(b, "ar", mixed);
        }),
        mixed_sines("mix_2", &[1.0, 2.0]),
        mixed_sines("mix_3", &[1.0, 2.0, 3.0]),
        mixed_sines("mix_4", &[1.0, 2.0, 3.0, 4.0]),
        mixed_sines("mix_5", &[1.0, 2.0, 3.0, 4.0, 5.0]),
        mixed_sines("mix_9", &nine),
        def("mix_rates3", vec![], |b| {
            let sounds = vec![sine(b, "kr", 1.0), sine(b, "ar", 2.0), sine(b, "kr", 3.0)];
            let mixed = mix_of(b, "new", sounds.into());
            out(b, "ar", mixed);
        }),
        def("mix_rates4", vec![], |b| {
            let sounds = vec![
                sine(b, "kr", 1.0),
                sine(b, "ar", 2.0),
                sine(b, "kr", 3.0),
                sine(b, "ar", 4.0),
            ];
            let mixed = mix_of(b, "new", sounds.into());
            out(b, "ar", mixed);
        }),
        def("mix_numbers", vec![], |b| {
            let pair = mix_of(b, "new", array(&[1.0, 2.0]));
            assert_eq!(pair, 3.0.into());
            let triple = mix_of(b, "new", array(&[1.0, 2.0, 3.0]));
            let sound = sine(b, "kr", 1.0);
            let channels = b
                .binary_op("*", UGenInput::Array(vec![pair, triple]), sound)
                .unwrap();
            out(b, "kr", channels);
        }),
        def("mix_nested", vec![], |b| {
            let pairs = vec![
                sine(b, "ar", array(&[1.0, 2.0])),
                sine(b, "ar", array(&[3.0, 4.0])),
                sine(b, "ar", array(&[5.0, 6.0])),
            ];
            let mixed = mix_of(b, "new", pairs.into());
            out(b, "ar", mixed);
        }),
        def("mix_ar", vec![], |b| {
            let sounds = sines(b, "kr", &[1.0, 2.0]);
            let mixed = mix_of(b, "ar", sounds);
            out(b, "ar", mixed);
        }),
        def("mix_ar_number", vec![], |b| {
            let mixed = mix_of(b, "ar", array(&[1.0, 2.0]));
            out(b, "ar", mixed);
        }),
        def("mix_kr", vec![], |b| {
            let sounds = vec![sine(b, "ar", 1.0), sine(b, "kr", 2.0)];
            let mixed = mix_of(b, "kr", sounds.into());
            out(b, "kr", mixed);
        }),
        def("mix_kr_number", vec![], |b| {
            let mixed = mix_of(b, "kr", array(&[1.0, 2.0]));
            out(b, "kr", mixed);
        }),
    ]
}
