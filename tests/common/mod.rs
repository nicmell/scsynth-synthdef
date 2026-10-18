//! The graphs of the golden files, built with the crate. `generator/fixtures.scd`
//! holds the sclang code of each.
#![allow(dead_code)]

use std::fs;
use std::path::Path;

use scsynth_synthdef::{
    Arg, ArgRate, NewUGen, Rate, SynthDef, SynthDefBuilder, UGenInput, write_def_file,
    write_def_file_old,
};

/// The golden files that the builder reproduces.
pub const NAMES: [&str; 14] = [
    "sine",
    "rates",
    "lags17",
    "multi_out",
    "sort",
    "fft",
    "demand",
    "constants",
    "operators",
    "variant",
    "two_defs",
    "old",
    "envgen",
    "sendreply",
];

/// The bytes of a golden file.
pub fn golden(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{name}.scsyndef"));
    fs::read(path).expect("the golden file reads")
}

/// The definitions of a golden file, built with `add`.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "sine" => vec![sine()],
        "rates" => vec![rates()],
        "lags17" => vec![lags17()],
        "multi_out" => vec![multi_out()],
        "sort" => vec![sort()],
        "fft" => vec![fft()],
        "demand" => vec![demand()],
        "constants" => vec![constants()],
        "operators" => vec![operators()],
        "variant" => vec![variant()],
        "two_defs" => vec![first(), second()],
        "old" => vec![old()],
        "envgen" => vec![envgen()],
        "sendreply" => vec![sendreply()],
        _ => panic!("no golden graph is named {name}"),
    }
}

/// The bytes that the crate writes for a golden file.
pub fn written(name: &str) -> Vec<u8> {
    let defs = definitions(name);
    let bytes = if name == "old" {
        write_def_file_old(&defs)
    } else {
        write_def_file(&defs)
    };
    bytes.expect("a golden graph meets the limits")
}

pub fn arg(builder: &SynthDefBuilder, name: &str) -> UGenInput {
    builder.argument(name).expect("an argument").clone()
}

pub fn element(input: &UGenInput, index: usize) -> UGenInput {
    let UGenInput::Array(inputs) = input else {
        panic!("{input:?} is not an array");
    };
    inputs[index].clone()
}

/// A UGen with one output at its rate.
pub fn ugen(
    builder: &mut SynthDefBuilder,
    name: &str,
    rate: Rate,
    inputs: Vec<UGenInput>,
) -> UGenInput {
    builder
        .add(NewUGen::new(name, rate).inputs(inputs))
        .expect("the UGen adds")
}

pub fn out(builder: &mut SynthDefBuilder, rate: Rate, inputs: Vec<UGenInput>) {
    builder
        .add(NewUGen::new("Out", rate).inputs(inputs).outputs([]))
        .expect("the Out adds");
}

/// A `BinaryOpUGen` by special index, at the highest rate of its inputs.
pub fn binary(
    builder: &mut SynthDefBuilder,
    special_index: i16,
    a: UGenInput,
    b: UGenInput,
) -> UGenInput {
    let rate = a.rate().max(b.rate()).expect("an input has a rate");
    builder
        .add(
            NewUGen::new("BinaryOpUGen", rate)
                .inputs([a, b])
                .special_index(special_index),
        )
        .expect("the operator adds")
}

/// A `UnaryOpUGen` by special index, at the rate of its input.
pub fn unary(builder: &mut SynthDefBuilder, special_index: i16, a: UGenInput) -> UGenInput {
    let rate = a.rate().expect("the input has a rate");
    builder
        .add(
            NewUGen::new("UnaryOpUGen", rate)
                .inputs([a])
                .special_index(special_index),
        )
        .expect("the operator adds")
}

fn numbers(numbers: &[f64]) -> Vec<UGenInput> {
    numbers.iter().copied().map(UGenInput::from).collect()
}

pub fn sine() -> SynthDef {
    let mut b = SynthDefBuilder::new(
        "sine",
        vec![
            Arg::new("out", 0.0),
            Arg::new("freq", 440.0),
            Arg::new("amp", 0.1),
        ],
    )
    .unwrap();
    let (out_bus, freq, amp) = (arg(&b, "out"), arg(&b, "freq"), arg(&b, "amp"));
    let sin = ugen(&mut b, "SinOsc", Rate::Audio, vec![freq, 0.0.into()]);
    let product = binary(&mut b, 2, sin, amp);
    out(&mut b, Rate::Audio, vec![out_bus, product]);
    b.finish().unwrap()
}

pub fn rates() -> SynthDef {
    let mut b = SynthDefBuilder::new(
        "rates",
        vec![
            Arg::new("freq", 440.0).lag(0.1),
            Arg::new("gate", 1.0).rate(ArgRate::Tr),
            Arg::new("src", 0.0).rate(ArgRate::Ar),
            Arg::new("n", 2.0).rate(ArgRate::Ir),
            Arg::new("i_k", 3.0).rate(ArgRate::Kr),
            Arg::new("a_in", 0.0),
            Arg::new("t_x", 0.0),
            Arg::array("amps", [1.0, 2.0]).lag(0.2),
            Arg::new("level", 0.5),
        ],
    )
    .unwrap();
    let amps = arg(&b, "amps");
    let factors = [
        element(&amps, 0),
        element(&amps, 1),
        arg(&b, "level"),
        arg(&b, "gate"),
        arg(&b, "src"),
        arg(&b, "n"),
        arg(&b, "i_k"),
        arg(&b, "a_in"),
        arg(&b, "t_x"),
    ];
    let freq = arg(&b, "freq");
    let mut sig = ugen(&mut b, "SinOsc", Rate::Audio, vec![freq, 0.0.into()]);
    for factor in factors {
        sig = binary(&mut b, 2, sig, factor);
    }
    out(&mut b, Rate::Audio, vec![0.0.into(), sig]);
    b.finish().unwrap()
}

pub fn lags17() -> SynthDef {
    let values: Vec<f64> = (1..=17).map(f64::from).collect();
    let mut b =
        SynthDefBuilder::new("lags17", vec![Arg::array("values", values).lag(0.1)]).unwrap();
    let UGenInput::Array(values) = arg(&b, "values") else {
        panic!("an array argument gives an array");
    };
    out(&mut b, Rate::Control, [vec![0.0.into()], values].concat());
    b.finish().unwrap()
}

pub fn multi_out() -> SynthDef {
    let mut b = SynthDefBuilder::new("multi_out", vec![Arg::new("pos", 0.0)]).unwrap();
    let pos = arg(&b, "pos");
    let input = b
        .add(
            NewUGen::new("In", Rate::Audio)
                .inputs([2.0.into()])
                .outputs([Rate::Audio, Rate::Audio]),
        )
        .unwrap();
    let pan = b
        .add(
            NewUGen::new("Pan2", Rate::Audio)
                .inputs([element(&input, 0), pos, 1.0.into()])
                .outputs([Rate::Audio, Rate::Audio]),
        )
        .unwrap();
    out(
        &mut b,
        Rate::Audio,
        vec![0.0.into(), element(&pan, 0), element(&pan, 1)],
    );
    out(&mut b, Rate::Audio, vec![2.0.into(), element(&input, 1)]);
    b.finish().unwrap()
}

pub fn sort() -> SynthDef {
    let mut b = SynthDefBuilder::new("sort", vec![]).unwrap();
    let sine = ugen(&mut b, "SinOsc", Rate::Audio, numbers(&[220.0, 0.0]));
    let saw = ugen(&mut b, "LFSaw", Rate::Audio, numbers(&[1.0, 0.0]));
    let low = ugen(&mut b, "LPF", Rate::Audio, vec![sine.clone(), 800.0.into()]);
    let high = ugen(&mut b, "HPF", Rate::Audio, vec![sine, 400.0.into()]);
    let resonant = ugen(
        &mut b,
        "RLPF",
        Rate::Audio,
        vec![saw, 600.0.into(), 1.0.into()],
    );
    let difference = binary(&mut b, 1, low, high);
    out(&mut b, Rate::Audio, vec![0.0.into(), difference, resonant]);
    b.finish().unwrap()
}

pub fn fft() -> SynthDef {
    let mut b = SynthDefBuilder::new("fft", vec![]).unwrap();
    let input = ugen(&mut b, "In", Rate::Audio, vec![0.0.into()]);
    let chain = b
        .add(
            NewUGen::new("FFT", Rate::Control)
                .inputs([
                    0.0.into(),
                    input,
                    0.5.into(),
                    0.0.into(),
                    1.0.into(),
                    0.0.into(),
                ])
                .width_first(true),
        )
        .unwrap();
    let x = ugen(&mut b, "SinOsc", Rate::Audio, numbers(&[2.0, 0.0]));
    let chain = b
        .add(
            NewUGen::new("PV_MagAbove", Rate::Control)
                .inputs([chain, 0.5.into()])
                .width_first(true),
        )
        .unwrap();
    let ifft = b
        .add(
            NewUGen::new("IFFT", Rate::Audio)
                .inputs([chain, 0.0.into(), 0.0.into()])
                .width_first(true),
        )
        .unwrap();
    let product = binary(&mut b, 2, ifft, x);
    out(&mut b, Rate::Audio, vec![0.0.into(), product]);
    b.finish().unwrap()
}

pub fn demand() -> SynthDef {
    let mut b = SynthDefBuilder::new("demand", vec![]).unwrap();
    let trig = ugen(&mut b, "Impulse", Rate::Control, numbers(&[4.0, 0.0]));
    let seq = ugen(
        &mut b,
        "Dseq",
        Rate::Demand,
        numbers(&[f64::INFINITY, 60.0, 62.0, 64.0]),
    );
    let demand = ugen(&mut b, "Demand", Rate::Control, vec![trig, 0.0.into(), seq]);
    out(&mut b, Rate::Control, vec![0.0.into(), demand]);
    b.finish().unwrap()
}

pub fn constants() -> SynthDef {
    let mut b = SynthDefBuilder::new("constants", vec![]).unwrap();
    let channels = numbers(&[
        0.0,
        1.0,
        1.0,
        0.5,
        0.5,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1.0 + 1e-12,
        2.0,
    ]);
    out(&mut b, Rate::Control, channels);
    b.finish().unwrap()
}

pub fn operators() -> SynthDef {
    let mut b = SynthDefBuilder::new(
        "operators",
        vec![Arg::new("freq", 440.0), Arg::new("i_n", 2.0)],
    )
    .unwrap();
    let (freq, i_n) = (arg(&b, "freq"), arg(&b, "i_n"));
    let a = ugen(
        &mut b,
        "SinOsc",
        Rate::Audio,
        vec![freq.clone(), 0.0.into()],
    );
    let abs = unary(&mut b, 5, a.clone());
    let neg = unary(&mut b, 0, a.clone());
    let power = binary(&mut b, 25, neg, 2.0.into());
    let difference = binary(&mut b, 1, a.clone(), freq.clone());
    let max = binary(&mut b, 13, difference, 0.1.into());
    let less = binary(&mut b, 8, a, 0.5.into());
    out(&mut b, Rate::Audio, vec![0.0.into(), abs, power, max, less]);
    let midicps = unary(&mut b, 17, freq);
    let doubled = binary(&mut b, 2, i_n, 2.0.into());
    let trig = ugen(&mut b, "Impulse", Rate::Control, numbers(&[1.0, 0.0]));
    let white = ugen(
        &mut b,
        "Dwhite",
        Rate::Demand,
        numbers(&[f64::INFINITY, 0.0, 1.0]),
    );
    let tripled = binary(&mut b, 2, white, 3.0.into());
    let demand = ugen(
        &mut b,
        "Demand",
        Rate::Control,
        vec![trig, 0.0.into(), tripled],
    );
    out(
        &mut b,
        Rate::Control,
        vec![0.0.into(), midicps, doubled, demand],
    );
    b.finish().unwrap()
}

pub fn variant() -> SynthDef {
    let mut b = SynthDefBuilder::new(
        "variant",
        vec![Arg::new("freq", 440.0), Arg::new("amp", 0.1)],
    )
    .unwrap();
    let (freq, amp) = (arg(&b, "freq"), arg(&b, "amp"));
    let sin = ugen(&mut b, "SinOsc", Rate::Audio, vec![freq, 0.0.into()]);
    let product = binary(&mut b, 2, sin, amp);
    out(&mut b, Rate::Audio, vec![0.0.into(), product]);
    b.variant("low", &[("freq", vec![220.0])]).unwrap();
    b.finish().unwrap()
}

pub fn first() -> SynthDef {
    let mut b = SynthDefBuilder::new("first", vec![]).unwrap();
    let dc = ugen(&mut b, "DC", Rate::Control, vec![1.0.into()]);
    out(&mut b, Rate::Control, vec![0.0.into(), dc]);
    b.finish().unwrap()
}

pub fn second() -> SynthDef {
    let mut b = SynthDefBuilder::new("second", vec![Arg::new("level", 0.5)]).unwrap();
    let level = arg(&b, "level");
    let dc = ugen(&mut b, "DC", Rate::Control, vec![level]);
    out(&mut b, Rate::Control, vec![1.0.into(), dc]);
    b.finish().unwrap()
}

pub fn old() -> SynthDef {
    let mut b = SynthDefBuilder::new("old", vec![Arg::new("freq", 440.0)]).unwrap();
    let freq = arg(&b, "freq");
    let sin = ugen(&mut b, "SinOsc", Rate::Audio, vec![freq, 0.0.into()]);
    out(&mut b, Rate::Audio, vec![0.0.into(), sin]);
    b.variant("low", &[("freq", vec![220.0])]).unwrap();
    b.finish().unwrap()
}

/// `Env.perc(0.01, 1).asArray`.
pub const PERC: [f64; 12] = [
    0.0, 2.0, -99.0, -99.0, 1.0, 0.01, 5.0, -4.0, 0.0, 1.0, 5.0, -4.0,
];

pub fn envgen() -> SynthDef {
    let mut b = SynthDefBuilder::new("envgen", vec![Arg::new("gate", 1.0)]).unwrap();
    let gate = arg(&b, "gate");
    let sin = ugen(&mut b, "SinOsc", Rate::Audio, numbers(&[440.0, 0.0]));
    let mut inputs = vec![gate];
    inputs.extend(numbers(&[1.0, 0.0, 1.0, 2.0]));
    inputs.extend(numbers(&PERC));
    let envelope = ugen(&mut b, "EnvGen", Rate::Control, inputs);
    let product = binary(&mut b, 2, sin, envelope);
    out(&mut b, Rate::Audio, vec![0.0.into(), product]);
    b.finish().unwrap()
}

pub fn sendreply() -> SynthDef {
    let mut b = SynthDefBuilder::new("sendreply", vec![Arg::new("freq", 440.0)]).unwrap();
    let freq = arg(&b, "freq");
    let trig = ugen(&mut b, "Impulse", Rate::Control, numbers(&[1.0, 0.0]));
    let mut inputs = vec![trig, 3.0.into(), 5.0.into()];
    inputs.extend("/tick".bytes().map(|byte| UGenInput::from(f64::from(byte))));
    inputs.extend([freq, 7.0.into()]);
    b.add(
        NewUGen::new("SendReply", Rate::Control)
            .inputs(inputs)
            .outputs([]),
    )
    .unwrap();
    b.finish().unwrap()
}
