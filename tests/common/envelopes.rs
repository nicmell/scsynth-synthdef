//! The graphs of the golden files of 0.9, the envelopes.
//! `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{ArgValue, Curve, Env, SynthDef, SynthDefBuilder, UGenInput};

use super::controls::kr;
use super::pseudo::{def, input, out, sine};
use super::{arg, array};

/// The definitions of a golden file of 0.9.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "env" => [constructors(), points(), curves(), outputs()].concat(),
        "ienv" => ienv(),
        "varlag" => varlag(),
        "envgate" => envgate(),
        _ => panic!("no golden graph of 0.9 is named {name}"),
    }
}

/// `Env.selector(args)`.
fn env(b: &mut SynthDefBuilder, selector: &str, args: &[(&str, ArgValue)]) -> Env {
    b.env(selector, args)
        .unwrap_or_else(|error| panic!("Env.{selector}: {error}"))
}

/// `Env(levels, times, curve)`.
fn env_new(b: &mut SynthDefBuilder, levels: UGenInput, times: UGenInput, curve: Curve) -> Env {
    env(
        b,
        "new",
        &[
            ("levels", input(levels)),
            ("times", input(times)),
            ("curve", ArgValue::Curve(curve)),
        ],
    )
}

fn shapes(names: &[&str]) -> Curve {
    Curve::Array(names.iter().map(|&name| Curve::from(name)).collect())
}

/// `EnvGen.selector(env)` with these more arguments.
fn env_gen(
    b: &mut SynthDefBuilder,
    selector: &str,
    env: Env,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![("envelope", ArgValue::Env(env))];
    all.extend(args.iter().cloned());
    b.ugen("EnvGen", selector, &all)
        .unwrap_or_else(|error| panic!("EnvGen.{selector}: {error}"))
}

/// `Out.kr(0, EnvGen.kr(env))`.
fn env_out(b: &mut SynthDefBuilder, env: Env) {
    let sound = env_gen(b, "kr", env, &[]);
    out(b, "kr", sound);
}

/// A definition of `Out.kr(0, EnvGen.kr(Env.selector(args)))`.
fn of(name: &str, selector: &'static str, args: Vec<(&'static str, ArgValue)>) -> SynthDef {
    def(name, vec![], move |b| {
        let made = env(b, selector, &args);
        env_out(b, made);
    })
}

fn constructors() -> Vec<SynthDef> {
    let curve = |name: &str| ArgValue::Curve(Curve::from(name));
    vec![
        def("env_new", vec![], |b| {
            let made = env_new(
                b,
                array(&[0.0, 1.0, 0.0]),
                array(&[1.0, 1.0]),
                Curve::from("lin"),
            );
            env_out(b, made);
        }),
        def("env_wrap_times", vec![], |b| {
            let made = env_new(
                b,
                array(&[0.0, 1.0, 0.5, 0.0]),
                array(&[1.0]),
                Curve::from("lin"),
            );
            env_out(b, made);
        }),
        of("env_default", "new", vec![]),
        of("env_perc", "perc", vec![]),
        of(
            "env_perc_args",
            "perc",
            vec![
                ("attackTime", input(0.1)),
                ("releaseTime", input(2.0)),
                ("level", input(0.5)),
                ("curve", curve("exp")),
            ],
        ),
        of("env_adsr", "adsr", vec![]),
        of("env_asr", "asr", vec![]),
        of("env_linen", "linen", vec![]),
        of("env_triangle", "triangle", vec![("dur", input(2.0))]),
        of("env_sine", "sine", vec![]),
        of("env_cutoff", "cutoff", vec![]),
        of(
            "env_cutoff_exp",
            "cutoff",
            vec![
                ("releaseTime", input(0.1)),
                ("level", input(1.0)),
                ("curve", curve("exp")),
            ],
        ),
        of("env_dadsr", "dadsr", vec![]),
        of(
            "env_step",
            "step",
            vec![
                ("levels", input(array(&[1.0, 2.0, 3.0]))),
                ("times", input(array(&[1.0, 1.0, 1.0]))),
            ],
        ),
        of(
            "env_new_clear",
            "newClear",
            vec![("numSegments", input(3.0))],
        ),
        of(
            "env_new_clear_two",
            "newClear",
            vec![("numSegments", input(2.0)), ("numChannels", input(2.0))],
        ),
        of(
            "env_new_clear_none",
            "newClear",
            vec![("numSegments", input(0.0))],
        ),
        of(
            "env_new_clear_none_two",
            "newClear",
            vec![("numSegments", input(0.0)), ("numChannels", input(2.0))],
        ),
    ]
}

fn points() -> Vec<SynthDef> {
    vec![
        of(
            "env_xyc",
            "xyc",
            vec![(
                "xyc",
                input(UGenInput::Array(vec![
                    array(&[2.0, 0.5]),
                    array(&[0.0, 1.0, 2.0]),
                    array(&[3.0, 0.0]),
                ])),
            )],
        ),
        of(
            "env_pairs",
            "pairs",
            vec![
                (
                    "pairs",
                    input(UGenInput::Array(vec![
                        array(&[0.0, 1.0]),
                        array(&[2.0, 0.5]),
                        array(&[3.0, 0.0]),
                    ])),
                ),
                ("curve", ArgValue::Curve(Curve::from("exp"))),
            ],
        ),
    ]
}

fn curves() -> Vec<SynthDef> {
    vec![
        def("env_shapes", vec![], |b| {
            let made = env_new(
                b,
                array(&[0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]),
                array(&[1.0; 8]),
                shapes(&["step", "hold", "lin", "exp", "sin", "wel", "sqr", "cub"]),
            );
            env_out(b, made);
        }),
        def("env_shape_names", vec![], |b| {
            let made = env_new(
                b,
                array(&[0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]),
                array(&[1.0; 6]),
                shapes(&["linear", "exponential", "sine", "welch", "squared", "cubed"]),
            );
            env_out(b, made);
        }),
        def("env_curves", vec![], |b| {
            let curves = Curve::Array(vec![Curve::from("exp"), Curve::from(3.0)]);
            let made = env_new(b, array(&[0.0, 1.0, 0.0]), array(&[1.0, 1.0]), curves);
            env_out(b, made);
        }),
        def("env_curve_ugen", vec![], |b| {
            let c = kr(b, "c", -4.0);
            let made = env_new(b, array(&[0.0, 1.0]), array(&[1.0]), Curve::from(c));
            env_out(b, made);
        }),
        def("env_multi", vec![], |b| {
            let levels = UGenInput::Array(vec![0.0.into(), array(&[1.0, 2.0]), 0.0.into()]);
            let made = env_new(b, levels, array(&[1.0, 1.0]), Curve::from("lin"));
            env_out(b, made);
        }),
        def("env_multi_curve", vec![], |b| {
            let curves = Curve::Array(vec![shapes(&["lin", "exp"]), Curve::from("sin")]);
            let made = env_new(b, array(&[0.0, 1.0, 0.0]), array(&[1.0, 1.0]), curves);
            env_out(b, made);
        }),
        def("env_nodes", vec![], |b| {
            let made = env(
                b,
                "new",
                &[
                    ("levels", input(array(&[0.0, 1.0, 0.5, 0.0]))),
                    ("times", input(array(&[1.0, 1.0, 1.0]))),
                    ("releaseNode", input(2.0)),
                    ("loopNode", input(1.0)),
                ],
            );
            env_out(b, made);
        }),
    ]
}

fn outputs() -> Vec<SynthDef> {
    vec![
        def("env_ugen_level", vec![], |b| {
            let l = kr(b, "l", 1.0);
            let levels = UGenInput::Array(vec![0.0.into(), l, 0.0.into()]);
            let made = env_new(b, levels, array(&[1.0, 1.0]), Curve::from("lin"));
            env_out(b, made);
        }),
        def("env_adsr_ugens", vec![], |b| {
            let p = kr(b, "p", 2.0);
            let s = kr(b, "s", 0.5);
            let made = env(
                b,
                "adsr",
                &[("peakLevel", input(p)), ("sustainLevel", input(s))],
            );
            let gate = kr(b, "gate", 1.0);
            let sound = env_gen(b, "kr", made, &[("gate", input(gate))]);
            out(b, "kr", sound);
        }),
        def("env_adsr_bias", vec![], |b| {
            let bias = kr(b, "b", 0.1);
            let made = env(b, "adsr", &[("bias", input(bias))]);
            env_out(b, made);
        }),
        def("env_circle", vec![], |b| {
            let made = env(
                b,
                "circle",
                &[
                    ("levels", input(array(&[0.0, 1.0, 0.0]))),
                    ("times", input(array(&[1.0, 1.0, 2.0]))),
                    ("curve", ArgValue::Curve(Curve::from("exp"))),
                ],
            );
            env_out(b, made);
        }),
        def("env_delay", vec![], |b| {
            let made = env(b, "adsr", &[]).delay(0.5);
            env_out(b, made);
        }),
        def("env_one_level", vec![], |b| {
            let made = env_new(
                b,
                array(&[1.0]),
                UGenInput::Array(Vec::new()),
                Curve::from("lin"),
            );
            env_out(b, made);
        }),
        def("env_kr", vec![], |b| {
            let made = env(b, "perc", &[]);
            let gate = kr(b, "gate", 1.0);
            let sound = env_gen(
                b,
                "kr",
                made,
                &[("gate", input(gate)), ("doneAction", input(2.0))],
            );
            out(b, "kr", sound);
        }),
        def("env_ar", vec![], |b| {
            let made = env(b, "sine", &[]);
            let sound = env_gen(b, "ar", made, &[]);
            out(b, "ar", sound);
        }),
    ]
}

/// `IEnvGen.selector(env, index)` with these more arguments.
fn ienv_gen(
    b: &mut SynthDefBuilder,
    selector: &str,
    env: Env,
    index: UGenInput,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    let mut all = vec![("envelope", ArgValue::Env(env)), ("index", input(index))];
    all.extend(args.iter().cloned());
    b.ugen("IEnvGen", selector, &all)
        .unwrap_or_else(|error| panic!("IEnvGen.{selector}: {error}"))
}

/// `Out.kr(0, IEnvGen.kr(env, index))`.
fn ienv_out(b: &mut SynthDefBuilder, env: Env, index: UGenInput) {
    let sound = ienv_gen(b, "kr", env, index, &[]);
    out(b, "kr", sound);
}

fn ienv() -> Vec<SynthDef> {
    vec![
        def("ienv_new", vec![], |b| {
            let made = env_new(
                b,
                array(&[0.0, 1.0, 0.0]),
                array(&[1.0, 2.0]),
                Curve::from("lin"),
            );
            let index = kr(b, "i", 0.0);
            ienv_out(b, made, index);
        }),
        def("ienv_offset", vec![], |b| {
            let made = env(
                b,
                "new",
                &[
                    ("levels", input(array(&[0.0, 1.0, 0.0]))),
                    ("times", input(array(&[1.0, 2.0]))),
                    ("curve", ArgValue::Curve(Curve::from("exp"))),
                    ("offset", input(0.5)),
                ],
            );
            ienv_out(b, made, 0.5.into());
        }),
        def("ienv_ugen_times", vec![], |b| {
            let times = UGenInput::Array(vec![kr(b, "t1", 1.0), kr(b, "t2", 2.0)]);
            let made = env_new(b, array(&[0.0, 1.0, 0.0]), times, Curve::from("lin"));
            ienv_out(b, made, 0.5.into());
        }),
        def("ienv_ugen_time_numbers", vec![], |b| {
            let times = UGenInput::Array(vec![kr(b, "t", 1.0), 1.0.into(), 1.0.into()]);
            let made = env_new(b, array(&[0.0, 1.0, 0.0, 1.0]), times, Curve::from("lin"));
            ienv_out(b, made, 0.5.into());
        }),
        def("ienv_multi", vec![], |b| {
            let levels = UGenInput::Array(vec![0.0.into(), array(&[1.0, 2.0]), 0.0.into()]);
            let made = env_new(b, levels, array(&[1.0, 2.0]), Curve::from("lin"));
            ienv_out(b, made, 0.5.into());
        }),
        def("ienv_multi_index", vec![], |b| {
            let levels = UGenInput::Array(vec![0.0.into(), array(&[1.0, 2.0]), 0.0.into()]);
            let made = env_new(b, levels, array(&[1.0, 2.0]), Curve::from("lin"));
            ienv_out(b, made, array(&[0.25, 0.5, 0.75]));
        }),
        def("ienv_madd", vec![], |b| {
            let made = env(b, "perc", &[]);
            let index = b
                .ugen(
                    "NamedControl",
                    "ar",
                    &[("name", "i".into()), ("values", input(0.0))],
                )
                .unwrap();
            let sound = ienv_gen(
                b,
                "ar",
                made,
                index,
                &[("mul", input(0.5)), ("add", input(0.1))],
            );
            out(b, "ar", sound);
        }),
        def("ienv_xyc", vec![], |b| {
            let points = UGenInput::Array(vec![
                array(&[1.0, 1.0]),
                array(&[2.0, 0.5]),
                array(&[4.0, 0.0]),
            ]);
            let made = env(b, "xyc", &[("xyc", input(points))]);
            ienv_out(b, made, 1.5.into());
        }),
    ]
}

/// `VarLag.selector(args)`.
fn var_lag(b: &mut SynthDefBuilder, selector: &str, args: &[(&str, ArgValue)]) -> UGenInput {
    b.ugen("VarLag", selector, args)
        .unwrap_or_else(|error| panic!("VarLag.{selector}: {error}"))
}

/// `\\name.ar(value)`.
fn ar(b: &mut SynthDefBuilder, name: &str, value: f64) -> UGenInput {
    b.ugen(
        "NamedControl",
        "ar",
        &[("name", name.into()), ("values", input(value))],
    )
    .unwrap()
}

/// A definition of `Out.kr(0, VarLag.kr(\\x.kr(1), 0.2, ...))`, with these
/// more arguments.
fn lag_of(name: &str, more: Vec<(&'static str, ArgValue)>) -> SynthDef {
    def(name, vec![], move |b| {
        let x = kr(b, "x", 1.0);
        let mut args = vec![("in", input(x)), ("time", input(0.2))];
        args.extend(more);
        let lagged = var_lag(b, "kr", &args);
        out(b, "kr", lagged);
    })
}

fn varlag() -> Vec<SynthDef> {
    let warp = |curve: Curve| ("warp", ArgValue::Curve(curve));
    vec![
        lag_of("varlag_kr", vec![]),
        def("varlag_ar", vec![], |b| {
            let x = ar(b, "x", 1.0);
            let lagged = var_lag(b, "ar", &[("in", input(x)), ("time", input(0.2))]);
            out(b, "ar", lagged);
        }),
        lag_of("varlag_lin", vec![warp(Curve::from("lin"))]),
        lag_of("varlag_one", vec![warp(Curve::from(1.0))]),
        def("varlag_exp", vec![], |b| {
            let x = ar(b, "x", 1.0);
            let t = kr(b, "t", 0.2);
            let lagged = var_lag(
                b,
                "ar",
                &[
                    ("in", input(x)),
                    ("time", input(t)),
                    ("curvature", input(2.0)),
                    warp(Curve::from("exp")),
                ],
            );
            out(b, "ar", lagged);
        }),
        lag_of(
            "varlag_three",
            vec![("curvature", input(-2.0)), warp(Curve::from(3.0))],
        ),
        lag_of("varlag_start", vec![("start", input(0.5))]),
        def("varlag_ugen_warp", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let w = kr(b, "w", 5.0);
            let lagged = var_lag(
                b,
                "kr",
                &[("in", input(x)), ("time", input(0.2)), warp(Curve::from(w))],
            );
            out(b, "kr", lagged);
        }),
        def("varlag_array", vec![], |b| {
            let inputs = UGenInput::Array(vec![kr(b, "x", 1.0), kr(b, "y", 2.0)]);
            let lagged = var_lag(b, "kr", &[("in", input(inputs)), ("time", input(0.2))]);
            out(b, "kr", lagged);
        }),
        lag_of(
            "varlag_array_warp",
            vec![warp(Curve::Array(vec![Curve::from(5.0), Curve::from(1.0)]))],
        ),
        lag_of(
            "varlag_madd",
            vec![("mul", input(2.0)), ("add", input(1.0))],
        ),
        def("varlag_scalar", vec![], |b| {
            let lagged = var_lag(
                b,
                "kr",
                &[
                    ("in", input(3.0)),
                    ("time", input(0.2)),
                    ("mul", input(2.0)),
                    ("add", input(1.0)),
                ],
            );
            out(b, "kr", lagged);
        }),
        def("varlag_time_zero", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let lagged = var_lag(b, "kr", &[("in", input(x)), ("time", input(0.0))]);
            out(b, "kr", lagged);
        }),
    ]
}

/// `EnvGate(args)`.
fn env_gate(b: &mut SynthDefBuilder, args: &[(&str, ArgValue)]) -> UGenInput {
    b.ugen("EnvGate", "new", args)
        .unwrap_or_else(|error| panic!("EnvGate.new: {error}"))
}

/// `Out.ar(0, sound * gate)`.
fn gated_out(b: &mut SynthDefBuilder, sound: UGenInput, gate: UGenInput) {
    let product = b.binary_op("*", sound, gate).unwrap();
    out(b, "ar", product);
}

fn envgate() -> Vec<SynthDef> {
    vec![
        def("envgate_new", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let gate = env_gate(b, &[]);
            gated_out(b, sound, gate);
        }),
        def("envgate_args", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let g = kr(b, "g", 1.0);
            let gate = env_gate(
                b,
                &[
                    ("i_level", input(0.0)),
                    ("gate", input(g)),
                    ("fadeTime", input(0.5)),
                    ("doneAction", input(0.0)),
                    ("curve", ArgValue::Curve(Curve::from("lin"))),
                ],
            );
            gated_out(b, sound, gate);
        }),
        def("envgate_twice", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let gate = env_gate(b, &[]);
            let first = b.binary_op("*", sound, gate).unwrap();
            let sound = sine(b, "ar", 2.0);
            let gate = env_gate(b, &[]);
            let second = b.binary_op("*", sound, gate).unwrap();
            let sum = b.binary_op("+", first, second).unwrap();
            out(b, "ar", sum);
        }),
        def("envgate_level", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let level = kr(b, "l", 0.5);
            let gate = env_gate(b, &[("i_level", input(level))]);
            gated_out(b, sound, gate);
        }),
        def(
            "envgate_markup",
            vec![scsynth_synthdef::Arg::new("freq", 440.0)],
            |b| {
                let freq = arg(b, "freq");
                let lag = var_lag(b, "kr", &[("in", input(freq)), ("time", input(0.2))]);
                let osc = sine(b, "ar", lag);
                let gate = env_gate(b, &[]);
                gated_out(b, osc, gate);
            },
        ),
    ]
}
