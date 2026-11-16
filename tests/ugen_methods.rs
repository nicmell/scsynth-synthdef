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

/// The class that a `CheckInputs` names, or none for another error.
fn checked(error: &BuildError) -> Option<&str> {
    match error {
        BuildError::CheckInputs { class, .. } => Some(class),
        _ => None,
    }
}

/// `SinOsc` at the rate of `selector`.
fn sine(builder: &mut SynthDefBuilder, selector: &str, freq: f64) -> UGenInput {
    builder
        .ugen("SinOsc", selector, &[("freq", freq.into())])
        .unwrap()
}

#[test]
fn blend_refuses_a_receiver_or_a_that_at_demand_rate() {
    // refused_blend_demand and refused_blend_demand_that.
    let mut builder = builder();
    let sequence = dseq(&mut builder);
    let error = builder
        .send(sequence.clone(), "blend", &[("that", 1.0.into())])
        .unwrap_err();
    assert_eq!(bad(&error), Some("blend"), "{error}");
    let sound = sine(&mut builder, "kr", 1.0);
    let error = builder
        .send(sound, "blend", &[("that", sequence.into())])
        .unwrap_err();
    assert_eq!(bad(&error), Some("blend"), "{error}");
}

#[test]
fn blend_takes_xfade2_only_for_two_inputs_at_audio_rate() {
    // refused_blend_control_that and refused_blend_audio_that.
    let mut builder = builder();
    let audio = sine(&mut builder, "ar", 440.0);
    let control = sine(&mut builder, "kr", 2.0);
    let pairs = [(audio.clone(), control.clone()), (control, audio)];
    for (this, that) in pairs {
        let error = builder
            .send(this, "blend", &[("that", that.into())])
            .unwrap_err();
        assert_eq!(checked(&error), Some("XFade2"), "{error}");
    }
}

#[test]
fn blend_and_sanitize_refuse_a_receiver_at_scalar_rate() {
    // refused_blend_scalar and refused_sanitize_scalar.
    let mut builder = builder();
    let value = rand(&mut builder);
    let that = rand(&mut builder);
    let error = builder
        .send(value.clone(), "blend", &[("that", that.into())])
        .unwrap_err();
    assert_eq!(bad(&error), Some("blend"), "{error}");
    let error = builder.send(value, "sanitize", &[]).unwrap_err();
    assert_eq!(bad(&error), Some("sanitize"), "{error}");
}

#[test]
fn sanitize_refuses_a_receiver_at_demand_rate() {
    // refused_sanitize_demand.
    let mut builder = builder();
    let sequence = dseq(&mut builder);
    let error = builder.send(sequence, "sanitize", &[]).unwrap_err();
    assert_eq!(bad(&error), Some("sanitize"), "{error}");
}

#[test]
fn varlag_refuses_a_receiver_at_scalar_rate() {
    // refused_varlag_scalar: the Changed.kr of VarLag runs an HPZ1.
    let mut builder = builder();
    let value = rand(&mut builder);
    let error = builder
        .send(value, "varlag", &[("time", 0.2.into())])
        .unwrap_err();
    assert_eq!(checked(&error), Some("HPZ1"), "{error}");
}

#[test]
fn a_number_gives_booleans_that_no_ugen_takes() {
    // refused_ispositive_number and refused_lazy_equals_numbers.
    let mut builder = builder();
    let error = builder.send(0.5, "isPositive", &[]).unwrap_err();
    assert_eq!(bad(&error), Some("isPositive"), "{error}");
    let error = builder
        .send(0.5, "|==|", &[("that", 0.5.into())])
        .unwrap_err();
    assert_eq!(bad(&error), Some("|==|"), "{error}");
}

#[test]
fn an_array_does_not_understand_snap() {
    // refused_snap_array.
    let mut builder = builder();
    let sound = sine(&mut builder, "kr", 1.0);
    let error = builder
        .send(UGenInput::Array(vec![sound]), "snap", &[])
        .unwrap_err();
    assert_eq!(bad(&error), Some("snap"), "{error}");
}
