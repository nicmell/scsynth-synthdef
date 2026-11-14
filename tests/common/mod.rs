//! The graphs of the golden files, built with the crate. `generator/fixtures.scd`
//! holds the sclang code of each.
#![allow(dead_code)]

mod pseudo;

use std::fs;
use std::path::Path;

use scsynth_synthdef::{
    Arg, ArgRate, ArgValue, NewUGen, Rate, SynthDef, SynthDefBuilder, UGenInput, write_def_file,
    write_def_file_old,
};

/// The golden files that the builder reproduces.
pub const NAMES: &[&str] = &[
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
    "pair",
    "wrap",
    "voices",
    "nested",
    "in_buses",
    "mul",
    "mul_add",
    "mul_array",
    "mul_expanded",
    "mul_swap",
    "mul_ops",
    "madd_numbers",
    "folds",
    "pmosc",
    "changed",
    "silent",
    "dstutter",
    "tchoose",
    "twchoose",
    "tap",
    "linlin",
    "soundin",
    "selectx",
    "blowpass4",
    "bhipass4",
    "sum",
    "mix",
    "numchannels",
    "selectxfocus",
    "multitap",
    "splay",
    "dynklank",
    "dynklang",
    "unpackfft",
    "delays",
    "linxfade2",
    "companderd",
    "hilbertfir",
    "linselectx",
    "splayaz",
    "lag",
    "outzeros",
    "localbuf",
    "beattrack",
    "localin",
    "poll",
    "symbols",
    "bufwr",
    "duty",
    "peak",
    "t2k",
    "klank",
    "klang",
];

/// The bytes of a golden file: the build without `optimizeGraph`, where
/// `tests/fixtures/before_optimize` holds one.
pub fn golden(name: &str) -> Vec<u8> {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let file = format!("{name}.scsyndef");
    let before_optimize = fixtures.join("before_optimize").join(&file);
    let path = if before_optimize.exists() {
        before_optimize
    } else {
        fixtures.join(file)
    };
    fs::read(path).expect("the golden file reads")
}

/// The definitions of a golden file: those of 0.1 built with `add`, and
/// those of 0.2 with `ugen` and the operators.
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
        "pair" => vec![pair()],
        "wrap" => vec![wrap()],
        "voices" => vec![voices()],
        "nested" => vec![nested()],
        "in_buses" => vec![in_buses()],
        "mul" => vec![sine_madd("mul", 440.0.into(), 0.1.into(), 0.0.into())],
        "mul_add" => vec![sine_madd("mul_add", 440.0.into(), 0.1.into(), 0.5.into())],
        "mul_array" => vec![sine_madd(
            "mul_array",
            array(&[440.0, 442.0]),
            array(&[0.1, 0.2, 0.3]),
            0.0.into(),
        )],
        "mul_expanded" => vec![sine_madd(
            "mul_expanded",
            array(&[440.0, 442.0]),
            0.5.into(),
            0.0.into(),
        )],
        "mul_swap" => vec![mul_swap()],
        "mul_ops" => vec![mul_ops()],
        "madd_numbers" => vec![madd_numbers()],
        "folds" => vec![folds()],
        _ => pseudo::definitions(name),
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

/// An array of numbers, as one input.
fn array(values: &[f64]) -> UGenInput {
    UGenInput::Array(numbers(values))
}

/// The elements of a nested array, in order, as `flat` gives them in sclang.
fn flat(input: UGenInput) -> Vec<UGenInput> {
    match input {
        UGenInput::Array(items) => items.into_iter().flat_map(flat).collect(),
        other => vec![other],
    }
}

/// A UGen by its class name, with the arguments by name.
fn by_name(
    builder: &mut SynthDefBuilder,
    class: &str,
    selector: &str,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    builder
        .ugen(class, selector, args)
        .unwrap_or_else(|error| panic!("{class}.{selector}: {error}"))
}

/// `Out.ar(0, channels)`, by class name.
fn out_by_name(builder: &mut SynthDefBuilder, channels: UGenInput) {
    by_name(
        builder,
        "Out",
        "ar",
        &[("bus", 0.0.into()), ("channelsArray", channels.into())],
    );
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

/// The `operators` golden graph, with `unary_op` and `binary_op`.
pub fn operators_by_name() -> SynthDef {
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
    let abs = b.unary_op("abs", a.clone()).unwrap();
    let neg = b.unary_op("neg", a.clone()).unwrap();
    let power = b.binary_op("pow", neg, 2.0).unwrap();
    let difference = b.binary_op("-", a.clone(), freq.clone()).unwrap();
    let max = b.binary_op("max", difference, 0.1).unwrap();
    let less = b.binary_op("<", a, 0.5).unwrap();
    out(&mut b, Rate::Audio, vec![0.0.into(), abs, power, max, less]);
    let midicps = b.unary_op("midicps", freq).unwrap();
    let doubled = b.binary_op("*", i_n, 2.0).unwrap();
    let trig = ugen(&mut b, "Impulse", Rate::Control, numbers(&[1.0, 0.0]));
    let white = ugen(
        &mut b,
        "Dwhite",
        Rate::Demand,
        numbers(&[f64::INFINITY, 0.0, 1.0]),
    );
    let tripled = b.binary_op("*", white, 3.0).unwrap();
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

pub fn pair() -> SynthDef {
    let mut b = SynthDefBuilder::new(
        "pair",
        vec![Arg::array("freqs", [220.0, 331.0]), Arg::new("amp", 0.4)],
    )
    .unwrap();
    let (freqs, amp) = (arg(&b, "freqs"), arg(&b, "amp"));
    let sines = by_name(&mut b, "SinOsc", "ar", &[("freq", freqs.into())]);
    let sound = b.binary_op("*", sines, amp).unwrap();
    out_by_name(&mut b, sound);
    b.finish().unwrap()
}

pub fn wrap() -> SynthDef {
    let mut b = SynthDefBuilder::new("wrap", vec![]).unwrap();
    let freqs = array(&[440.0, 442.0, 444.0]);
    let sines = by_name(&mut b, "SinOsc", "ar", &[("freq", freqs.into())]);
    let sound = b.binary_op("*", sines, array(&[0.1, 0.2])).unwrap();
    out_by_name(&mut b, sound);
    b.finish().unwrap()
}

pub fn voices() -> SynthDef {
    let mut b = SynthDefBuilder::new(
        "voices",
        vec![
            Arg::array("freqs", [220.0, 330.0]),
            Arg::array("places", [-0.5, 0.5]),
            Arg::new("amp", 0.2),
        ],
    )
    .unwrap();
    let (freqs, places, amp) = (arg(&b, "freqs"), arg(&b, "places"), arg(&b, "amp"));
    let sines = by_name(&mut b, "SinOsc", "ar", &[("freq", freqs.into())]);
    let voices = b.binary_op("*", sines, amp).unwrap();
    let pans = by_name(
        &mut b,
        "Pan2",
        "ar",
        &[("in", voices.into()), ("pos", places.into())],
    );
    let sum = b
        .binary_op("+", element(&pans, 0), element(&pans, 1))
        .unwrap();
    out_by_name(&mut b, sum);
    b.finish().unwrap()
}

pub fn nested() -> SynthDef {
    let mut b = SynthDefBuilder::new("nested", vec![]).unwrap();
    let freqs = array(&[220.0, 330.0]);
    let sines = by_name(&mut b, "SinOsc", "ar", &[("freq", freqs.into())]);
    let places = array(&[-0.5, 0.5]);
    let pans = by_name(
        &mut b,
        "Pan2",
        "ar",
        &[("in", sines.into()), ("pos", places.into())],
    );
    let low = by_name(
        &mut b,
        "LPF",
        "ar",
        &[("in", pans.into()), ("freq", 1000.0.into())],
    );
    let sum = b
        .binary_op("+", element(&low, 0), element(&low, 1))
        .unwrap();
    out_by_name(&mut b, sum);
    b.finish().unwrap()
}

pub fn in_buses() -> SynthDef {
    let mut b = SynthDefBuilder::new("in_buses", vec![]).unwrap();
    let inputs = by_name(
        &mut b,
        "In",
        "ar",
        &[
            ("bus", array(&[4.0, 6.0]).into()),
            ("numChannels", array(&[1.0, 2.0]).into()),
        ],
    );
    out_by_name(&mut b, UGenInput::Array(flat(inputs)));
    b.finish().unwrap()
}

/// `Out.ar(0, SinOsc.ar(freq, 0, mul, add))`.
fn sine_madd(name: &str, freq: UGenInput, mul: UGenInput, add: UGenInput) -> SynthDef {
    let mut b = SynthDefBuilder::new(name, vec![]).unwrap();
    let sound = by_name(
        &mut b,
        "SinOsc",
        "ar",
        &[
            ("freq", freq.into()),
            ("mul", mul.into()),
            ("add", add.into()),
        ],
    );
    out_by_name(&mut b, sound);
    b.finish().unwrap()
}

/// `SinOsc.ar(2)`, which sclang evaluates before the method that takes it.
fn fast(builder: &mut SynthDefBuilder) -> UGenInput {
    by_name(builder, "SinOsc", "ar", &[("freq", 2.0.into())])
}

pub fn mul_swap() -> SynthDef {
    let mut b = SynthDefBuilder::new("mul_swap", vec![]).unwrap();
    let fast = fast(&mut b);
    let sound = by_name(
        &mut b,
        "SinOsc",
        "kr",
        &[
            ("freq", 1.0.into()),
            ("mul", fast.into()),
            ("add", 0.5.into()),
        ],
    );
    out_by_name(&mut b, sound);
    b.finish().unwrap()
}

pub fn mul_ops() -> SynthDef {
    let mut b = SynthDefBuilder::new("mul_ops", vec![]).unwrap();
    let fast = fast(&mut b);
    let sound = by_name(
        &mut b,
        "SinOsc",
        "kr",
        &[
            ("freq", 1.0.into()),
            ("mul", 0.5.into()),
            ("add", fast.into()),
        ],
    );
    out_by_name(&mut b, sound);
    b.finish().unwrap()
}

pub fn madd_numbers() -> SynthDef {
    let mut b = SynthDefBuilder::new("madd_numbers", vec![]).unwrap();
    let fast = fast(&mut b);
    let sound = b.madd(2.0, 3.0, fast).unwrap();
    out_by_name(&mut b, sound);
    b.finish().unwrap()
}

pub fn folds() -> SynthDef {
    let mut b = SynthDefBuilder::new("folds", vec![]).unwrap();
    let sine =
        |b: &mut SynthDefBuilder, freq: f64| by_name(b, "SinOsc", "ar", &[("freq", freq.into())]);
    let s0 = sine(&mut b, 440.0);
    let times_one = b.binary_op("*", s0, 1.0).unwrap();
    let s1 = sine(&mut b, 441.0);
    let zero_minus = b.binary_op("-", 0.0, s1).unwrap();
    let mul_minus_one = by_name(
        &mut b,
        "SinOsc",
        "ar",
        &[("freq", 442.0.into()), ("mul", (-1.0).into())],
    );
    let s3 = sine(&mut b, 443.0);
    let over_minus_one = b.binary_op("/", s3, -1.0).unwrap();
    let s4 = sine(&mut b, 444.0);
    let plus_zero = b.binary_op("+", s4, 0.0).unwrap();
    let s5 = sine(&mut b, 445.0);
    let times_minus_one = b.binary_op("*", s5, -1.0).unwrap();
    let channels = vec![
        times_one,
        zero_minus,
        mul_minus_one,
        over_minus_one,
        plus_zero,
        times_minus_one,
    ];
    out_by_name(&mut b, channels.into());
    b.finish().unwrap()
}
