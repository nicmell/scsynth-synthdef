//! The golden graphs, built again with `SynthDefBuilder::ugen`, by class name:
//! the crate writes the bytes that sclang 3.14.1 wrote.

mod common;

use common::{PERC, arg, element, golden};
use scsynth_synthdef::{Arg, ArgRate, ArgValue, SynthDef, SynthDefBuilder, UGenInput};

fn ugen(
    builder: &mut SynthDefBuilder,
    class: &str,
    selector: &str,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    builder
        .ugen(class, selector, args)
        .unwrap_or_else(|error| panic!("{class}.{selector}: {error}"))
}

fn numbers(numbers: &[f64]) -> ArgValue {
    numbers
        .iter()
        .copied()
        .map(UGenInput::from)
        .collect::<Vec<_>>()
        .into()
}

fn assert_golden(def: &SynthDef, name: &str) {
    assert_eq!(def.as_bytes().unwrap(), golden(name), "{name}");
}

#[test]
fn sine() {
    let mut b = SynthDefBuilder::new(
        "sine",
        vec![
            Arg::new("out", 0.0),
            Arg::new("freq", 440.0),
            Arg::new("amp", 0.1),
        ],
    )
    .unwrap();
    let (out, freq, amp) = (arg(&b, "out"), arg(&b, "freq"), arg(&b, "amp"));
    let sin = ugen(&mut b, "SinOsc", "ar", &[("freq", freq.into())]);
    let product = b.binary_op("*", sin, amp).unwrap();
    ugen(
        &mut b,
        "Out",
        "ar",
        &[("bus", out.into()), ("channelsArray", product.into())],
    );
    assert_golden(&b.finish().unwrap(), "sine");
}

#[test]
fn rates() {
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
    let mut sig = ugen(&mut b, "SinOsc", "ar", &[("freq", freq.into())]);
    for factor in factors {
        sig = b.binary_op("*", sig, factor).unwrap();
    }
    ugen(
        &mut b,
        "Out",
        "ar",
        &[("bus", 0.0.into()), ("channelsArray", sig.into())],
    );
    assert_golden(&b.finish().unwrap(), "rates");
}

#[test]
fn multi_out() {
    let mut b = SynthDefBuilder::new("multi_out", vec![Arg::new("pos", 0.0)]).unwrap();
    let pos = arg(&b, "pos");
    let input = ugen(
        &mut b,
        "In",
        "ar",
        &[("bus", 2.0.into()), ("numChannels", 2.0.into())],
    );
    let pan = ugen(
        &mut b,
        "Pan2",
        "ar",
        &[("in", element(&input, 0).into()), ("pos", pos.into())],
    );
    ugen(
        &mut b,
        "Out",
        "ar",
        &[("bus", 0.0.into()), ("channelsArray", pan.into())],
    );
    ugen(
        &mut b,
        "Out",
        "ar",
        &[
            ("bus", 2.0.into()),
            ("channelsArray", element(&input, 1).into()),
        ],
    );
    assert_golden(&b.finish().unwrap(), "multi_out");
}

#[test]
fn sort() {
    let mut b = SynthDefBuilder::new("sort", vec![]).unwrap();
    let sine = ugen(&mut b, "SinOsc", "ar", &[("freq", 220.0.into())]);
    let saw = ugen(&mut b, "LFSaw", "ar", &[("freq", 1.0.into())]);
    let low = ugen(
        &mut b,
        "LPF",
        "ar",
        &[("in", sine.clone().into()), ("freq", 800.0.into())],
    );
    let high = ugen(
        &mut b,
        "HPF",
        "ar",
        &[("in", sine.into()), ("freq", 400.0.into())],
    );
    let resonant = ugen(
        &mut b,
        "RLPF",
        "ar",
        &[("in", saw.into()), ("freq", 600.0.into())],
    );
    let difference = b.binary_op("-", low, high).unwrap();
    ugen(
        &mut b,
        "Out",
        "ar",
        &[
            ("bus", 0.0.into()),
            ("channelsArray", vec![difference, resonant].into()),
        ],
    );
    assert_golden(&b.finish().unwrap(), "sort");
}

#[test]
fn fft() {
    let mut b = SynthDefBuilder::new("fft", vec![]).unwrap();
    let input = ugen(&mut b, "In", "ar", &[("bus", 0.0.into())]);
    let chain = ugen(
        &mut b,
        "FFT",
        "new",
        &[("buffer", 0.0.into()), ("in", input.into())],
    );
    let x = ugen(&mut b, "SinOsc", "ar", &[("freq", 2.0.into())]);
    let chain = ugen(
        &mut b,
        "PV_MagAbove",
        "new",
        &[("buffer", chain.into()), ("threshold", 0.5.into())],
    );
    let ifft = ugen(&mut b, "IFFT", "new", &[("buffer", chain.into())]);
    let product = b.binary_op("*", ifft, x).unwrap();
    ugen(
        &mut b,
        "Out",
        "ar",
        &[("bus", 0.0.into()), ("channelsArray", product.into())],
    );
    assert_golden(&b.finish().unwrap(), "fft");
}

#[test]
fn demand() {
    let mut b = SynthDefBuilder::new("demand", vec![]).unwrap();
    let trig = ugen(&mut b, "Impulse", "kr", &[("freq", 4.0.into())]);
    let seq = ugen(
        &mut b,
        "Dseq",
        "new",
        &[
            ("list", numbers(&[60.0, 62.0, 64.0])),
            ("repeats", f64::INFINITY.into()),
        ],
    );
    let demand = ugen(
        &mut b,
        "Demand",
        "kr",
        &[
            ("trig", trig.into()),
            ("reset", 0.0.into()),
            ("demandUGens", seq.into()),
        ],
    );
    ugen(
        &mut b,
        "Out",
        "kr",
        &[("bus", 0.0.into()), ("channelsArray", demand.into())],
    );
    assert_golden(&b.finish().unwrap(), "demand");
}

#[test]
fn envgen() {
    let mut b = SynthDefBuilder::new("envgen", vec![Arg::new("gate", 1.0)]).unwrap();
    let gate = arg(&b, "gate");
    let sin = ugen(&mut b, "SinOsc", "ar", &[]);
    let envelope = ugen(
        &mut b,
        "EnvGen",
        "kr",
        &[
            ("envelope", numbers(&PERC)),
            ("gate", gate.into()),
            ("doneAction", 2.0.into()),
        ],
    );
    let product = b.binary_op("*", sin, envelope).unwrap();
    ugen(
        &mut b,
        "Out",
        "ar",
        &[("bus", 0.0.into()), ("channelsArray", product.into())],
    );
    assert_golden(&b.finish().unwrap(), "envgen");
}

#[test]
fn sendreply() {
    let mut b = SynthDefBuilder::new("sendreply", vec![Arg::new("freq", 440.0)]).unwrap();
    let freq = arg(&b, "freq");
    let trig = ugen(&mut b, "Impulse", "kr", &[("freq", 1.0.into())]);
    ugen(
        &mut b,
        "SendReply",
        "kr",
        &[
            ("trig", trig.into()),
            ("cmdName", "/tick".into()),
            ("values", vec![freq, 7.0.into()].into()),
            ("replyID", 3.0.into()),
        ],
    );
    assert_golden(&b.finish().unwrap(), "sendreply");
}
