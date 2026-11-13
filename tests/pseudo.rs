//! What the golden files of the pseudo-UGens do not show: their refusals,
//! and that a call that fails adds no UGen.

use scsynth_synthdef::{Arg, ArgValue, BuildError, SynthDefBuilder, UGenInput};

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
