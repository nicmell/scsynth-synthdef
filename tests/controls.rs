//! What the golden files of the controls do not show: their refusals, and
//! that a call that fails leaves no parameter and no name behind.

use scsynth_synthdef::{Arg, ArgValue, BuildError, SynthDefBuilder, UGenInput};

fn input(value: impl Into<UGenInput>) -> ArgValue {
    ArgValue::Input(value.into())
}

fn numbers(values: &[f64]) -> UGenInput {
    UGenInput::Array(values.iter().map(|&value| value.into()).collect())
}

fn sine_out(b: &mut SynthDefBuilder, freq: UGenInput) {
    let sine = b.ugen("SinOsc", "ar", &[("freq", input(freq))]).unwrap();
    b.ugen(
        "Out",
        "ar",
        &[("bus", input(0.0)), ("channelsArray", input(sine))],
    )
    .unwrap();
}

#[test]
fn a_control_ugen_refuses_what_sclang_refuses() {
    let mut b = SynthDefBuilder::new("controls", vec![Arg::new("x", 1.0)]).unwrap();
    let x = b.arguments()[0].clone();
    // sclang throws "numChannels must be a nonzero positive integer".
    let empty = b.ugen("Control", "kr", &[("values", input(numbers(&[])))]);
    assert!(
        matches!(empty, Err(BuildError::EmptyInput { .. })),
        "{empty:?}"
    );
    // sclang writes the output as a parameter, and throws.
    let output = b.ugen("Control", "kr", &[("values", input(x))]);
    assert!(
        matches!(output, Err(BuildError::WrongKind { .. })),
        "{output:?}"
    );
    // sclang gives nil, and the reader of nil throws.
    let sizes = b.ugen(
        "LagControl",
        "kr",
        &[
            ("values", input(numbers(&[1.0, 2.0]))),
            ("lags", input(numbers(&[0.1]))),
        ],
    );
    assert!(
        matches!(&sizes, Err(BuildError::BadControl { class, .. }) if class == "LagControl"),
        "{sizes:?}"
    );
    // sclang adds nothing for no values.
    let none = b.ugen(
        "LagControl",
        "kr",
        &[
            ("values", input(numbers(&[]))),
            ("lags", input(numbers(&[]))),
        ],
    );
    assert!(
        matches!(none, Err(BuildError::EmptyInput { .. })),
        "{none:?}"
    );
}

#[test]
fn a_variant_takes_no_value_for_a_name_without_a_default_or_past_the_parameters() {
    let mut b = SynthDefBuilder::new("variants", vec![]).unwrap();
    b.control_names(&["a", "b"]);
    let c = b
        .ugen("Control", "kr", &[("values", input(numbers(&[1.0, 2.0])))])
        .unwrap();
    sine_out(&mut b, c);
    // sclang writes the count of the variants and no variant.
    let default = b.variant("x", &[("a", vec![3.0])]);
    assert!(
        matches!(default, Err(BuildError::VariantSize { size: 0, .. })),
        "{default:?}"
    );
    // b has the default [1, 2], but one parameter from its index: sclang
    // throws "Index out of range".
    let past = b.variant("y", &[("b", vec![5.0, 6.0])]);
    assert!(
        matches!(past, Err(BuildError::VariantSize { size: 1, .. })),
        "{past:?}"
    );
}

#[test]
fn a_name_past_the_end_of_the_parameters_is_refused() {
    let mut b = SynthDefBuilder::new("past", vec![]).unwrap();
    b.control_names(&["a", "b", "c"]);
    let c = b.ugen("Control", "kr", &[("values", input(1.0))]).unwrap();
    sine_out(&mut b, c);
    let finished = b.finish();
    assert!(
        matches!(finished, Err(BuildError::Limit(_))),
        "{finished:?}"
    );
}

#[test]
fn a_control_call_that_fails_leaves_the_parameters_and_the_names_as_they_were() {
    let mut b = SynthDefBuilder::new("undo", vec![Arg::new("x", 1.0)]).unwrap();
    let x = b.arguments()[0].clone();
    b.control_names(&["a"]);
    // The first instance adds a Control and gives a its size. The second
    // takes an output among its values, and fails.
    let values = UGenInput::Array(vec![UGenInput::Array(vec![2.0.into(), x])]);
    let failed = b.ugen("Control", "kr", &[("values", input(values))]);
    assert!(
        matches!(failed, Err(BuildError::WrongKind { .. })),
        "{failed:?}"
    );
    let c = b
        .ugen("Control", "kr", &[("values", input(numbers(&[5.0, 6.0])))])
        .unwrap();
    sine_out(&mut b, c);
    b.variant("v", &[("a", vec![7.0, 8.0])]).unwrap();
    let def = b.finish().unwrap();
    assert_eq!(def.controls, [1.0, 5.0, 6.0]);
    assert_eq!(def.variants[0].values, [1.0, 7.0, 8.0]);
}
