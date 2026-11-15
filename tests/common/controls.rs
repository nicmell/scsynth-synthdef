//! The graphs of the golden files of 0.8, the controls inside the graph.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, BuildError, SynthDef, SynthDefBuilder, UGenInput, catalog};

use super::pseudo::{def, input, out, sine};
use super::{arg, array, by_name, flat};

/// The definitions of a golden file of 0.8.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "controlugens" => [names(), control_ugens(), lag_controls(), vec![variant()]].concat(),
        "defwrap" => [wraps(), prepends()].concat(),
        "specs" => vec![specs()],
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

/// `SinOsc.ar * x`, at the default frequency.
fn sine_times(b: &mut SynthDefBuilder, x: UGenInput) -> Result<UGenInput, BuildError> {
    let sound = sine(b, "ar", 440.0);
    b.binary_op("*", sound, x)
}

fn wraps() -> Vec<SynthDef> {
    vec![
        def("defwrap_arg", vec![Arg::new("freq", 440.0)], |b| {
            let freq = arg(b, "freq");
            let sound = b
                .wrap(vec![Arg::new("amp", 0.1)], |b, inputs| {
                    let sound = sine(b, "ar", freq);
                    b.binary_op("*", sound, inputs[0].clone())
                })
                .unwrap();
            out(b, "ar", sound);
        }),
        def("defwrap_prepend", vec![], |b| {
            let sig = sine(b, "ar", 440.0);
            let sound = b
                .wrap(vec![Arg::new("amp", 0.1)], |b, inputs| {
                    b.binary_op("*", sig, inputs[0].clone())
                })
                .unwrap();
            out(b, "ar", sound);
        }),
        def("defwrap_rates", vec![], |b| {
            let args = vec![Arg::new("amp", 0.1).lag(0.2), Arg::new("i_x", 2.0)];
            let sound = b
                .wrap(args, |b, inputs| {
                    let sound = sine(b, "ar", inputs[1].clone());
                    b.binary_op("*", sound, inputs[0].clone())
                })
                .unwrap();
            out(b, "ar", sound);
        }),
        def("defwrap_same_name", vec![Arg::new("amp", 0.1)], |b| {
            let amp = arg(b, "amp");
            let sound = b
                .wrap(vec![Arg::new("amp", 0.2)], |b, inputs| {
                    sine_times(b, inputs[0].clone())
                })
                .unwrap();
            let sound = b.binary_op("*", sound, amp).unwrap();
            out(b, "ar", sound);
        }),
        def("defwrap_twice", vec![], |b| {
            let a = b
                .wrap(vec![Arg::new("a", 1.0)], |b, inputs| {
                    Ok(sine(b, "ar", inputs[0].clone()))
                })
                .unwrap();
            let c = b
                .wrap(vec![Arg::new("b", 2.0)], |b, inputs| {
                    Ok(sine(b, "ar", inputs[0].clone()))
                })
                .unwrap();
            let sum = b.binary_op("+", a, c).unwrap();
            out(b, "ar", sum);
        }),
        def("defwrap_names", vec![], |b| {
            b.control_names(&["a", "b"]);
            b.wrap(vec![Arg::new("c", 7.0)], |_, inputs| Ok(inputs[0].clone()))
                .unwrap();
            let c = control(b, "Control", "kr", array(&[1.0, 2.0]));
            sines_out(b, c);
            b.variant("x", &[("b", vec![5.0])]).unwrap();
        }),
        def("defwrap_variant", vec![Arg::new("amp", 0.1)], |b| {
            let sound = b
                .wrap(vec![Arg::new("amp", 0.2)], |b, inputs| {
                    sine_times(b, inputs[0].clone())
                })
                .unwrap();
            out(b, "ar", sound);
            b.variant("x", &[("amp", vec![0.5])]).unwrap();
        }),
    ]
}

/// The graph function `{ |bus, freq = 440| Out.ar(bus, SinOsc.ar(freq)) }`
/// with the `prependArgs` `[bus]`: only `freq` becomes a control.
fn prepend(name: &str, bus: UGenInput) -> SynthDef {
    def(name, vec![Arg::new("freq", 440.0)], |b| {
        let freq = arg(b, "freq");
        let sound = sine(b, "ar", freq);
        by_name(
            b,
            "Out",
            "ar",
            &[("bus", input(bus)), ("channelsArray", input(sound))],
        );
    })
}

fn prepends() -> Vec<SynthDef> {
    vec![
        prepend("prepend", 0.0.into()),
        prepend("prepend_array", array(&[0.0, 1.0])),
    ]
}

/// An argument without a default per spec, under `metadata: (specs: ())`:
/// each takes the default of the spec of its name.
fn specs() -> SynthDef {
    let args = catalog::specs()
        .iter()
        .map(|spec| Arg::new(spec.name, spec.default))
        .collect();
    def("specs_args", args, |b| {
        let values = UGenInput::Array(b.arguments().to_vec());
        out(b, "kr", values);
    })
}
