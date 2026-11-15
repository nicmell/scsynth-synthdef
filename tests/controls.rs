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

#[test]
fn a_wrap_without_arguments_hides_the_names_of_the_outer_function() {
    // controlNames is nil around the wrapped function, so its Control gives
    // a no default, and sclang writes the variant broken.
    let mut b = SynthDefBuilder::new("hidden", vec![]).unwrap();
    b.control_names(&["a"]);
    let c = b
        .wrap(vec![], |b, _| {
            b.ugen("Control", "kr", &[("values", input(1.0))])
        })
        .unwrap();
    sine_out(&mut b, c);
    let variant = b.variant("x", &[("a", vec![2.0])]);
    assert!(
        matches!(variant, Err(BuildError::VariantSize { size: 0, .. })),
        "{variant:?}"
    );
}

#[test]
fn a_wrap_that_fails_leaves_the_build_as_it_was() {
    let mut b = SynthDefBuilder::new("undo", vec![]).unwrap();
    let failed = b.wrap(vec![Arg::new("a", 1.0)], |b, inputs| {
        b.ugen("SinOsc", "ar", &[("freq", input(inputs[0].clone()))])?;
        b.ugen("NoSuchClass", "ar", &[])
    });
    assert!(
        matches!(failed, Err(BuildError::UnknownClass(_))),
        "{failed:?}"
    );
    let c = b
        .wrap(vec![Arg::new("b", 2.0)], |_, inputs| Ok(inputs[0].clone()))
        .unwrap();
    sine_out(&mut b, c);
    let def = b.finish().unwrap();
    assert_eq!(def.controls, [2.0]);
    let names: Vec<&str> = def
        .control_names
        .iter()
        .map(|name| name.name.as_str())
        .collect();
    assert_eq!(names, ["b"]);
    assert_eq!(def.children.len(), 3);
}

#[test]
fn a_wrap_refuses_its_arguments_as_new_does() {
    let mut b = SynthDefBuilder::new("twice", vec![]).unwrap();
    let twice = b.wrap(vec![Arg::new("a", 1.0), Arg::new("a", 2.0)], |_, _| Ok(()));
    assert!(
        matches!(twice, Err(BuildError::DuplicateArgument(_))),
        "{twice:?}"
    );
}

/// `NamedControl.selector` with these arguments by name.
fn named(
    b: &mut SynthDefBuilder,
    selector: &str,
    args: &[(&str, ArgValue)],
) -> Result<UGenInput, BuildError> {
    b.ugen("NamedControl", selector, args)
}

fn kr(name: &str, values: impl Into<UGenInput>) -> Vec<(&str, ArgValue)> {
    vec![("name", ArgValue::from(name)), ("values", input(values))]
}

fn kr_lag(
    name: &str,
    values: impl Into<UGenInput>,
    lags: impl Into<UGenInput>,
) -> Vec<(&str, ArgValue)> {
    let mut args = kr(name, values);
    args.push(("lags", input(lags)));
    args
}

fn is_bad_control(result: &Result<UGenInput, BuildError>) -> bool {
    matches!(result, Err(BuildError::BadControl { class, .. }) if class == "NamedControl")
}

#[test]
fn a_named_control_refuses_a_second_use_that_disagrees() {
    let mut b = SynthDefBuilder::new("named", vec![]).unwrap();
    named(&mut b, "kr", &kr("freq", 440.0)).unwrap();
    let values = named(&mut b, "kr", &kr("freq", 220.0));
    assert!(is_bad_control(&values), "{values:?}");
    let rate = named(&mut b, "ar", &kr("freq", 440.0));
    assert!(is_bad_control(&rate), "{rate:?}");
    named(&mut b, "kr", &kr_lag("a", 1.0, 0.1)).unwrap();
    let lags = named(&mut b, "kr", &kr_lag("a", 1.0, 0.2));
    assert!(is_bad_control(&lags), "{lags:?}");
    let lt = named(&mut b, "kr", &kr("lt", 0.2)).unwrap();
    let output = named(&mut b, "kr", &kr_lag("a", 1.0, lt));
    assert!(is_bad_control(&output), "{output:?}");
    // A spec without values gives its default first.
    let spec = |name: &str| {
        vec![
            ("name", ArgValue::from("s")),
            ("spec", ArgValue::from(name)),
        ]
    };
    named(&mut b, "kr", &spec("freq")).unwrap();
    let other = named(&mut b, "kr", &spec("amp"));
    assert!(is_bad_control(&other), "{other:?}");
}

#[test]
fn a_named_control_refuses_a_0_among_the_lags_that_reach_a_ugen() {
    let mut b = SynthDefBuilder::new("zeros", vec![]).unwrap();
    let fixed = named(
        &mut b,
        "kr",
        &kr_lag("x", numbers(&[1.0, 2.0]), numbers(&[0.1, 0.0])),
    );
    assert!(is_bad_control(&fixed), "{fixed:?}");
    let l = named(&mut b, "kr", &kr("l", 0.2)).unwrap();
    let lags = UGenInput::Array(vec![0.0.into(), l]);
    let lagged = named(&mut b, "kr", &kr_lag("y", numbers(&[1.0, 2.0]), lags));
    assert!(is_bad_control(&lagged), "{lagged:?}");
    // The failed call leaves no y in the table, so another default holds.
    let again = named(&mut b, "kr", &kr("y", 3.0));
    assert!(again.is_ok(), "{again:?}");
    // A prefix skips the LagControl, and the lags reach no UGen.
    let prefixed = named(
        &mut b,
        "kr",
        &kr_lag("x_a", numbers(&[1.0, 2.0]), numbers(&[0.1, 0.0])),
    );
    assert!(prefixed.is_ok(), "{prefixed:?}");
}

#[test]
fn a_named_control_refuses_what_sclang_throws_on() {
    let mut b = SynthDefBuilder::new("refused", vec![Arg::new("x", 1.0)]).unwrap();
    let x = b.arguments()[0].clone();
    let spec = named(
        &mut b,
        "kr",
        &[
            ("name", ArgValue::from("a")),
            ("spec", ArgValue::from("nosuchspec")),
        ],
    );
    assert!(is_bad_control(&spec), "{spec:?}");
    // sclang reads the default of the spec only for no values.
    let valued = named(
        &mut b,
        "kr",
        &[
            ("name", ArgValue::from("b")),
            ("values", input(0.5)),
            ("spec", ArgValue::from("nosuchspec")),
        ],
    );
    assert!(valued.is_ok(), "{valued:?}");
    let output = named(&mut b, "kr", &kr("c", x));
    assert!(
        matches!(output, Err(BuildError::WrongKind { .. })),
        "{output:?}"
    );
    let empty = named(&mut b, "kr", &kr("d", numbers(&[])));
    assert!(
        matches!(empty, Err(BuildError::EmptyInput { .. })),
        "{empty:?}"
    );
    let nameless = named(&mut b, "kr", &[("values", input(1.0))]);
    assert!(
        matches!(nameless, Err(BuildError::MissingArgument { .. })),
        "{nameless:?}"
    );
    let sizes = named(
        &mut b,
        "kr",
        &kr_lag("e", numbers(&[1.0, 2.0]), numbers(&[0.1, 0.2, 0.3])),
    );
    assert!(
        matches!(&sizes, Err(BuildError::BadControl { class, .. }) if class == "LagControl"),
        "{sizes:?}"
    );
}

#[test]
fn a_named_control_that_fails_leaves_no_name_behind() {
    let mut b = SynthDefBuilder::new("undo", vec![]).unwrap();
    let sizes = named(
        &mut b,
        "kr",
        &kr_lag("x", numbers(&[1.0, 2.0]), numbers(&[0.1, 0.2, 0.3])),
    );
    assert!(sizes.is_err(), "{sizes:?}");
    let x = named(&mut b, "kr", &kr("x", 5.0)).unwrap();
    sine_out(&mut b, x);
    let def = b.finish().unwrap();
    assert_eq!(def.controls, [5.0]);
    let names: Vec<&str> = def
        .control_names
        .iter()
        .map(|name| name.name.as_str())
        .collect();
    assert_eq!(names, ["x"]);
}
