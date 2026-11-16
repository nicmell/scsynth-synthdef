//! The refusals of `SynthDefBuilder::send`. Each is a graph that sclang
//! 3.14.1 throws on, or writes without its `Out`, in
//! `generator/fixtures.scd`.

use scsynth_synthdef::{ArgValue, BuildError, SynthDefBuilder, UGenInput};

/// The selector that a `BadMethod` names, or none for another error.
fn bad(error: &BuildError) -> Option<&str> {
    match error {
        BuildError::BadMethod { selector, .. } => Some(selector),
        _ => None,
    }
}

fn builder() -> SynthDefBuilder {
    SynthDefBuilder::new("refused", vec![]).unwrap()
}

/// `\name.kr(value)`.
fn control(builder: &mut SynthDefBuilder, name: &str, value: f64) -> UGenInput {
    let args = [("name", name.into()), ("values", value.into())];
    builder.ugen("NamedControl", "kr", &args).unwrap()
}

/// `Rand(0, 1)`.
fn rand(builder: &mut SynthDefBuilder) -> UGenInput {
    builder.ugen("Rand", "new", &[]).unwrap()
}

/// `Dseq([1, 2], inf)`.
fn dseq(builder: &mut SynthDefBuilder) -> UGenInput {
    let list = UGenInput::Array(vec![1.0.into(), 2.0.into()]);
    let args = [("list", list.into()), ("repeats", f64::INFINITY.into())];
    builder.ugen("Dseq", "new", &args).unwrap()
}

#[test]
fn a_number_does_not_understand_range() {
    // refused_range_number.
    let mut builder = builder();
    let args = [("lo", ArgValue::from(0.0)), ("hi", 1.0.into())];
    let error = builder.send(0.5, "range", &args).unwrap_err();
    assert_eq!(bad(&error), Some("range"), "{error}");
}

#[test]
fn fold_and_wrap_refuse_a_receiver_at_demand_rate() {
    // refused_fold_demand and refused_wrap_demand.
    for selector in ["fold", "wrap"] {
        let mut builder = builder();
        let sequence = dseq(&mut builder);
        let args = [("lo", ArgValue::from(0.0)), ("hi", 1.0.into())];
        let error = builder.send(sequence, selector, &args).unwrap_err();
        assert_eq!(bad(&error), Some(selector), "{error}");
    }
}

#[test]
fn lincurve_refuses_an_output_curve_on_a_receiver_at_scalar_rate() {
    // refused_lincurve_scalar.
    let mut builder = builder();
    let value = rand(&mut builder);
    let curve = control(&mut builder, "c", 1.0);
    let error = builder
        .send(value, "lincurve", &[("curve", curve.into())])
        .unwrap_err();
    assert_eq!(bad(&error), Some("lincurve"), "{error}");
}

#[test]
fn curvelin_refuses_an_output_curve_on_a_receiver_at_demand_rate() {
    // refused_curvelin_demand.
    let mut builder = builder();
    let sequence = dseq(&mut builder);
    let curve = control(&mut builder, "c", 1.0);
    let error = builder
        .send(sequence, "curvelin", &[("curve", curve.into())])
        .unwrap_err();
    assert_eq!(bad(&error), Some("curvelin"), "{error}");
}

#[test]
fn bilin_refuses_a_receiver_at_scalar_rate() {
    // refused_bilin_scalar.
    let mut builder = builder();
    let value = rand(&mut builder);
    let names = [
        "inCenter",
        "inMin",
        "inMax",
        "outCenter",
        "outMin",
        "outMax",
    ];
    let args: Vec<(&str, ArgValue)> = names
        .into_iter()
        .zip([0.5, 0.0, 1.0, 0.2, 0.0, 1.0])
        .map(|(name, value)| (name, value.into()))
        .collect();
    let error = builder.send(value, "bilin", &args).unwrap_err();
    assert_eq!(bad(&error), Some("bilin"), "{error}");
}

#[test]
fn clip_of_a_number_refuses_an_output_as_a_bound() {
    // refused_clip_number_ugen.
    let mut builder = builder();
    let hi = control(&mut builder, "hi", 1.0);
    let args = [("lo", ArgValue::from(0.0)), ("hi", hi.into())];
    let error = builder.send(0.5, "clip", &args).unwrap_err();
    assert_eq!(bad(&error), Some("clip"), "{error}");
}

#[test]
fn linlin_of_a_number_refuses_an_output_in_its_test() {
    // refused_linlin_number_test.
    let mut builder = builder();
    let low = control(&mut builder, "lo", 0.0);
    let args = [
        ("inMin", ArgValue::from(low)),
        ("inMax", 1.0.into()),
        ("outMin", 2.0.into()),
        ("outMax", 4.0.into()),
    ];
    let error = builder.send(0.5, "linlin", &args).unwrap_err();
    assert_eq!(bad(&error), Some("linlin"), "{error}");
}

#[test]
fn send_refuses_a_selector_that_builds_no_graph() {
    let mut builder = builder();
    let sound = builder.ugen("SinOsc", "kr", &[]).unwrap();
    for selector in ["madd", "scope", "nothing"] {
        let error = builder.send(sound.clone(), selector, &[]).unwrap_err();
        assert_eq!(bad(&error), Some(selector), "{error}");
    }
}
