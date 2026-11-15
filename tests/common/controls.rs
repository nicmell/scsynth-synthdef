//! The graphs of the golden files of 0.8, the controls inside the graph.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, SynthDef, SynthDefBuilder, UGenInput};

use super::pseudo::{def, input, out, sine};
use super::{arg, array, by_name, flat};

/// The definitions of a golden file of 0.8.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "controlugens" => [names(), control_ugens(), lag_controls(), vec![variant()]].concat(),
        _ => panic!("no golden graph of 0.8 is named {name}"),
    }
}

/// `class.selector(values)` of a control class, such as `Control.kr`.
fn control(b: &mut SynthDefBuilder, class: &str, selector: &str, values: UGenInput) -> UGenInput {
    by_name(b, class, selector, &[("values", input(values))])
}

/// `LagControl.selector(values, lags)`.
fn lag_control(
    b: &mut SynthDefBuilder,
    selector: &str,
    values: UGenInput,
    lags: UGenInput,
) -> UGenInput {
    by_name(
        b,
        "LagControl",
        selector,
        &[("values", input(values)), ("lags", input(lags))],
    )
}

/// `Out.ar(0, SinOsc.ar(freqs))`.
fn sines_out(b: &mut SynthDefBuilder, freqs: UGenInput) {
    let sines = sine(b, "ar", freqs);
    out(b, "ar", sines);
}

fn names() -> Vec<SynthDef> {
    vec![
        def("controlugens_names", vec![], |b| {
            b.control_names(&["a", "b"]);
            let c = control(b, "Control", "kr", array(&[1.0, 2.0]));
            sines_out(b, c);
        }),
        def("controlugens_names_short", vec![], |b| {
            b.control_names(&["a"]);
            let c = control(b, "Control", "kr", array(&[1.0, 2.0]));
            sines_out(b, c);
        }),
        def("controlugens_names_long", vec![], |b| {
            b.control_names(&["a", "b"]);
            let c = control(b, "Control", "kr", array(&[1.0]));
            sines_out(b, c);
        }),
        def("controlugens_names_ir", vec![], |b| {
            b.control_names(&["a", "b"]);
            let c = control(b, "Control", "ir", array(&[1.0, 2.0]));
            sines_out(b, c);
        }),
        def("controlugens_trig_names", vec![], |b| {
            b.control_names(&["t"]);
            let c = control(b, "TrigControl", "kr", 1.0.into());
            sines_out(b, c);
        }),
        def("controlugens_audio_names", vec![], |b| {
            b.control_names(&["in"]);
            let c = control(b, "AudioControl", "ar", 0.0.into());
            sines_out(b, c);
        }),
        def(
            "controlugens_after_arg",
            vec![Arg::new("freq", 440.0)],
            |b| {
                let freq = arg(b, "freq");
                let sound = sine(b, "ar", freq);
                b.control_names(&["amp"]);
                let amp = control(b, "Control", "kr", 0.1.into());
                let product = b.binary_op("*", sound, amp).unwrap();
                out(b, "ar", product);
            },
        ),
        def("controlugens_names_trig", vec![], |b| {
            b.control_names(&["a", "b"]);
            let c = control(b, "TrigControl", "kr", array(&[1.0, 2.0]));
            sines_out(b, c);
        }),
    ]
}

fn control_ugens() -> Vec<SynthDef> {
    vec![
        def("controlugens_unnamed", vec![], |b| {
            let c = control(b, "Control", "kr", array(&[1.0, 2.0]));
            sines_out(b, c);
        }),
        def("controlugens_nested", vec![], |b| {
            let values = UGenInput::Array(vec![array(&[1.0, 2.0]), 3.0.into()]);
            let c = control(b, "Control", "kr", values);
            sines_out(b, UGenInput::Array(flat(c)));
        }),
        def("controlugens_trig_ir", vec![], |b| {
            let c = control(b, "TrigControl", "ir", 1.0.into());
            sines_out(b, c);
        }),
    ]
}

/// The variant needs the default that `Control.kr` gives the last name.
fn variant() -> SynthDef {
    def("controlugens_variant", vec![], |b| {
        b.control_names(&["a", "b"]);
        let c = control(b, "Control", "kr", array(&[1.0, 2.0]));
        sines_out(b, c);
        b.variant("x", &[("b", vec![5.0])]).unwrap();
    })
}

fn lag_controls() -> Vec<SynthDef> {
    let seventeen: Vec<f64> = (1..=17).map(f64::from).collect();
    vec![
        def("lagcontrol_two", vec![], |b| {
            let c = lag_control(b, "kr", array(&[1.0, 2.0]), array(&[0.1, 0.2]));
            sines_out(b, c);
        }),
        def("lagcontrol_number", vec![], |b| {
            let c = lag_control(b, "kr", array(&[1.0, 2.0]), 0.1.into());
            sines_out(b, c);
        }),
        def("lagcontrol_seventeen", vec![], |b| {
            let c = lag_control(b, "kr", array(&seventeen), 0.1.into());
            out(b, "kr", c);
        }),
        def("lagcontrol_nested", vec![], |b| {
            let values = UGenInput::Array(vec![array(&[1.0, 2.0]), 3.0.into()]);
            let c = lag_control(b, "kr", values, array(&[0.1, 0.2]));
            sines_out(b, UGenInput::Array(flat(c)));
        }),
        def("lagcontrol_arg_lag", vec![Arg::new("lag", 0.1)], |b| {
            let lag = arg(b, "lag");
            let lags = UGenInput::Array(vec![lag, 0.2.into()]);
            let c = lag_control(b, "kr", array(&[1.0, 2.0]), lags);
            sines_out(b, c);
        }),
        def("lagcontrol_names", vec![], |b| {
            b.control_names(&["a", "b"]);
            let c = lag_control(b, "kr", array(&[1.0, 2.0]), array(&[0.1, 0.2]));
            sines_out(b, c);
        }),
        def("lagcontrol_ar", vec![], |b| {
            let c = lag_control(b, "ar", 1.0.into(), 0.1.into());
            sines_out(b, c);
        }),
        def("lagcontrol_ar_two", vec![], |b| {
            let c = lag_control(b, "ar", array(&[1.0, 2.0]), array(&[0.1, 0.2]));
            sines_out(b, c);
        }),
    ]
}
