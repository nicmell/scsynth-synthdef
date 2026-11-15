//! The graphs of the golden files of 0.8, the controls inside the graph.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, ArgValue, BuildError, SynthDef, SynthDefBuilder, UGenInput, catalog};

use super::pseudo::{def, input, out, sine};
use super::{arg, array, by_name, flat};

/// The definitions of a golden file of 0.8.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "controlugens" => [names(), control_ugens(), lag_controls(), vec![variant()]].concat(),
        "defwrap" => [wraps(), prepends()].concat(),
        "specs" => vec![specs()],
        "namedcontrol" => [
            named_uses(),
            named_lags(),
            named_lag_inputs(),
            named_others(),
        ]
        .concat(),
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

/// `NamedControl.selector(name, values, lags)`, as `\\name.selector(values,
/// lags)` calls it.
fn named(
    b: &mut SynthDefBuilder,
    selector: &str,
    name: &str,
    values: Option<UGenInput>,
    lags: Option<UGenInput>,
) -> UGenInput {
    let mut args = vec![("name", ArgValue::from(name))];
    args.extend(values.map(|values| ("values", input(values))));
    args.extend(lags.map(|lags| ("lags", input(lags))));
    by_name(b, "NamedControl", selector, &args)
}

/// `\\name.kr(value)`.
pub(super) fn kr(b: &mut SynthDefBuilder, name: &str, value: impl Into<UGenInput>) -> UGenInput {
    named(b, "kr", name, Some(value.into()), None)
}

/// `\\name.kr(value, lags)`.
fn kr_lag(
    b: &mut SynthDefBuilder,
    name: &str,
    value: impl Into<UGenInput>,
    lags: impl Into<UGenInput>,
) -> UGenInput {
    named(b, "kr", name, Some(value.into()), Some(lags.into()))
}

fn binary(b: &mut SynthDefBuilder, op: &str, a: UGenInput, c: UGenInput) -> UGenInput {
    b.binary_op(op, a, c).unwrap()
}

/// `Out.ar(0, SinOsc.ar(a) + SinOsc.ar(c))`.
fn two_sines_out(b: &mut SynthDefBuilder, a: UGenInput, c: UGenInput) {
    let first = sine(b, "ar", a);
    let second = sine(b, "ar", c);
    let sum = binary(b, "+", first, second);
    out(b, "ar", sum);
}

fn named_uses() -> Vec<SynthDef> {
    vec![
        def("namedcontrol_one", vec![], |b| {
            let freq = kr(b, "freq", 440.0);
            sines_out(b, freq);
        }),
        def("namedcontrol_twice", vec![], |b| {
            let freq = kr(b, "freq", 440.0);
            let first = sine(b, "ar", freq);
            let again = named(b, "kr", "freq", None, None);
            let second = sine(b, "ar", again);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_after_arg", vec![Arg::new("amp", 0.1)], |b| {
            let amp = arg(b, "amp");
            let freq = kr(b, "freq", 440.0);
            let sound = sine(b, "ar", freq);
            let product = binary(b, "*", sound, amp);
            out(b, "ar", product);
        }),
        def(
            "namedcontrol_arg_name",
            vec![Arg::new("freq", 440.0)],
            |b| {
                let freq = arg(b, "freq");
                let named = kr(b, "freq", 220.0);
                two_sines_out(b, named, freq);
            },
        ),
        def("namedcontrol_rates", vec![], |b| {
            let a = named(b, "ir", "a", Some(1.0.into()), None);
            let sound = sine(b, "ar", a);
            let c = named(b, "tr", "b", Some(2.0.into()), None);
            let sound = binary(b, "*", sound, c);
            let c = named(b, "ar", "c", Some(3.0.into()), None);
            let sound = binary(b, "+", sound, c);
            let c = kr(b, "d", 4.0);
            let sound = binary(b, "+", sound, c);
            out(b, "ar", sound);
        }),
        def("namedcontrol_prefixes", vec![], |b| {
            let a = kr(b, "i_a", 1.0);
            let sound = sine(b, "ar", a);
            let c = kr(b, "t_b", 2.0);
            let sound = binary(b, "*", sound, c);
            let c = kr(b, "a_c", 3.0);
            let sound = binary(b, "+", sound, c);
            out(b, "ar", sound);
        }),
        def("namedcontrol_prefix_rate", vec![], |b| {
            let x = named(b, "ar", "i_x", Some(1.0.into()), None);
            let mut sound = sine(b, "ar", x);
            for (selector, name) in [("ir", "t_y"), ("tr", "i_z"), ("ir", "a_w")] {
                let c = named(b, selector, name, Some(1.0.into()), None);
                sound = binary(b, "+", sound, c);
            }
            out(b, "ar", sound);
        }),
        def("namedcontrol_new", vec![], |b| {
            let new = |b: &mut SynthDefBuilder, name: &str, value: f64, rate: Option<&str>| {
                let mut args = vec![("name", ArgValue::from(name)), ("values", input(value))];
                args.extend(rate.map(|rate| ("rate", ArgValue::from(rate))));
                by_name(b, "NamedControl", "new", &args)
            };
            let a = new(b, "a", 1.0, Some("audio"));
            let mut sound = sine(b, "ar", a);
            for (name, value, rate) in [
                ("b", 2.0, None),
                ("c", 3.0, Some("foo")),
                ("d", 4.0, Some("trigger")),
                ("e", 5.0, Some("scalar")),
            ] {
                let c = new(b, name, value, rate);
                sound = binary(b, "+", sound, c);
            }
            out(b, "ar", sound);
        }),
        def("namedcontrol_array", vec![], |b| {
            let freqs = kr(b, "freqs", array(&[440.0, 660.0]));
            sines_out(b, freqs);
        }),
        def("namedcontrol_nested", vec![], |b| {
            let values = UGenInput::Array(vec![array(&[1.0, 2.0]), array(&[3.0, 4.0])]);
            let x = kr(b, "x", values);
            sines_out(b, UGenInput::Array(flat(x)));
        }),
        def("namedcontrol_no_value", vec![], |b| {
            let freq = named(b, "kr", "freq", None, None);
            sines_out(b, freq);
        }),
        def("namedcontrol_one_value", vec![], |b| {
            let x = kr(b, "x", array(&[440.0]));
            sines_out(b, x);
        }),
    ]
}

fn named_lags() -> Vec<SynthDef> {
    let seventeen: Vec<f64> = (1..=17).map(f64::from).collect();
    vec![
        def("namedcontrol_lag", vec![], |b| {
            let freq = kr_lag(b, "freq", 440.0, 0.1);
            sines_out(b, freq);
        }),
        def("namedcontrol_lag_zero", vec![], |b| {
            let freq = kr_lag(b, "freq", 440.0, 0.0);
            sines_out(b, freq);
        }),
        def("namedcontrol_lag_array", vec![], |b| {
            let freqs = kr_lag(b, "freqs", array(&[440.0, 660.0]), array(&[0.1, 0.2]));
            sines_out(b, freqs);
        }),
        def("namedcontrol_lag_one_for_two", vec![], |b| {
            let freqs = kr_lag(b, "freqs", array(&[440.0, 660.0]), 0.1);
            sines_out(b, freqs);
        }),
        def("namedcontrol_lag_seventeen", vec![], |b| {
            let x = kr_lag(b, "x", array(&seventeen), 0.1);
            out(b, "kr", x);
        }),
        def("namedcontrol_lag_twice", vec![], |b| {
            let first = kr_lag(b, "freq", 440.0, 0.1);
            let first = sine(b, "ar", first);
            let second = kr_lag(b, "freq", 440.0, 0.1);
            let second = sine(b, "ar", second);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_lag_then_none", vec![], |b| {
            let first = kr_lag(b, "freq", 440.0, 0.1);
            let first = sine(b, "ar", first);
            let second = kr(b, "freq", 440.0);
            let second = sine(b, "ar", second);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_none_then_lag", vec![], |b| {
            let first = kr(b, "freq", 440.0);
            let first = sine(b, "ar", first);
            let second = kr_lag(b, "freq", 440.0, 0.1);
            let second = sine(b, "ar", second);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_lag_ugen", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let freq = kr_lag(b, "freq", 440.0, lt);
            sines_out(b, freq);
        }),
        def("namedcontrol_lag_ugen_twice", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let first = kr_lag(b, "freq", 440.0, lt);
            let first = sine(b, "ar", first);
            let lt = kr(b, "lt", 0.2);
            let second = kr_lag(b, "freq", 440.0, lt);
            let second = sine(b, "ar", second);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_ugen_then_fixed", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let first = kr_lag(b, "a", 1.0, lt);
            let first = sine(b, "ar", first);
            let second = kr_lag(b, "a", 1.0, 0.1);
            let second = sine(b, "ar", second);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_fixed_ugen", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let args = [
                ("name", ArgValue::from("a")),
                ("values", input(1.0)),
                ("lags", input(lt)),
                ("fixedLag", ArgValue::from(true)),
            ];
            let a = by_name(b, "NamedControl", "kr", &args);
            sines_out(b, a);
        }),
    ]
}

fn named_lag_inputs() -> Vec<SynthDef> {
    vec![
        def("namedcontrol_lag_ugens", vec![], |b| {
            let lags = UGenInput::Array(vec![kr(b, "l1", 0.1), kr(b, "l2", 0.2)]);
            let x = kr_lag(b, "x", array(&[1.0, 2.0]), lags);
            sines_out(b, x);
        }),
        def("namedcontrol_lag_ugens_one_value", vec![], |b| {
            let lags = UGenInput::Array(vec![kr(b, "l1", 0.1), kr(b, "l2", 0.2)]);
            let x = kr_lag(b, "x", 1.0, lags);
            sines_out(b, x);
        }),
        def("namedcontrol_lag_ugen_two_values", vec![], |b| {
            let lag = kr(b, "l", 0.1);
            let x = kr_lag(b, "x", array(&[1.0, 2.0]), lag);
            sines_out(b, x);
        }),
        def("namedcontrol_lag_mixed", vec![], |b| {
            let lags = UGenInput::Array(vec![0.1.into(), kr(b, "l", 0.2)]);
            let x = kr_lag(b, "x", array(&[1.0, 2.0]), lags);
            sines_out(b, x);
        }),
        def("namedcontrol_prefix_lag", vec![], |b| {
            let freq = kr_lag(b, "x_freq", 440.0, 0.1);
            let sound = sine(b, "ar", freq);
            let a = kr_lag(b, "a_", 1.0, 0.1);
            let sum = binary(b, "+", sound, a);
            out(b, "ar", sum);
        }),
        def("namedcontrol_ir_lag", vec![], |b| {
            let a = named(b, "ir", "a", Some(1.0.into()), Some(0.1.into()));
            let sound = sine(b, "ar", a);
            let c = named(b, "tr", "b", Some(1.0.into()), Some(0.1.into()));
            let product = binary(b, "*", sound, c);
            out(b, "ar", product);
        }),
        def("namedcontrol_ar_lag", vec![], |b| {
            let a = named(b, "ar", "a", Some(1.0.into()), Some(0.1.into()));
            sines_out(b, a);
        }),
        def("namedcontrol_ir_lag_ugen", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let a = named(b, "ir", "a", Some(1.0.into()), Some(lt));
            sines_out(b, a);
        }),
        def("namedcontrol_tr_lag_ugen", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let x = kr_lag(b, "t_x", 1.0, lt);
            sines_out(b, x);
        }),
        def("namedcontrol_ar_lag_ugen", vec![], |b| {
            let lt = kr(b, "lt", 0.2);
            let a = named(b, "ar", "a", Some(1.0.into()), Some(lt));
            sines_out(b, a);
        }),
    ]
}

/// `\\name.kr(values, spec: spec)`.
fn kr_spec(b: &mut SynthDefBuilder, name: &str, values: Option<f64>, spec: &str) -> UGenInput {
    let mut args = vec![
        ("name", ArgValue::from(name)),
        ("spec", ArgValue::from(spec)),
    ];
    args.extend(values.map(|values| ("values", input(values))));
    by_name(b, "NamedControl", "kr", &args)
}

fn named_others() -> Vec<SynthDef> {
    vec![
        def("namedcontrol_wrap", vec![], |b| {
            let freq = kr(b, "freq", 440.0);
            let first = sine(b, "ar", freq);
            let second = b
                .wrap(vec![], |b, _| {
                    let freq = named(b, "kr", "freq", None, None);
                    Ok(sine(b, "ar", freq))
                })
                .unwrap();
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_control_names", vec![], |b| {
            b.control_names(&["a"]);
            let a = control(b, "Control", "kr", 1.0.into());
            let first = sine(b, "ar", a);
            let named = kr(b, "a", 2.0);
            let second = sine(b, "ar", named);
            let sum = binary(b, "+", first, second);
            out(b, "ar", sum);
        }),
        def("namedcontrol_spec", vec![], |b| {
            let freq = kr_spec(b, "freq", None, "freq");
            let sound = sine(b, "ar", freq);
            let amp = kr_spec(b, "amp", Some(0.5), "amp");
            let sound = binary(b, "*", sound, amp);
            let rq = kr_spec(b, "rq", None, "rq");
            let sound = binary(b, "*", sound, rq);
            let freq = kr(b, "freq", 440.0);
            let other = sine(b, "ar", freq);
            let sum = binary(b, "+", sound, other);
            out(b, "ar", sum);
        }),
        def("namedcontrol_variant", vec![Arg::new("freq", 440.0)], |b| {
            let freq = arg(b, "freq");
            let named = kr(b, "freq", 220.0);
            two_sines_out(b, freq, named);
            b.variant("x", &[("freq", vec![330.0])]).unwrap();
        }),
        def("namedcontrol_markup", vec![], |b| {
            let freq = kr_lag(b, "freq", 440.0, 0.1);
            let osc = sine(b, "ar", freq);
            let amp = kr(b, "amp", 0.1);
            let product = binary(b, "*", osc, amp);
            out(b, "ar", product);
        }),
    ]
}
