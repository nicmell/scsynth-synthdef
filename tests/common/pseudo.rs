//! The graphs of the golden files of the pseudo-UGens, built with `ugen`.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, ArgValue, SynthDef, SynthDefBuilder, UGenInput};

use super::{arg, array, by_name, element, flat};

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
        "numchannels" => numchannels(),
        "selectxfocus" => selectxfocus(),
        "multitap" => multitap(),
        "splay" => splay(),
        "dynklank" => dynklank(),
        "dynklang" => dynklang(),
        "unpackfft" => unpackfft(),
        "delays" => delays(),
        "linxfade2" => linxfade2(),
        "companderd" => companderd(),
        "hilbertfir" => hilbertfir(),
        "linselectx" => linselectx(),
        "splayaz" => splayaz(),
        "lag" => [lag(), lag_folds()].concat(),
        "outzeros" => outzeros(),
        "localbuf" => localbuf(),
        "beattrack" => beattrack(),
        "localin" => localin(),
        "poll" => poll(),
        "symbols" => symbols(),
        "bufwr" => bufwr(),
        "duty" => duty(),
        "peak" => peak(),
        "t2k" => t2k(),
        "klank" => klank(),
        "klang" => klang(),
        "pingpong" => pingpong(),
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

/// `NumChannels.ar(input, numChannels, mixdown)`.
fn num_channels(
    b: &mut SynthDefBuilder,
    input_value: UGenInput,
    channels: f64,
    mixdown: bool,
) -> UGenInput {
    by_name(
        b,
        "NumChannels",
        "ar",
        &[
            ("input", input(input_value)),
            ("numChannels", input(channels)),
            ("mixdown", ArgValue::Bool(mixdown)),
        ],
    )
}

fn numchannels() -> Vec<SynthDef> {
    vec![
        def("numchannels_mix", vec![], |b| {
            let sounds = sine(b, "ar", array(&[1.0, 2.0, 3.0, 4.0]));
            let channels = num_channels(b, sounds, 2.0, true);
            out(b, "ar", channels);
        }),
        def("numchannels_cut", vec![], |b| {
            let sounds = sine(b, "ar", array(&[1.0, 2.0, 3.0]));
            let channels = num_channels(b, sounds, 2.0, false);
            out(b, "ar", channels);
        }),
        def("numchannels_dup", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let channels = num_channels(b, sound, 3.0, true);
            out(b, "ar", channels);
        }),
        def("numchannels_one", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let channels = num_channels(b, vec![sound].into(), 1.0, true);
            out(b, "ar", channels);
        }),
    ]
}

/// `SelectXFocus` at the rate of `selector`.
fn select_x_focus(
    b: &mut SynthDefBuilder,
    selector: &str,
    which: UGenInput,
    choices: UGenInput,
    focus: f64,
    wrap: bool,
) -> UGenInput {
    by_name(
        b,
        "SelectXFocus",
        selector,
        &[
            ("which", input(which)),
            ("array", input(choices)),
            ("focus", input(focus)),
            ("wrap", ArgValue::Bool(wrap)),
        ],
    )
}

fn selectxfocus() -> Vec<SynthDef> {
    vec![
        def("selectxfocus", vec![Arg::new("which", 0.0)], |b| {
            let which = arg(b, "which");
            let choices = sines(b, "ar", &[1.0, 2.0, 3.0]);
            let mixed = select_x_focus(b, "ar", which, choices, 1.0, false);
            out(b, "ar", mixed);
        }),
        def("selectxfocus_wrap", vec![Arg::new("which", 0.0)], |b| {
            let which = arg(b, "which");
            let choices = sines(b, "ar", &[1.0, 2.0, 3.0]);
            let mixed = select_x_focus(b, "ar", which, choices, 0.5, true);
            out(b, "ar", mixed);
        }),
        def("selectxfocus_number", vec![], |b| {
            let choices = sines(b, "kr", &[1.0, 2.0, 3.0]);
            let mixed = select_x_focus(b, "kr", 1.25.into(), choices, 1.0, false);
            out(b, "kr", mixed);
        }),
    ]
}

/// `MultiTap.ar(times, levels, SinOsc.ar(1), mul, 0, bufnum)`.
fn multi_tap(
    b: &mut SynthDefBuilder,
    times: &[f64],
    levels: &[f64],
    mul: f64,
    bufnum: f64,
) -> UGenInput {
    let sound = sine(b, "ar", 1.0);
    by_name(
        b,
        "MultiTap",
        "ar",
        &[
            ("timesArray", input(array(times))),
            ("levelsArray", input(array(levels))),
            ("in", input(sound)),
            ("mul", input(mul)),
            ("bufnum", input(bufnum)),
        ],
    )
}

fn multitap() -> Vec<SynthDef> {
    vec![
        def("multitap", vec![], |b| {
            let taps = multi_tap(b, &[0.1, 0.2], &[0.5, 0.3], 1.0, 0.0);
            out(b, "ar", taps);
        }),
        def("multitap_levels", vec![], |b| {
            let taps = multi_tap(b, &[0.1, 0.2, 0.3], &[0.5], 0.5, 1.0);
            out(b, "ar", taps);
        }),
    ]
}

/// `Splay` at the rate of `selector`, of `inputs`, with these arguments.
fn splay_of(
    b: &mut SynthDefBuilder,
    selector: &str,
    inputs: UGenInput,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![("inArray", input(inputs))];
    all.extend(args.iter().cloned());
    by_name(b, "Splay", selector, &all)
}

/// `Out.ar(0, Splay.ar(SinOsc.ar(freqs), args))`.
fn splayed_sines(name: &str, freqs: &[f64], args: &[(&str, ArgValue)]) -> SynthDef {
    def(name, vec![], |b| {
        let sounds = sine(b, "ar", array(freqs));
        let spread = splay_of(b, "ar", sounds, args);
        out(b, "ar", spread);
    })
}

fn splay() -> Vec<SynthDef> {
    vec![
        splayed_sines("splay_2", &[1.0, 2.0], &[]),
        splayed_sines("splay_3", &[1.0, 2.0, 3.0], &[]),
        def("splay_mono", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let spread = splay_of(b, "ar", sound, &[]);
            out(b, "ar", spread);
        }),
        def("splay_spread", vec![Arg::new("spread", 1.0)], |b| {
            let spread = arg(b, "spread");
            let sounds = sine(b, "ar", array(&[1.0, 2.0, 3.0]));
            let spread_out = splay_of(b, "ar", sounds, &[("spread", input(spread))]);
            out(b, "ar", spread_out);
        }),
        def("splay_nested", vec![], |b| {
            let pairs = vec![
                sine(b, "ar", array(&[1.0, 2.0])),
                sine(b, "ar", array(&[3.0, 4.0])),
            ];
            let spread = splay_of(b, "ar", pairs.into(), &[]);
            out(b, "ar", UGenInput::Array(flat(spread)));
        }),
        def("splay_kr", vec![], |b| {
            let sounds = sine(b, "kr", array(&[1.0, 2.0, 3.0]));
            let spread = splay_of(b, "kr", sounds, &[]);
            out(b, "kr", spread);
        }),
        def("splay_level", vec![Arg::new("level", 0.5)], |b| {
            let level = arg(b, "level");
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let spread = splay_of(
                b,
                "ar",
                sounds,
                &[("spread", input(1.0)), ("level", input(level))],
            );
            out(b, "ar", spread);
        }),
        splayed_sines(
            "splay_center",
            &[1.0, 2.0],
            &[
                ("spread", input(0.5)),
                ("level", input(1.0)),
                ("center", input(0.25)),
            ],
        ),
        splayed_sines(
            "splay_levelcomp_false",
            &[1.0, 2.0],
            &[("levelComp", ArgValue::Bool(false))],
        ),
        splayed_sines(
            "splay_levelcomp_number",
            &[1.0, 2.0],
            &[("levelComp", input(0.5))],
        ),
        def("splay_levelcomp_ugen", vec![Arg::new("lc", 0.5)], |b| {
            let comp = arg(b, "lc");
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let spread = splay_of(b, "ar", sounds, &[("levelComp", input(comp))]);
            out(b, "ar", spread);
        }),
    ]
}

/// A spec of `DynKlank` or `DynKlang`: arrays of numbers.
fn spec(parts: &[&[f64]]) -> UGenInput {
    UGenInput::Array(parts.iter().map(|part| array(part)).collect())
}

/// `DynKlank` at the rate of `selector`, rung by `Impulse` of `freq`.
fn dyn_klank(
    b: &mut SynthDefBuilder,
    selector: &str,
    parts: &[&[f64]],
    freq: UGenInput,
    scale: Option<UGenInput>,
) -> UGenInput {
    let trigger = by_name(b, "Impulse", selector, &[("freq", input(freq))]);
    let mut args = vec![
        ("specificationsArrayRef", input(spec(parts))),
        ("input", input(trigger)),
    ];
    if let Some(scale) = scale {
        args.push(("freqscale", input(scale)));
    }
    by_name(b, "DynKlank", selector, &args)
}

fn dynklank() -> Vec<SynthDef> {
    vec![
        def("dynklank_2", vec![], |b| {
            let rings = dyn_klank(b, "ar", &[&[800.0, 1071.0]], 2.0.into(), None);
            out(b, "ar", rings);
        }),
        def("dynklank_3", vec![], |b| {
            let parts: &[&[f64]] = &[&[800.0, 1071.0, 1353.0], &[1.0], &[1.0, 1.0, 1.0]];
            let rings = dyn_klank(b, "ar", parts, 2.0.into(), None);
            out(b, "ar", rings);
        }),
        def("dynklank_amps", vec![], |b| {
            let rings = dyn_klank(b, "ar", &[&[800.0, 1071.0], &[0.5, 0.3]], 2.0.into(), None);
            out(b, "ar", rings);
        }),
        def("dynklank_scale", vec![Arg::new("scale", 1.0)], |b| {
            let scale = arg(b, "scale");
            let parts: &[&[f64]] = &[&[800.0, 1071.0], &[1.0], &[1.0, 2.0]];
            let rings = dyn_klank(b, "ar", parts, 2.0.into(), Some(scale));
            out(b, "ar", rings);
        }),
        def("dynklank_kr", vec![], |b| {
            let rings = dyn_klank(b, "kr", &[&[8.0, 10.0]], 2.0.into(), None);
            out(b, "kr", rings);
        }),
        def("dynklank_expand", vec![], |b| {
            let rings = dyn_klank(b, "ar", &[&[800.0, 1071.0]], array(&[2.0, 3.0]), None);
            out(b, "ar", rings);
        }),
    ]
}

/// `DynKlang` at the rate of `selector`.
fn dyn_klang(
    b: &mut SynthDefBuilder,
    selector: &str,
    parts: &[&[f64]],
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![("specificationsArrayRef", input(spec(parts)))];
    all.extend(args.iter().cloned());
    by_name(b, "DynKlang", selector, &all)
}

fn dynklang() -> Vec<SynthDef> {
    vec![
        def("dynklang_2", vec![], |b| {
            let sines = dyn_klang(b, "ar", &[&[800.0, 1000.0]], &[]);
            out(b, "ar", sines);
        }),
        def("dynklang_amps", vec![], |b| {
            let pi = std::f64::consts::PI;
            let parts: &[&[f64]] = &[&[800.0, 1000.0], &[0.3, 0.3], &[pi, pi]];
            let sines = dyn_klang(b, "ar", parts, &[]);
            out(b, "ar", sines);
        }),
        def("dynklang_kr", vec![], |b| {
            let parts: &[&[f64]] = &[&[1.0, 2.0], &[1.0], &[0.5, 0.25]];
            let sines = dyn_klang(
                b,
                "kr",
                parts,
                &[("freqscale", input(2.0)), ("freqoffset", input(1.0))],
            );
            out(b, "kr", sines);
        }),
    ]
}

/// `Demand.kr(Impulse.kr(1), 0, UnpackFFT(FFT(0, SinOsc.ar(1)), bufsize, args))`.
fn unpacked(name: &str, bufsize: f64, args: &[(&str, ArgValue)]) -> SynthDef {
    def(name, vec![], |b| {
        let sound = sine(b, "ar", 1.0);
        let chain = by_name(
            b,
            "FFT",
            "new",
            &[("buffer", input(0.0)), ("in", input(sound))],
        );
        let trig = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
        let mut all = vec![("chain", input(chain)), ("bufsize", input(bufsize))];
        all.extend(args.iter().cloned());
        let bins = by_name(b, "UnpackFFT", "new", &all);
        let demand = by_name(
            b,
            "Demand",
            "kr",
            &[
                ("trig", input(trig)),
                ("reset", input(0.0)),
                ("demandUGens", input(bins)),
            ],
        );
        out(b, "kr", demand);
    })
}

fn unpackfft() -> Vec<SynthDef> {
    vec![
        unpacked(
            "unpackfft",
            16.0,
            &[("frombin", input(0.0)), ("tobin", input(2.0))],
        ),
        unpacked("unpackfft_default", 4.0, &[]),
    ]
}

/// A delay of `class` at audio rate, with these arguments.
fn delay(b: &mut SynthDefBuilder, class: &str, args: &[(&str, ArgValue)]) -> UGenInput {
    by_name(b, class, "ar", args)
}

/// `class.ar(in, 0.2, 0.1, 1)`, for a comb or an allpass.
fn comb(b: &mut SynthDefBuilder, class: &str, sound: UGenInput) -> UGenInput {
    delay(
        b,
        class,
        &[
            ("in", input(sound)),
            ("maxdelaytime", input(0.2)),
            ("delaytime", input(0.1)),
            ("decaytime", input(1.0)),
        ],
    )
}

fn delays() -> Vec<SynthDef> {
    vec![
        def("delayn_kr_input", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            let delayed = delay(
                b,
                "DelayN",
                &[
                    ("in", input(sound)),
                    ("maxdelaytime", input(0.2)),
                    ("delaytime", input(0.1)),
                ],
            );
            out(b, "ar", delayed);
        }),
        def("combl_number", vec![], |b| {
            let delayed = comb(b, "CombL", 0.5.into());
            out(b, "ar", delayed);
        }),
        def("allpassc_zero", vec![], |b| {
            let delayed = comb(b, "AllpassC", 0.0.into());
            out(b, "ar", delayed);
        }),
        def("bufdelayc_audio", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let delayed = delay(
                b,
                "BufDelayC",
                &[
                    ("buf", input(0.0)),
                    ("in", input(sound)),
                    ("delaytime", input(0.1)),
                ],
            );
            out(b, "ar", delayed);
        }),
        def("bufcombn_array", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            let delayed = delay(
                b,
                "BufCombN",
                &[
                    ("buf", input(0.0)),
                    ("in", input(vec![sound, 0.25.into()])),
                    ("delaytime", input(0.1)),
                    ("decaytime", input(1.0)),
                ],
            );
            out(b, "ar", delayed);
        }),
        def("delayn_mul", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let delayed = delay(
                b,
                "DelayN",
                &[
                    ("in", input(sound)),
                    ("maxdelaytime", input(0.2)),
                    ("delaytime", input(0.1)),
                    ("mul", input(0.5)),
                ],
            );
            out(b, "ar", delayed);
        }),
        def("deltapwr", vec![], |b| {
            let sound = sine(b, "kr", 2.0);
            let tap = delay(
                b,
                "DelTapWr",
                &[("buffer", input(0.0)), ("in", input(sound))],
            );
            let read = by_name(
                b,
                "DelTapRd",
                "ar",
                &[
                    ("buffer", input(0.0)),
                    ("phase", input(tap)),
                    ("delTime", input(0.1)),
                ],
            );
            out(b, "ar", read);
        }),
    ]
}

/// `LinXFade2` of two `SinOsc`, at the rate of `selector`.
fn lin_x_fade2(
    b: &mut SynthDefBuilder,
    selector: &str,
    pan: f64,
    level: Option<UGenInput>,
) -> UGenInput {
    let (first, second) = (sine(b, selector, 1.0), sine(b, selector, 2.0));
    let mut args = vec![
        ("inA", input(first)),
        ("inB", input(second)),
        ("pan", input(pan)),
    ];
    if let Some(level) = level {
        args.push(("level", input(level)));
    }
    by_name(b, "LinXFade2", selector, &args)
}

fn linxfade2() -> Vec<SynthDef> {
    vec![
        def("linxfade2_level", vec![], |b| {
            let faded = lin_x_fade2(b, "ar", 0.5, Some(0.5.into()));
            out(b, "ar", faded);
        }),
        def("linxfade2_one", vec![], |b| {
            let faded = lin_x_fade2(b, "kr", -0.5, None);
            out(b, "kr", faded);
        }),
        def("linxfade2_ugen", vec![Arg::new("level", 0.5)], |b| {
            let level = arg(b, "level");
            let faded = lin_x_fade2(b, "ar", 0.0, Some(level));
            out(b, "ar", faded);
        }),
    ]
}

fn companderd() -> Vec<SynthDef> {
    vec![
        def("companderd", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let compressed = by_name(b, "CompanderD", "ar", &[("in", input(sound))]);
            out(b, "ar", compressed);
        }),
        def("companderd_number", vec![], |b| {
            let compressed = by_name(
                b,
                "CompanderD",
                "ar",
                &[
                    ("in", input(0.5)),
                    ("thresh", input(0.4)),
                    ("slopeBelow", input(1.0)),
                    ("slopeAbove", input(0.5)),
                    ("clampTime", input(0.02)),
                    ("relaxTime", input(0.1)),
                    ("mul", input(0.5)),
                ],
            );
            out(b, "ar", compressed);
        }),
    ]
}

fn hilbertfir() -> Vec<SynthDef> {
    vec![def("hilbertfir", vec![], |b| {
        let sound = sine(b, "ar", 1.0);
        let pair = by_name(
            b,
            "HilbertFIR",
            "ar",
            &[("in", input(sound)), ("buffer", input(0.0))],
        );
        out(b, "ar", pair);
    })]
}

fn linselectx() -> Vec<SynthDef> {
    vec![
        def("linselectx_ar", vec![Arg::new("which", 0.0)], |b| {
            let which = arg(b, "which");
            let choices = sines(b, "ar", &[1.0, 2.0, 3.0]);
            let mixed = by_name(
                b,
                "LinSelectX",
                "ar",
                &[("which", input(which)), ("array", input(choices))],
            );
            out(b, "ar", mixed);
        }),
        def("linselectx_kr", vec![], |b| {
            let choices = sines(b, "kr", &[1.0, 2.0]);
            let mixed = by_name(
                b,
                "LinSelectX",
                "kr",
                &[("which", input(0.5)), ("array", input(choices))],
            );
            out(b, "kr", mixed);
        }),
    ]
}

/// `SplayAz` at the rate of `selector`, with these arguments.
fn splay_az(
    b: &mut SynthDefBuilder,
    selector: &str,
    channels: f64,
    inputs: UGenInput,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![("numChans", input(channels)), ("inArray", input(inputs))];
    all.extend(args.iter().cloned());
    by_name(b, "SplayAz", selector, &all)
}

fn splayaz() -> Vec<SynthDef> {
    vec![
        def("splayaz_3", vec![], |b| {
            let sounds = sine(b, "ar", array(&[1.0, 2.0, 3.0]));
            let spread = splay_az(b, "ar", 4.0, sounds, &[]);
            out(b, "ar", spread);
        }),
        def("splayaz_mono", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let spread = splay_az(b, "ar", 2.0, sound, &[]);
            out(b, "ar", spread);
        }),
        def("splayaz_kr", vec![], |b| {
            let sounds = sine(b, "kr", array(&[1.0, 2.0]));
            let spread = splay_az(b, "kr", 3.0, sounds, &[]);
            out(b, "kr", spread);
        }),
        def("splayaz_spread", vec![], |b| {
            let sounds = sine(b, "ar", array(&[1.0, 2.0, 3.0]));
            let wobble = sine(b, "kr", 0.1);
            let spread = splay_az(b, "ar", 4.0, sounds, &[("spread", input(wobble))]);
            out(b, "ar", spread);
        }),
        def("splayaz_center", vec![Arg::new("c", 0.0)], |b| {
            let center = arg(b, "c");
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let spread = splay_az(b, "ar", 2.0, sounds, &[("center", input(center))]);
            out(b, "ar", spread);
        }),
        def("splayaz_audio_center", vec![], |b| {
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let center = sine(b, "ar", 0.1);
            let spread = splay_az(b, "ar", 2.0, sounds, &[("center", input(center))]);
            out(b, "ar", spread);
        }),
    ]
}

/// `class.selector(args)` into `Out` at the rate of `selector`.
fn lag_out(b: &mut SynthDefBuilder, class: &str, selector: &str, args: &[(&str, ArgValue)]) {
    let lagged = by_name(b, class, selector, args);
    out(b, selector, lagged);
}

/// A definition of `class.selector` on the argument `name`, at 440, with
/// these other arguments.
fn lag_of(
    def_name: &str,
    name: &str,
    class: &'static str,
    selector: &'static str,
    args: &'static [(&str, f64)],
) -> SynthDef {
    def(def_name, vec![Arg::new(name, 440.0)], |b| {
        let mut all = vec![("in", input(arg(b, name)))];
        all.extend(args.iter().map(|&(name, value)| (name, input(value))));
        lag_out(b, class, selector, &all);
    })
}

/// A definition of `class.selector` on a `SinOsc.ar(1)`, with these other
/// arguments.
fn lag_of_sine(def_name: &str, class: &'static str, args: &'static [(&str, f64)]) -> SynthDef {
    def(def_name, vec![], |b| {
        let sound = sine(b, "ar", 1.0);
        let mut all = vec![("in", input(sound))];
        all.extend(args.iter().map(|&(name, value)| (name, input(value))));
        lag_out(b, class, "ar", &all);
    })
}

fn lag() -> Vec<SynthDef> {
    vec![
        lag_of("lag_kr", "freq", "Lag", "kr", &[("lagTime", 0.1)]),
        lag_of_sine("lag_ar", "Lag", &[("lagTime", 0.01)]),
        lag_of("lag2_kr", "freq", "Lag2", "kr", &[("lagTime", 0.1)]),
        lag_of_sine("lag3_ar", "Lag3", &[("lagTime", 0.01)]),
        lag_of("ramp_kr", "freq", "Ramp", "kr", &[("lagTime", 0.1)]),
        lag_of(
            "lagud_kr",
            "freq",
            "LagUD",
            "kr",
            &[("lagTimeU", 0.1), ("lagTimeD", 0.2)],
        ),
        lag_of_sine(
            "lag2ud_ar",
            "Lag2UD",
            &[("lagTimeU", 0.01), ("lagTimeD", 0.02)],
        ),
        lag_of(
            "lag3ud_kr",
            "freq",
            "Lag3UD",
            "kr",
            &[("lagTimeU", 0.1), ("lagTimeD", 0.2)],
        ),
        lag_of_sine("slew_ar", "Slew", &[("up", 100.0), ("dn", 200.0)]),
        lag_of("slew_kr", "freq", "Slew", "kr", &[]),
    ]
}

/// The cases of `lag` that give back their input, or that do not although
/// a lag is 0.
fn lag_folds() -> Vec<SynthDef> {
    vec![
        def("lag_number", vec![], |b| {
            lag_out(
                b,
                "Lag",
                "kr",
                &[("in", input(440.0)), ("lagTime", input(0.1))],
            );
        }),
        lag_of("lag_zero", "freq", "Lag", "kr", &[("lagTime", 0.0)]),
        lag_of("lag_scalar", "i_freq", "Lag", "kr", &[("lagTime", 0.1)]),
        lag_of(
            "lagud_zero",
            "freq",
            "LagUD",
            "kr",
            &[("lagTimeU", 0.0), ("lagTimeD", 0.0)],
        ),
        lag_of("slew_scalar", "i_freq", "Slew", "kr", &[]),
        def("lag_madd_number", vec![], |b| {
            lag_out(
                b,
                "Lag",
                "kr",
                &[
                    ("in", input(2.0)),
                    ("lagTime", input(0.1)),
                    ("mul", input(3.0)),
                    ("add", input(4.0)),
                ],
            );
        }),
        def("lag_madd_scalar", vec![Arg::new("i_x", 1.0)], |b| {
            let x = arg(b, "i_x");
            lag_out(
                b,
                "Lag",
                "kr",
                &[
                    ("in", input(x)),
                    ("lagTime", input(0.1)),
                    ("mul", input(2.0)),
                    ("add", input(3.0)),
                ],
            );
        }),
        lag_of(
            "lag_madd",
            "freq",
            "Lag",
            "kr",
            &[("lagTime", 0.1), ("mul", 2.0), ("add", 3.0)],
        ),
        def(
            "lag_array",
            vec![Arg::new("a", 1.0), Arg::new("b", 2.0)],
            |b| {
                let pair = UGenInput::Array(vec![arg(b, "a"), arg(b, "b")]);
                lag_out(
                    b,
                    "Lag",
                    "kr",
                    &[("in", input(pair)), ("lagTime", input(0.1))],
                );
            },
        ),
        def("lag_array_scalar", vec![], |b| {
            lag_out(
                b,
                "Lag",
                "kr",
                &[
                    ("in", input(array(&[1.0, 2.0]))),
                    ("lagTime", input(0.1)),
                    ("mul", input(2.0)),
                ],
            );
        }),
    ]
}

/// `class.ar` with a bus of 0 and these other arguments.
fn out_ar(b: &mut SynthDefBuilder, class: &str, args: &[(&str, ArgValue)]) {
    let mut all = vec![("bus", input(0.0))];
    all.extend(args.iter().cloned());
    by_name(b, class, "ar", &all);
}

fn outzeros() -> Vec<SynthDef> {
    vec![
        def("outzeros_one", vec![], |b| out(b, "ar", 0.0.into())),
        def("outzeros_two", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            out(
                b,
                "ar",
                UGenInput::Array(vec![sound, 0.0.into(), 0.0.into()]),
            );
        }),
        def("outzeros_negative", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            out(b, "ar", UGenInput::Array(vec![(-0.0).into(), sound]));
        }),
        def("outzeros_replace", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let channels = UGenInput::Array(vec![0.0.into(), sound]);
            out_ar(b, "ReplaceOut", &[("channelsArray", input(channels))]);
        }),
        def("outzeros_offset", vec![], |b| {
            out_ar(b, "OffsetOut", &[("channelsArray", input(0.0))]);
        }),
        def("outzeros_x", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let channels = UGenInput::Array(vec![sound, 0.0.into()]);
            out_ar(
                b,
                "XOut",
                &[("xfade", input(0.5)), ("channelsArray", input(channels))],
            );
        }),
        def("outzeros_kr", vec![], |b| out(b, "kr", 0.0.into())),
    ]
}

/// `LocalBuf(frames, channels)`.
fn local_buf(b: &mut SynthDefBuilder, frames: impl Into<UGenInput>, channels: f64) -> UGenInput {
    by_name(
        b,
        "LocalBuf",
        "new",
        &[
            ("numFrames", input(frames)),
            ("numChannels", input(channels)),
        ],
    )
}

/// `FFT(buffer, sound)`.
fn fft(b: &mut SynthDefBuilder, buffer: UGenInput, sound: UGenInput) -> UGenInput {
    by_name(
        b,
        "FFT",
        "new",
        &[("buffer", input(buffer)), ("in", input(sound))],
    )
}

/// `IFFT(chain)`.
fn ifft(b: &mut SynthDefBuilder, chain: UGenInput) -> UGenInput {
    by_name(b, "IFFT", "new", &[("buffer", input(chain))])
}

fn localbuf() -> Vec<SynthDef> {
    vec![
        def("localbuf_fft", vec![], |b| {
            let buffer = local_buf(b, 2048.0, 1.0);
            let sound = sine(b, "ar", 1.0);
            let chain = fft(b, buffer, sound);
            let back = ifft(b, chain);
            out(b, "ar", back);
        }),
        def("localbuf_two", vec![], |b| {
            let buffer = local_buf(b, 1024.0, 1.0);
            let sound = sine(b, "ar", 1.0);
            let a = fft(b, buffer, sound);
            let buffer = local_buf(b, 2048.0, 1.0);
            let sound = sine(b, "ar", 2.0);
            let c = fft(b, buffer, sound);
            let backs = UGenInput::Array(vec![ifft(b, a), ifft(b, c)]);
            out(b, "ar", backs);
        }),
        def("localbuf_array", vec![], |b| {
            let buffers = local_buf(b, array(&[512.0, 1024.0]), 1.0);
            let sound = sine(b, "ar", 1.0);
            let chains = fft(b, buffers, sound);
            let backs = ifft(b, chains);
            out(b, "ar", backs);
        }),
        def("localbuf_channels", vec![], |b| {
            let buffer = local_buf(b, 1024.0, 2.0);
            let sound = by_name(
                b,
                "PlayBuf",
                "ar",
                &[("numChannels", input(2.0)), ("bufnum", input(buffer))],
            );
            out(b, "ar", sound);
        }),
    ]
}

fn beattrack() -> Vec<SynthDef> {
    vec![
        def("beattrack_auto", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let beats = by_name(b, "BeatTrack", "kr", &[("chain", input(sound))]);
            out(b, "kr", beats);
        }),
        def("beattrack_chain", vec![], |b| {
            let buffer = local_buf(b, 1024.0, 1.0);
            let sound = sine(b, "ar", 1.0);
            let chain = fft(b, buffer, sound);
            let beats = by_name(
                b,
                "BeatTrack",
                "kr",
                &[("chain", input(chain)), ("lock", input(0.5))],
            );
            out(b, "kr", beats);
        }),
    ]
}

/// `LocalIn.selector(channels, default)`.
fn local_in(
    b: &mut SynthDefBuilder,
    selector: &str,
    channels: f64,
    default: Option<UGenInput>,
) -> UGenInput {
    let mut args = vec![("numChannels", input(channels))];
    args.extend(default.map(|default| ("default", input(default))));
    by_name(b, "LocalIn", selector, &args)
}

/// `LocalOut.selector(channels)`.
fn local_out(b: &mut SynthDefBuilder, selector: &str, channels: UGenInput) {
    by_name(
        b,
        "LocalOut",
        selector,
        &[("channelsArray", input(channels))],
    );
}

fn localin() -> Vec<SynthDef> {
    vec![
        def("localin_ar", vec![], |b| {
            let back = local_in(b, "ar", 2.0, None);
            let half = b.binary_op("*", back.clone(), 0.5).unwrap();
            local_out(b, "ar", half);
            out(b, "ar", back);
        }),
        def("localin_default", vec![], |b| {
            let back = local_in(b, "ar", 3.0, Some(array(&[0.1, 0.2])));
            local_out(b, "ar", back.clone());
            out(b, "ar", back);
        }),
        def("localin_kr", vec![], |b| {
            let back = local_in(b, "kr", 1.0, Some(0.5.into()));
            let more = b.binary_op("+", back.clone(), 1.0).unwrap();
            local_out(b, "kr", more);
            out(b, "kr", back);
        }),
        def("localin_zero", vec![], |b| {
            let back = local_in(b, "ar", 2.0, None);
            let channels = UGenInput::Array(vec![element(&back, 0), 0.0.into()]);
            local_out(b, "ar", channels);
            out(b, "ar", back);
        }),
    ]
}

/// `Poll.selector(trig, sound)`, with these other arguments.
fn poll_of(
    b: &mut SynthDefBuilder,
    selector: &str,
    trig: UGenInput,
    sound: UGenInput,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![("trig", input(trig)), ("in", input(sound))];
    all.extend(args.iter().cloned());
    by_name(b, "Poll", selector, &all)
}

fn poll() -> Vec<SynthDef> {
    let text = |label: &str| ArgValue::Text(label.to_owned());
    vec![
        def("poll_kr", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            poll_of(b, "kr", 10.0.into(), sound, &[]);
        }),
        def("poll_label", vec![], |b| {
            let trig = by_name(b, "Impulse", "ar", &[("freq", input(2.0))]);
            let sound = sine(b, "ar", 1.0);
            let args = [("label", text("freq")), ("trigid", input(7.0))];
            poll_of(b, "ar", trig, sound, &args);
        }),
        def("poll_symbol", vec![], |b| {
            let trig = by_name(b, "Impulse", "kr", &[("freq", input(2.0))]);
            let sound = sine(b, "kr", 1.0);
            poll_of(b, "kr", trig, sound, &[("label", text("amp"))]);
        }),
        def("poll_proxy", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            let pair = by_name(b, "Pan2", "kr", &[("in", input(sound))]);
            poll_of(b, "kr", 10.0.into(), pair, &[]);
        }),
        def("poll_integer", vec![], |b| {
            poll_of(b, "kr", 10.0.into(), 440.0.into(), &[]);
        }),
        def("poll_float", vec![], |b| {
            poll_of(b, "kr", 10.0.into(), 0.5.into(), &[]);
        }),
        def("poll_new", vec![Arg::new("freq", 440.0)], |b| {
            let freq = arg(b, "freq");
            poll_of(b, "new", 10.0.into(), freq, &[]);
        }),
        def("poll_new_scalar", vec![Arg::new("i_x", 1.0)], |b| {
            let x = arg(b, "i_x");
            poll_of(b, "new", 10.0.into(), x, &[]);
        }),
        def("poll_out", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let polled = poll_of(b, "ar", 10.0.into(), sound, &[]);
            out(b, "ar", polled);
        }),
        def("poll_bytes", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            poll_of(b, "kr", 10.0.into(), sound, &[("label", text("é"))]);
        }),
    ]
}

fn symbols() -> Vec<SynthDef> {
    vec![def("sendreply_bytes", vec![], |b| {
        let trig = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
        let sound = sine(b, "kr", 1.0);
        by_name(
            b,
            "SendReply",
            "kr",
            &[
                ("trig", input(trig)),
                ("cmdName", ArgValue::Text("/é".to_owned())),
                ("values", input(sound)),
            ],
        );
    })]
}

/// `Phasor.selector(0, 1, 0, end)`.
fn phasor(b: &mut SynthDefBuilder, selector: &str, end: f64) -> UGenInput {
    by_name(
        b,
        "Phasor",
        selector,
        &[
            ("trig", input(0.0)),
            ("rate", input(1.0)),
            ("start", input(0.0)),
            ("end", input(end)),
        ],
    )
}

fn bufwr() -> Vec<SynthDef> {
    vec![
        def("bufwr_ar", vec![], |b| {
            let buffer = local_buf(b, 1024.0, 2.0);
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let phase = phasor(b, "ar", 1024.0);
            by_name(
                b,
                "BufWr",
                "ar",
                &[
                    ("inputArray", input(sounds)),
                    ("bufnum", input(buffer)),
                    ("phase", input(phase)),
                ],
            );
        }),
        def("bufwr_kr", vec![], |b| {
            let buffer = local_buf(b, 64.0, 1.0);
            let sound = sine(b, "kr", 1.0);
            let phase = phasor(b, "kr", 64.0);
            by_name(
                b,
                "BufWr",
                "kr",
                &[
                    ("inputArray", input(sound)),
                    ("bufnum", input(buffer)),
                    ("phase", input(phase)),
                    ("loop", input(0.0)),
                ],
            );
        }),
    ]
}

/// `Dseq(list, inf)`.
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

/// `class.selector(dur, reset, level)`, with these other arguments.
fn duty_of(
    b: &mut SynthDefBuilder,
    class: &str,
    selector: &str,
    dur: UGenInput,
    reset: UGenInput,
    level: UGenInput,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![
        ("dur", input(dur)),
        ("reset", input(reset)),
        ("level", input(level)),
    ];
    all.extend(args.iter().cloned());
    by_name(b, class, selector, &all)
}

fn duty() -> Vec<SynthDef> {
    vec![
        def("duty_kr", vec![], |b| {
            let dur = dseq(b, &[0.1, 0.2]);
            let level = dseq(b, &[1.0, 2.0, 3.0]);
            let duty = duty_of(b, "Duty", "kr", dur, 0.0.into(), level, &[]);
            out(b, "kr", duty);
        }),
        def("duty_ar", vec![], |b| {
            let dur = dseq(b, &[0.1, 0.2]);
            let reset = by_name(b, "Impulse", "ar", &[("freq", input(1.0))]);
            let level = dseq(b, &[1.0, 2.0]);
            let duty = duty_of(b, "Duty", "ar", dur, reset, level, &[]);
            out(b, "ar", duty);
        }),
        def("duty_demand_reset", vec![], |b| {
            let dur = dseq(b, &[0.1]);
            let reset = dseq(b, &[1.0, 0.0]);
            let level = by_name(
                b,
                "Dwhite",
                "new",
                &[("lo", input(0.0)), ("hi", input(1.0))],
            );
            let duty = duty_of(b, "Duty", "kr", dur, reset, level, &[]);
            out(b, "kr", duty);
        }),
        def("duty_number", vec![], |b| {
            let level = dseq(b, &[1.0, 2.0]);
            let duty = duty_of(b, "Duty", "kr", 0.5.into(), 0.0.into(), level, &[]);
            out(b, "kr", duty);
        }),
        def("tduty_ar", vec![], |b| {
            let dur = dseq(b, &[0.1, 0.2]);
            let args = [("doneAction", input(0.0)), ("gapFirst", input(1.0))];
            let duty = duty_of(b, "TDuty", "ar", dur, 0.0.into(), 1.0.into(), &args);
            out(b, "ar", duty);
        }),
    ]
}

/// `class.selector(args)` into `Out` at the rate of `selector`.
fn rule_out(b: &mut SynthDefBuilder, class: &str, selector: &str, args: &[(&str, ArgValue)]) {
    let result = by_name(b, class, selector, args);
    out(b, selector, result);
}

fn peak() -> Vec<SynthDef> {
    vec![
        def("peak_kr_audio", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let trig = by_name(b, "Impulse", "kr", &[("freq", input(1.0))]);
            rule_out(
                b,
                "Peak",
                "kr",
                &[("in", input(sound)), ("trig", input(trig))],
            );
        }),
        def("peak_ar", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            let trig = by_name(b, "Impulse", "ar", &[("freq", input(2.0))]);
            rule_out(
                b,
                "Peak",
                "ar",
                &[("in", input(sound)), ("trig", input(trig))],
            );
        }),
        def("runningmax_kr", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            rule_out(b, "RunningMax", "kr", &[("in", input(sound))]);
        }),
        def("runningmin_ar", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            rule_out(b, "RunningMin", "ar", &[("in", input(sound))]);
        }),
    ]
}

fn t2k() -> Vec<SynthDef> {
    vec![def("t2k", vec![], |b| {
        let trig = by_name(b, "Impulse", "ar", &[("freq", input(10.0))]);
        rule_out(b, "T2K", "kr", &[("in", input(trig))]);
    })]
}

/// A spec of `Klank` or `Klang`: its parts as arrays of numbers.
fn spec_of(parts: &[&[f64]]) -> UGenInput {
    UGenInput::Array(parts.iter().map(|part| array(part)).collect())
}

/// `Klank.ar(spec, input)`, with these other arguments, into `Out.ar`.
fn klank_out(
    b: &mut SynthDefBuilder,
    parts: &[&[f64]],
    impulses: UGenInput,
    args: &[(&str, ArgValue)],
) {
    let trig = by_name(b, "Impulse", "ar", &[("freq", input(impulses))]);
    let mut all = vec![
        ("specificationsArrayRef", input(spec_of(parts))),
        ("input", input(trig)),
    ];
    all.extend(args.iter().cloned());
    let rings = by_name(b, "Klank", "ar", &all);
    out(b, "ar", rings);
}

fn klank() -> Vec<SynthDef> {
    vec![
        def("klank_3", vec![], |b| {
            let parts: &[&[f64]] = &[&[800.0, 1071.0, 1353.0], &[0.3, 0.2, 0.1], &[1.0, 1.0, 1.0]];
            klank_out(b, parts, 2.0.into(), &[]);
        }),
        def("klank_amps", vec![], |b| {
            klank_out(b, &[&[800.0, 1071.0], &[0.5, 0.3]], 2.0.into(), &[]);
        }),
        def("klank_times", vec![], |b| {
            let parts: &[&[f64]] = &[&[800.0, 1071.0], &[1.0], &[1.0, 2.0]];
            klank_out(b, parts, 2.0.into(), &[]);
        }),
        def("klank_wrap", vec![], |b| {
            let parts: &[&[f64]] = &[&[800.0, 1071.0, 1353.0], &[0.5, 0.3]];
            klank_out(b, parts, 2.0.into(), &[]);
        }),
        def("klank_scale", vec![Arg::new("scale", 1.0)], |b| {
            let scale = arg(b, "scale");
            klank_out(
                b,
                &[&[800.0, 1071.0]],
                2.0.into(),
                &[("freqscale", input(scale))],
            );
        }),
        def("klank_expand", vec![], |b| {
            klank_out(b, &[&[800.0, 1071.0]], array(&[2.0, 3.0]), &[]);
        }),
    ]
}

fn klang() -> Vec<SynthDef> {
    let klang_out = |b: &mut SynthDefBuilder, parts: &[&[f64]], args: &[(&str, ArgValue)]| {
        let mut all = vec![("specificationsArrayRef", input(spec_of(parts)))];
        all.extend(args.iter().cloned());
        let sines = by_name(b, "Klang", "ar", &all);
        out(b, "ar", sines);
    };
    vec![
        def("klang_3", vec![], |b| {
            let pi = std::f64::consts::PI;
            klang_out(b, &[&[800.0, 1000.0], &[0.3, 0.3], &[pi, pi]], &[]);
        }),
        def("klang_default", vec![], |b| {
            klang_out(b, &[&[800.0, 1000.0, 1200.0]], &[]);
        }),
        def("klang_scale", vec![], |b| {
            let scale = by_name(
                b,
                "SinOsc",
                "kr",
                &[
                    ("freq", input(0.1)),
                    ("mul", input(0.1)),
                    ("add", input(1.0)),
                ],
            );
            klang_out(b, &[&[800.0, 1000.0]], &[("freqscale", input(scale))]);
        }),
    ]
}

/// `PingPong.ar(bufnum, inputs, args)` into `Out.ar`.
fn ping_pong_out(
    b: &mut SynthDefBuilder,
    bufnum: UGenInput,
    inputs: UGenInput,
    args: &[(&str, ArgValue)],
) {
    let mut all = vec![("bufnum", input(bufnum)), ("inputs", input(inputs))];
    all.extend(args.iter().cloned());
    let delayed = by_name(b, "PingPong", "ar", &all);
    out(b, "ar", delayed);
}

/// `[SinOsc.ar(1), SinOsc.ar(2)]`.
fn two_sines(b: &mut SynthDefBuilder) -> UGenInput {
    UGenInput::Array(vec![sine(b, "ar", 1.0), sine(b, "ar", 2.0)])
}

fn pingpong() -> Vec<SynthDef> {
    vec![
        def("pingpong", vec![], |b| {
            let buffer = local_buf(b, 44100.0, 2.0);
            let sounds = two_sines(b);
            ping_pong_out(b, buffer, sounds, &[("delayTime", input(0.25))]);
        }),
        def("pingpong_rotate", vec![], |b| {
            let buffer = local_buf(b, 44100.0, 2.0);
            let sounds = two_sines(b);
            let args = [
                ("delayTime", input(0.25)),
                ("feedback", input(0.5)),
                ("rotate", input(0.0)),
            ];
            ping_pong_out(b, buffer, sounds, &args);
        }),
        def("pingpong_negative", vec![], |b| {
            let buffer = local_buf(b, 44100.0, 3.0);
            let sounds = sine(b, "ar", array(&[1.0, 2.0, 3.0]));
            let args = [
                ("delayTime", input(0.25)),
                ("feedback", input(0.7)),
                ("rotate", input(-1.0)),
            ];
            ping_pong_out(b, buffer, sounds, &args);
        }),
        def("pingpong_ugens", vec![Arg::new("fb", 0.5)], |b| {
            let fb = arg(b, "fb");
            let buffer = local_buf(b, 44100.0, 2.0);
            let sounds = two_sines(b);
            let delay = by_name(
                b,
                "SinOsc",
                "kr",
                &[
                    ("freq", input(0.1)),
                    ("mul", input(0.1)),
                    ("add", input(0.2)),
                ],
            );
            let args = [("delayTime", input(delay)), ("feedback", input(fb))];
            ping_pong_out(b, buffer, sounds, &args);
        }),
        def("pingpong_bufnum", vec![], |b| {
            let sounds = two_sines(b);
            ping_pong_out(b, 0.0.into(), sounds, &[("delayTime", input(0.25))]);
        }),
    ]
}
