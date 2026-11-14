//! What the golden files of the pseudo-UGens do not show: their refusals,
//! and that a call that fails adds no UGen.

use scsynth_synthdef::{Arg, ArgValue, BuildError, InputSpec, Rate, SynthDefBuilder, UGenInput};

fn builder() -> SynthDefBuilder {
    SynthDefBuilder::new("pseudo", vec![Arg::new("x", 1.0)]).unwrap()
}

fn input(value: impl Into<UGenInput>) -> ArgValue {
    ArgValue::Input(value.into())
}

#[test]
fn a_pseudo_ugen_refuses_what_sclang_refuses() {
    let mut b = builder();
    let x = b.arguments()[0].clone();
    let missing = b.ugen("PMOsc", "ar", &[("modfreq", input(2.0))]);
    assert!(
        matches!(&missing, Err(BuildError::MissingArgument { class, argument, .. })
            if class == "PMOsc" && argument == "carfreq"),
        "{missing:?}"
    );
    let count = b.ugen("Silent", "ar", &[("numChannels", input(0.0))]);
    assert!(
        matches!(count, Err(BuildError::BadCount { .. })),
        "{count:?}"
    );
    let empty = b.ugen(
        "TChoose",
        "kr",
        &[("trig", input(x.clone())), ("array", input(Vec::new()))],
    );
    assert!(
        matches!(empty, Err(BuildError::EmptyInput { .. })),
        "{empty:?}"
    );
    let method = b.ugen("SoundIn", "kr", &[]);
    assert!(
        matches!(&method, Err(BuildError::UnknownMethod { class, selector, .. })
            if class == "SoundIn" && selector == "kr"),
        "{method:?}"
    );
    let unknown = b.ugen("LinLin", "kr", &[("freq", input(1.0))]);
    assert!(
        matches!(unknown, Err(BuildError::UnknownArgument { .. })),
        "{unknown:?}"
    );
}

#[test]
fn sound_in_refuses_an_output_in_an_array_of_buses() {
    // sclang builds a `+ 1` for its test of consecutive buses, which only
    // optimizeGraph removes (fact 17 of the plan of 0.3).
    let mut b = builder();
    let x = b.arguments()[0].clone();
    let buses = UGenInput::Array(vec![x, 1.0.into()]);
    let refused = b.ugen("SoundIn", "ar", &[("bus", input(buses))]);
    assert!(
        matches!(refused, Err(BuildError::WrongKind { .. })),
        "{refused:?}"
    );
    let empty = b.ugen("SoundIn", "ar", &[("bus", input(Vec::new()))]);
    assert!(
        matches!(empty, Err(BuildError::EmptyInput { .. })),
        "{empty:?}"
    );
}

#[test]
fn a_pseudo_ugen_that_fails_adds_no_ugen() {
    let mut b = builder();
    let x = b.arguments()[0].clone();
    // NumOutputBuses comes before the test of the buses.
    let buses = UGenInput::Array(vec![x]);
    assert!(b.ugen("SoundIn", "ar", &[("bus", input(buses))]).is_err());
    let def = b.finish().unwrap();
    let names: Vec<&str> = def.children.iter().map(|ugen| ugen.name.as_str()).collect();
    assert_eq!(names, ["Control"]);
}

#[test]
fn a_lag_of_a_control_and_a_number_breaks_check_inputs_as_in_sclang() {
    // The control makes the array run at control rate, so nothing passes
    // through, and Lag.kr(2) breaks checkSameRateAsFirstInput.
    let mut b = builder();
    let x = b.arguments()[0].clone();
    let pair = UGenInput::Array(vec![x.clone(), 2.0.into()]);
    let mixed = b.ugen("Lag", "kr", &[("in", input(pair)), ("lagTime", input(0.1))]);
    assert!(
        matches!(&mixed, Err(BuildError::CheckInputs { class, .. }) if class == "Lag"),
        "{mixed:?}"
    );
    let audio = b.ugen("Lag", "ar", &[("in", input(x))]);
    assert!(
        matches!(&audio, Err(BuildError::CheckInputs { class, .. }) if class == "Lag"),
        "{audio:?}"
    );
    let def = b.finish().unwrap();
    let names: Vec<&str> = def.children.iter().map(|ugen| ugen.name.as_str()).collect();
    assert_eq!(names, ["Control"]);
}

#[test]
fn a_call_that_fails_leaves_the_max_local_bufs_as_it_was() {
    let mut b = builder();
    let nan = |frames: f64| input(UGenInput::Array(vec![frames.into(), f64::NAN.into()]));
    // The first call adds the MaxLocalBufs, then fails: neither stays.
    let first = b.ugen("LocalBuf", "new", &[("numFrames", nan(256.0))]);
    assert!(
        matches!(first, Err(BuildError::NotANumber { .. })),
        "{first:?}"
    );
    b.ugen("LocalBuf", "new", &[("numFrames", input(1024.0))])
        .unwrap();
    // A later call counts one more buffer, then fails: the count stays 1.
    let later = b.ugen("LocalBuf", "new", &[("numFrames", nan(512.0))]);
    assert!(
        matches!(later, Err(BuildError::NotANumber { .. })),
        "{later:?}"
    );
    let def = b.finish().unwrap();
    let names: Vec<&str> = def.children.iter().map(|ugen| ugen.name.as_str()).collect();
    assert_eq!(names, ["Control", "MaxLocalBufs", "LocalBuf"]);
    let InputSpec::Constant(count) = def.children[1].inputs[0] else {
        panic!("the count of MaxLocalBufs is a constant");
    };
    assert_eq!(def.constants[count as usize].to_bits(), 1.0_f32.to_bits());
}

#[test]
fn local_in_and_poll_refuse_what_sclang_refuses() {
    let mut b = builder();
    let x = b.arguments()[0].clone();
    let none = b.ugen("LocalIn", "ar", &[("numChannels", input(0.0))]);
    assert!(
        matches!(&none, Err(BuildError::BadCount { class, .. }) if class == "LocalIn"),
        "{none:?}"
    );
    let empty = b.ugen(
        "LocalIn",
        "ar",
        &[("numChannels", input(2.0)), ("default", input(Vec::new()))],
    );
    assert!(
        matches!(empty, Err(BuildError::EmptyInput { .. })),
        "{empty:?}"
    );
    // The rule of Poll is checkSameRateAsFirstInput: its trigger.
    let trig = b.ugen("Impulse", "ar", &[("freq", input(1.0))]).unwrap();
    let audio = b.ugen("Poll", "kr", &[("trig", input(trig)), ("in", input(x))]);
    assert!(
        matches!(&audio, Err(BuildError::CheckInputs { class, .. }) if class == "Poll"),
        "{audio:?}"
    );
}

#[test]
fn klank_refuses_an_empty_part_beside_a_full_one() {
    // flop would read past the end of the empty part.
    let mut b = builder();
    let x = b.arguments()[0].clone();
    let freqs = UGenInput::Array(vec![800.0.into(), 1071.0.into()]);
    let spec = UGenInput::Array(vec![freqs, UGenInput::Array(Vec::new())]);
    let empty = b.ugen(
        "Klank",
        "ar",
        &[("specificationsArrayRef", input(spec)), ("input", input(x))],
    );
    assert!(
        matches!(&empty, Err(BuildError::EmptyInput { ugen, .. }) if ugen == "Klank"),
        "{empty:?}"
    );
}

#[test]
fn mix_gives_0_for_no_input_and_refuses_a_mix_at_demand_rate() {
    let mut b = builder();
    let nothing = b.ugen("Mix", "new", &[("array", input(Vec::new()))]);
    assert_eq!(nothing, Ok(UGenInput::Number(0.0)));
    let list = input(vec![1.0.into(), 2.0.into()]);
    let seq = b.ugen("Dseq", "new", &[("list", list)]).unwrap();
    let refused = b.ugen("Mix", "ar", &[("array", input(vec![seq.clone(), seq]))]);
    assert!(
        matches!(&refused, Err(BuildError::UnsupportedRate { ugen, rate, .. })
            if ugen == "Mix.ar" && *rate == Rate::Demand),
        "{refused:?}"
    );
}

#[test]
fn the_second_wave_refuses_what_sclang_refuses() {
    let mut b = builder();
    let x = b.arguments()[0].clone();
    let wrap = b.ugen(
        "SelectXFocus",
        "kr",
        &[
            ("which", input(x.clone())),
            ("array", input(vec![x.clone(), 2.0.into()])),
            ("wrap", input(1.0)),
        ],
    );
    assert!(
        matches!(&wrap, Err(BuildError::WrongKind { argument, .. }) if argument == "wrap"),
        "{wrap:?}"
    );
    let empty = b.ugen("NumChannels", "ar", &[("input", input(Vec::new()))]);
    assert!(
        matches!(empty, Err(BuildError::EmptyInput { .. })),
        "{empty:?}"
    );
    let bufsize = b.ugen(
        "UnpackFFT",
        "new",
        &[("chain", input(x.clone())), ("bufsize", input(x.clone()))],
    );
    assert!(
        matches!(&bufsize, Err(BuildError::WrongKind { argument, .. }) if argument == "bufsize"),
        "{bufsize:?}"
    );
    let bufnum = b.ugen(
        "MultiTap",
        "ar",
        &[
            ("timesArray", input(vec![0.1.into()])),
            ("levelsArray", input(vec![1.0.into()])),
            ("in", input(x)),
        ],
    );
    assert!(
        matches!(&bufnum, Err(BuildError::MissingArgument { argument, .. }) if argument == "bufnum"),
        "{bufnum:?}"
    );
}

#[test]
fn splay_az_refuses_to_blend_a_spread_at_scalar_or_demand_rate() {
    // At scalar rate, sclang builds the LinXFade2 of `blend` outside the
    // graph (fact 10 of the plan of 0.3).
    let mut b = SynthDefBuilder::new("pseudo", vec![Arg::new("i_spread", 1.0)]).unwrap();
    let spread = b.arguments()[0].clone();
    let pair = input(vec![1.0.into(), 2.0.into()]);
    let scalar = b.ugen(
        "SplayAz",
        "kr",
        &[("inArray", pair.clone()), ("spread", input(spread))],
    );
    assert!(
        matches!(&scalar, Err(BuildError::UnsupportedRate { rate, .. }) if *rate == Rate::Scalar),
        "{scalar:?}"
    );
    let list = input(vec![1.0.into(), 2.0.into()]);
    let seq = b.ugen("Dseq", "new", &[("list", list)]).unwrap();
    let demand = b.ugen(
        "SplayAz",
        "kr",
        &[("inArray", pair), ("spread", input(seq))],
    );
    assert!(
        matches!(&demand, Err(BuildError::UnsupportedRate { rate, .. }) if *rate == Rate::Demand),
        "{demand:?}"
    );
}

#[test]
fn a_ugen_refuses_a_boolean() {
    let mut b = builder();
    let refused = b.ugen("SinOsc", "ar", &[("freq", ArgValue::Bool(true))]);
    assert!(
        matches!(&refused, Err(BuildError::WrongKind { class, argument, .. })
            if class == "SinOsc" && argument == "freq"),
        "{refused:?}"
    );
}

#[test]
fn select_x_ignores_wrap() {
    let graph = |wrap: Option<f64>| {
        let mut b = builder();
        let x = b.arguments()[0].clone();
        let mut args = vec![("which", input(0.5)), ("array", input(vec![x, 2.0.into()]))];
        if let Some(wrap) = wrap {
            args.push(("wrap", input(wrap)));
        }
        let mixed = b.ugen("SelectX", "kr", &args).unwrap();
        b.ugen(
            "Out",
            "kr",
            &[("bus", input(0.0)), ("channelsArray", input(mixed))],
        )
        .unwrap();
        b.finish().unwrap().as_bytes().unwrap()
    };
    assert_eq!(graph(None), graph(Some(0.0)));
}
