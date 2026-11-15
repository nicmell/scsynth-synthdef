//! What the golden files of the envelopes do not show: their refusals, both
//! arrays of an `Env`, and that a call that fails adds no UGen.

use scsynth_synthdef::{ArgValue, BuildError, Curve, SynthDefBuilder, UGenInput};

fn input(value: impl Into<UGenInput>) -> ArgValue {
    ArgValue::Input(value.into())
}

fn numbers(values: &[f64]) -> UGenInput {
    UGenInput::Array(values.iter().map(|&value| value.into()).collect())
}

fn is_bad_env(result: &Result<impl std::fmt::Debug, BuildError>, method: &str) -> bool {
    matches!(result, Err(BuildError::BadEnv { method: found, .. }) if found == method)
}

#[test]
fn an_env_refuses_what_sclang_throws_on() {
    let mut b = SynthDefBuilder::new("refused", vec![]).unwrap();
    // sclang: "Primitive '_ArrayExtendWrap' failed".
    let empty = b.env(
        "new",
        &[
            ("levels", input(numbers(&[]))),
            ("times", input(numbers(&[]))),
        ],
    );
    assert!(is_bad_env(&empty, "Env.new"), "{empty:?}");
    let sizes = b.env(
        "step",
        &[
            ("levels", input(numbers(&[1.0, 2.0]))),
            ("times", input(numbers(&[1.0]))),
        ],
    );
    assert!(is_bad_env(&sizes, "Env.step"), "{sizes:?}");
    // cutoff takes the number of its shape at once.
    let cutoff = b.env("cutoff", &[("curve", "foo".into())]);
    assert!(is_bad_env(&cutoff, "Env"), "{cutoff:?}");
    // Env.new keeps the name, and the conversion for EnvGen throws.
    let unknown = b.env("new", &[("curve", "foo".into())]).unwrap();
    let converted = b.ugen("EnvGen", "kr", &[("envelope", unknown.into())]);
    assert!(is_bad_env(&converted, "Env"), "{converted:?}");
    let method = b.env("plot", &[]);
    assert!(
        matches!(method, Err(BuildError::UnknownMethod { .. })),
        "{method:?}"
    );
}

#[test]
fn an_env_goes_only_where_convert_env_takes_it() {
    let mut b = SynthDefBuilder::new("misplaced", vec![]).unwrap();
    let perc = b.env("perc", &[]).unwrap();
    let sine = b.ugen("SinOsc", "ar", &[("freq", perc.into())]);
    assert!(
        matches!(sine, Err(BuildError::WrongKind { .. })),
        "{sine:?}"
    );
}

#[test]
fn as_array_drops_the_outer_array_of_one_channel_and_the_interpolation_keeps_it() {
    let mut b = SynthDefBuilder::new("arrays", vec![]).unwrap();
    let perc = b.env("perc", &[]).unwrap();
    let flat = perc.as_array().unwrap();
    assert_eq!(
        perc.as_multichannel_array().unwrap(),
        UGenInput::Array(vec![flat.clone()])
    );
    let UGenInput::Array(values) = &flat else {
        panic!("{flat:?} is not an array");
    };
    assert_eq!(values.len(), 12);
    let interpolation = b.env_array_for_interpolation(&perc).unwrap();
    let UGenInput::Array(channels) = &interpolation else {
        panic!("{interpolation:?} is not an array");
    };
    assert_eq!(channels.len(), 1);
    assert_eq!(perc.delay(0.5).release_node, None);
    let adsr = b.env("adsr", &[]).unwrap();
    assert_eq!(adsr.delay(0.5).release_node, Some(3.0));
}

#[test]
fn an_ienv_gen_that_fails_takes_back_the_sum_of_the_times() {
    let mut b = SynthDefBuilder::new("undo", vec![]).unwrap();
    let t1 = b
        .ugen(
            "NamedControl",
            "kr",
            &[("name", "t1".into()), ("values", input(1.0))],
        )
        .unwrap();
    let t2 = b
        .ugen(
            "NamedControl",
            "kr",
            &[("name", "t2".into()), ("values", input(2.0))],
        )
        .unwrap();
    let env = b
        .env(
            "new",
            &[
                ("levels", input(numbers(&[0.0, 1.0, 0.0]))),
                ("times", input(UGenInput::Array(vec![t1.clone(), t2]))),
                ("curve", ArgValue::Curve(Curve::from("foo"))),
            ],
        )
        .unwrap();
    // The sum of the times adds t1 + t2, then the shape fails.
    let failed = b.ugen(
        "IEnvGen",
        "kr",
        &[("envelope", env.into()), ("index", input(0.5))],
    );
    assert!(is_bad_env(&failed, "Env"), "{failed:?}");
    b.ugen(
        "Out",
        "kr",
        &[("bus", input(0.0)), ("channelsArray", input(t1))],
    )
    .unwrap();
    let def = b.finish().unwrap();
    assert!(
        def.children.iter().all(|ugen| ugen.name != "BinaryOpUGen"),
        "{:?}",
        def.children
    );
}

#[test]
fn var_lag_refuses_what_sclang_throws_on() {
    let mut b = SynthDefBuilder::new("var_lag", vec![]).unwrap();
    let x = b
        .ugen(
            "NamedControl",
            "kr",
            &[("name", "x".into()), ("values", input(1.0))],
        )
        .unwrap();
    // sclang: "Env shape not defined".
    let warp = b.ugen(
        "VarLag",
        "kr",
        &[
            ("in", input(x.clone())),
            ("time", input(0.2)),
            ("warp", "foo".into()),
        ],
    );
    assert!(is_bad_env(&warp, "VarLag"), "{warp:?}");
    // A real VarLag at audio rate takes the rule of Filter.
    let rate = b.ugen(
        "VarLag",
        "ar",
        &[
            ("in", input(x)),
            ("time", input(0.2)),
            ("warp", "lin".into()),
        ],
    );
    assert!(
        matches!(rate, Err(BuildError::CheckInputs { .. })),
        "{rate:?}"
    );
}

#[test]
fn env_gate_refuses_a_shape_that_sclang_does_not_know() {
    let mut b = SynthDefBuilder::new("env_gate", vec![]).unwrap();
    let gate = b.ugen("EnvGate", "new", &[("curve", "foo".into())]);
    assert!(is_bad_env(&gate, "Env"), "{gate:?}");
}
