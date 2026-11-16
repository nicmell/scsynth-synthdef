//! The graphs of the golden files of the methods of `UGen` that build graphs,
//! built with `SynthDefBuilder::send`. `generator/fixtures.scd` holds the
//! sclang code of each.

use scsynth_synthdef::{ArgValue, Env, SynthDef, SynthDefBuilder, UGenInput};

use super::controls::kr;
use super::pseudo::{def, dseq, impulse, input, out, sine};
use super::{array, by_name, element};

/// The definitions of a golden file of 0.12.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "ugenranges" => [ranges(), other_ranges()].concat(),
        "ugenmaps" => [maps(), curves(), bilins(), clips()].concat(),
        "ugenlags" => [lags(), varlags()].concat(),
        "ugenmisc" => [misc(), checks(), more_misc()].concat(),
        "ugenreceivers" => [arrays(), numbers()].concat(),
        "envranges" => env_ranges(),
        "pollarrays" => poll_arrays(),
        _ => panic!("no golden graph is named {name}"),
    }
}

fn send(
    b: &mut SynthDefBuilder,
    receiver: impl Into<UGenInput>,
    selector: &str,
    args: &[(&str, ArgValue)],
) -> UGenInput {
    b.send(receiver, selector, args)
        .unwrap_or_else(|error| panic!("{selector}: {error}"))
}

/// `LFPulse` at the rate of `selector`, of `freq`.
fn pulse(b: &mut SynthDefBuilder, selector: &str, freq: f64) -> UGenInput {
    by_name(b, "LFPulse", selector, &[("freq", input(freq))])
}

/// `Rand(lo, hi)`.
fn rand(b: &mut SynthDefBuilder, lo: f64, hi: f64) -> UGenInput {
    by_name(b, "Rand", "new", &[("lo", input(lo)), ("hi", input(hi))])
}

/// `LFGauss.ar(0.01, width)`.
fn gauss(b: &mut SynthDefBuilder, width: impl Into<UGenInput>) -> UGenInput {
    let args = [("duration", input(0.01)), ("width", input(width))];
    by_name(b, "LFGauss", "ar", &args)
}

/// `Out.kr(0, Demand.kr(Impulse.kr(1), 0, body(Dseq([1, 2], inf))))`.
fn demand_out(
    b: &mut SynthDefBuilder,
    body: impl FnOnce(&mut SynthDefBuilder, UGenInput) -> UGenInput,
) {
    let trig = impulse(b, "kr", 1.0);
    let sequence = dseq(b, &[1.0, 2.0]);
    let value = body(b, sequence);
    let args = [
        ("trig", input(trig)),
        ("reset", input(0.0)),
        ("demandUGens", input(value)),
    ];
    let demand = by_name(b, "Demand", "kr", &args);
    out(b, "kr", demand);
}

/// The arguments `lo` and `hi`.
fn lo_hi(lo: impl Into<UGenInput>, hi: impl Into<UGenInput>) -> [(&'static str, ArgValue); 2] {
    [("lo", input(lo)), ("hi", input(hi))]
}

/// The arguments `inMin`, `inMax`, `outMin` and `outMax` of `linlin` and its
/// kin.
fn map(ranges: [f64; 4]) -> Vec<(&'static str, ArgValue)> {
    ["inMin", "inMax", "outMin", "outMax"]
        .into_iter()
        .zip(ranges)
        .map(|(name, value)| (name, input(value)))
        .collect()
}

/// `map(ranges)`, with these other arguments.
fn map_with(
    ranges: [f64; 4],
    others: &[(&'static str, ArgValue)],
) -> Vec<(&'static str, ArgValue)> {
    let mut args = map(ranges);
    args.extend_from_slice(others);
    args
}

/// The cases of `ugenranges` of `range`, from `ugenranges_bipolar` to
/// `ugenranges_demand`.
fn ranges() -> Vec<SynthDef> {
    vec![
        def("ugenranges_bipolar", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "range", &lo_hi(200.0, 800.0));
            out(b, "kr", r);
        }),
        def("ugenranges_unipolar", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let r = send(b, p, "range", &lo_hi(200.0, 800.0));
            out(b, "kr", r);
        }),
        def("ugenranges_inherited", vec![], |b| {
            let mouse = by_name(b, "MouseY", "kr", &[]);
            let r = send(b, mouse, "range", &lo_hi(2.0, 4.0));
            out(b, "kr", r);
        }),
        def("ugenranges_proxy", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let pan = by_name(b, "Pan2", "kr", &[("in", input(p))]);
            let r = send(b, element(&pan, 0), "range", &lo_hi(2.0, 4.0));
            out(b, "kr", r);
        }),
        def("ugenranges_operator", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let negated = b.unary_op("neg", p).unwrap();
            let r = send(b, negated, "range", &lo_hi(2.0, 4.0));
            out(b, "kr", r);
        }),
        def("ugenranges_defaults", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "range", &[]);
            out(b, "ar", r);
        }),
        def("ugenranges_ugen", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let lo = kr(b, "lo", 2.0);
            let hi = kr(b, "hi", 4.0);
            let r = send(b, s, "range", &lo_hi(lo, hi));
            out(b, "ar", r);
        }),
        def("ugenranges_array", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "range", &lo_hi(0.0, array(&[1.0, 2.0])));
            out(b, "kr", r);
        }),
        def("ugenranges_demand", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "range", &lo_hi(0.0, 10.0)));
        }),
    ]
}

/// The cases of `ugenranges` of the other methods, from
/// `ugenranges_exprange` to `ugenranges_demand_exprange`.
fn other_ranges() -> Vec<SynthDef> {
    let curve = |lo: f64, hi: f64, curve: f64| {
        [
            ("lo", input(lo)),
            ("hi", input(hi)),
            ("curve", input(curve)),
        ]
    };
    vec![
        def("ugenranges_exprange", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "exprange", &lo_hi(20.0, 20000.0));
            out(b, "kr", r);
        }),
        def("ugenranges_exprange_unipolar", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let r = send(b, p, "exprange", &lo_hi(20.0, 20000.0));
            out(b, "kr", r);
        }),
        def("ugenranges_curverange", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "curverange", &curve(0.0, 1.0, -4.0));
            out(b, "kr", r);
        }),
        def("ugenranges_curverange_unipolar", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let r = send(b, p, "curverange", &curve(2.0, 4.0, 2.0));
            out(b, "kr", r);
        }),
        def("ugenranges_curverange_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let c = kr(b, "c", -4.0);
            let args = [("lo", input(0.0)), ("hi", input(1.0)), ("curve", input(c))];
            let r = send(b, s, "curverange", &args);
            out(b, "kr", r);
        }),
        def("ugenranges_unipolar_method", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "unipolar", &[("mul", input(2.0))]);
            out(b, "kr", r);
        }),
        def("ugenranges_bipolar_method", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let r = send(b, p, "bipolar", &[("mul", input(2.0))]);
            out(b, "kr", r);
        }),
        def("ugenranges_bipolar_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let m = kr(b, "m", 2.0);
            let r = send(b, s, "bipolar", &[("mul", input(m))]);
            out(b, "kr", r);
        }),
        def("ugenranges_gauss", vec![], |b| {
            let g = gauss(b, 0.1);
            let r = send(b, g, "range", &lo_hi(2.0, 4.0));
            out(b, "ar", r);
        }),
        def("ugenranges_gauss_ugen", vec![], |b| {
            let w = kr(b, "w", 0.1);
            let g = gauss(b, w);
            let r = send(b, g, "range", &lo_hi(2.0, 4.0));
            out(b, "ar", r);
        }),
        def("ugenranges_gauss_exprange", vec![], |b| {
            let g = gauss(b, 0.1);
            let r = send(b, g, "exprange", &lo_hi(2.0, 4.0));
            out(b, "ar", r);
        }),
        def("ugenranges_gauss_unipolar", vec![], |b| {
            let g = gauss(b, 0.1);
            let r = send(b, g, "unipolar", &[("mul", input(2.0))]);
            out(b, "ar", r);
        }),
        def("ugenranges_demand_exprange", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "exprange", &lo_hi(2.0, 4.0)));
        }),
    ]
}

/// The cases of `ugenmaps` of `linlin`, `linexp`, `explin` and `expexp`,
/// from `ugenmaps_linlin` to `ugenmaps_expexp`.
fn maps() -> Vec<SynthDef> {
    let clip = |value: &str| [("clip", ArgValue::from(value))];
    vec![
        def("ugenmaps_linlin", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "linlin", &map([-1.0, 1.0, 200.0, 800.0]));
            out(b, "kr", r);
        }),
        def("ugenmaps_linlin_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "linlin", &map([-1.0, 1.0, 200.0, 800.0]));
            out(b, "ar", r);
        }),
        def("ugenmaps_linlin_min", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "linlin",
                &map_with([-1.0, 1.0, 200.0, 800.0], &clip("min")),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_linlin_max", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "linlin",
                &map_with([-1.0, 1.0, 200.0, 800.0], &clip("max")),
            );
            out(b, "kr", r);
        }),
        // nil in sclang: no clip, as any type but minmax, min and max.
        def("ugenmaps_linlin_none", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "linlin",
                &map_with([-1.0, 1.0, 200.0, 800.0], &clip("none")),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_linlin_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let low = kr(b, "a", -1.0);
            let high = kr(b, "b", 800.0);
            let args = [
                ("inMin", input(low)),
                ("inMax", input(1.0)),
                ("outMin", input(200.0)),
                ("outMax", input(high)),
            ];
            let r = send(b, s, "linlin", &args);
            out(b, "kr", r);
        }),
        def("ugenmaps_linexp", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "linexp", &map([-1.0, 1.0, 20.0, 20000.0]));
            out(b, "kr", r);
        }),
        def("ugenmaps_linexp_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(
                b,
                s,
                "linexp",
                &map_with([-1.0, 1.0, 20.0, 20000.0], &clip("min")),
            );
            out(b, "ar", r);
        }),
        def("ugenmaps_explin", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let r = send(b, p, "explin", &map([0.01, 1.0, 0.0, 1.0]));
            out(b, "kr", r);
        }),
        def("ugenmaps_expexp", vec![], |b| {
            let p = pulse(b, "kr", 1.0);
            let r = send(b, p, "expexp", &map([0.01, 1.0, 20.0, 20000.0]));
            out(b, "kr", r);
        }),
    ]
}

/// The cases of `ugenmaps` of `lincurve` and `curvelin`, from
/// `ugenmaps_lincurve` to `ugenmaps_curvelin_ugen`.
fn curves() -> Vec<SynthDef> {
    let curve = |value: f64| [("curve", input(value))];
    vec![
        def("ugenmaps_lincurve", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "lincurve",
                &map_with([-1.0, 1.0, 0.0, 1.0], &curve(-4.0)),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_lincurve_small", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "lincurve",
                &map_with([-1.0, 1.0, 0.0, 1.0], &curve(0.1)),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_lincurve_defaults", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "lincurve", &[]);
            out(b, "kr", r);
        }),
        def("ugenmaps_lincurve_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let c = kr(b, "c", -4.0);
            let r = send(
                b,
                s,
                "lincurve",
                &map_with([-1.0, 1.0, 0.0, 1.0], &[("curve", input(c))]),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_lincurve_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let c = kr(b, "c", -4.0);
            let r = send(
                b,
                s,
                "lincurve",
                &map_with([-1.0, 1.0, 0.0, 1.0], &[("curve", input(c))]),
            );
            out(b, "ar", r);
        }),
        def("ugenmaps_curvelin", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "curvelin",
                &map_with([-1.0, 1.0, 0.0, 1.0], &curve(-4.0)),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_curvelin_small", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                s,
                "curvelin",
                &map_with([-1.0, 1.0, 0.0, 1.0], &curve(0.1)),
            );
            out(b, "kr", r);
        }),
        def("ugenmaps_curvelin_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let c = kr(b, "c", -4.0);
            let r = send(
                b,
                s,
                "curvelin",
                &map_with([-1.0, 1.0, 0.0, 1.0], &[("curve", input(c))]),
            );
            out(b, "kr", r);
        }),
    ]
}

/// The cases of `ugenmaps` of `bilin` and `prune`, from `ugenmaps_bilin` to
/// `ugenmaps_prune`.
fn bilins() -> Vec<SynthDef> {
    vec![
        def("ugenmaps_bilin", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "bilin", &bilin(&[]));
            out(b, "kr", r);
        }),
        def("ugenmaps_bilin_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "bilin", &bilin(&[("clip", ArgValue::from("max"))]));
            out(b, "ar", r);
        }),
        def("ugenmaps_prune", vec![], |b| {
            let types = [Some("minmax"), Some("min"), Some("max"), None];
            let pruned = (1..=4)
                .zip(types)
                .map(|(freq, kind)| {
                    let s = sine(b, "kr", f64::from(freq));
                    let mut args = vec![("min", input(0.0)), ("max", input(1.0))];
                    args.extend(kind.map(|kind| ("type", ArgValue::from(kind))));
                    send(b, s, "prune", &args)
                })
                .collect();
            out(b, "kr", UGenInput::Array(pruned));
        }),
    ]
}

/// The arguments of `bilin(0, -1, 1, 0.2, 0, 1)`, with these others.
fn bilin(others: &[(&'static str, ArgValue)]) -> Vec<(&'static str, ArgValue)> {
    let names = [
        "inCenter",
        "inMin",
        "inMax",
        "outCenter",
        "outMin",
        "outMax",
    ];
    let values = [0.0, -1.0, 1.0, 0.2, 0.0, 1.0];
    let mut args: Vec<(&'static str, ArgValue)> = names
        .into_iter()
        .zip(values)
        .map(|(name, value)| (name, input(value)))
        .collect();
    args.extend_from_slice(others);
    args
}

/// The cases of `ugenmaps` of `clip`, `fold`, `wrap` and the overrides of
/// `DUGen`, from `ugenmaps_clip` to `ugenmaps_demand_lincurve_small`.
fn clips() -> Vec<SynthDef> {
    let mut defs = vec![
        def("ugenmaps_clip", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "clip", &lo_hi(0.0, 0.5));
            let r = rand(b, 0.0, 2.0);
            let second = send(b, r, "clip", &[]);
            let s = sine(b, "kr", 2.0);
            let lo = kr(b, "lo", 0.0);
            let third = send(b, s, "clip", &lo_hi(lo, 1.0));
            out(b, "kr", UGenInput::Array(vec![first, second, third]));
        }),
        def("ugenmaps_clip_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "clip", &lo_hi(-0.5, 0.5));
            out(b, "ar", r);
        }),
        def("ugenmaps_clip_demand", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "clip", &lo_hi(0.0, 1.0)));
        }),
    ];
    for selector in ["fold", "wrap"] {
        defs.push(def(&format!("ugenmaps_{selector}"), vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, selector, &lo_hi(0.0, 0.5));
            let r = rand(b, 0.0, 2.0);
            let second = if selector == "fold" {
                send(b, r, selector, &lo_hi(0.0, 1.0))
            } else {
                send(b, r, selector, &[])
            };
            let s = sine(b, "kr", 2.0);
            let third = send(b, s, selector, &[]);
            out(b, "kr", UGenInput::Array(vec![first, second, third]));
        }));
        defs.push(def(&format!("ugenmaps_{selector}_audio"), vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, selector, &lo_hi(-0.5, 0.5));
            out(b, "ar", r);
        }));
    }
    let overrides: [(&str, [f64; 4]); 4] = [
        ("linlin", [0.0, 2.0, 2.0, 4.0]),
        ("linexp", [0.0, 2.0, 2.0, 4.0]),
        ("explin", [1.0, 2.0, 2.0, 4.0]),
        ("expexp", [1.0, 2.0, 2.0, 4.0]),
    ];
    for (selector, ranges) in overrides {
        defs.push(def(&format!("ugenmaps_demand_{selector}"), vec![], |b| {
            demand_out(b, |b, d| send(b, d, selector, &map(ranges)));
        }));
    }
    for (name, curve) in [
        ("ugenmaps_demand_lincurve", -4.0),
        ("ugenmaps_demand_lincurve_small", 0.1),
    ] {
        defs.push(def(name, vec![], |b| {
            let args = map_with([0.0, 2.0, 2.0, 4.0], &[("curve", input(curve))]);
            demand_out(b, |b, d| send(b, d, "lincurve", &args));
        }));
    }
    defs
}

/// The arguments `t1` and, if given, `t2` of `lag`.
fn times(t1: impl Into<UGenInput>, t2: Option<f64>) -> Vec<(&'static str, ArgValue)> {
    let mut args = vec![("t1", input(t1))];
    args.extend(t2.map(|t2| ("t2", input(t2))));
    args
}

/// The cases of `ugenlags` of the lag family, from `ugenlags_lag` to
/// `ugenlags_demand`.
fn lags() -> Vec<SynthDef> {
    vec![
        def("ugenlags_lag", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let r = send(b, x, "lag", &times(0.2, None));
            out(b, "kr", r);
        }),
        def("ugenlags_lag_defaults", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "lag", &[]);
            out(b, "kr", r);
        }),
        def("ugenlags_lag_zero", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let r = send(b, x, "lag", &times(0.0, None));
            out(b, "kr", r);
        }),
        def("ugenlags_lag_ud", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let r = send(b, x, "lag", &times(0.1, Some(0.2)));
            out(b, "kr", r);
        }),
        def("ugenlags_lag2", vec![], |b| two_lags(b, "lag2")),
        def("ugenlags_lag3", vec![], |b| two_lags(b, "lag3")),
        def("ugenlags_lagud", vec![], |b| {
            let up_down = |up: f64, down: f64| [("lagTimeU", input(up)), ("lagTimeD", input(down))];
            let x = kr(b, "x", 1.0);
            let first = send(b, x, "lagud", &[]);
            let y = kr(b, "y", 1.0);
            let second = send(b, y, "lag2ud", &up_down(0.1, 0.3));
            let z = kr(b, "z", 1.0);
            let third = send(b, z, "lag3ud", &up_down(0.0, 0.0));
            out(b, "kr", UGenInput::Array(vec![first, second, third]));
        }),
        def("ugenlags_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "lag", &times(0.01, None));
            out(b, "ar", r);
        }),
        def("ugenlags_scalar", vec![], |b| {
            let r = rand(b, 0.0, 1.0);
            let lagged = send(b, r, "lag", &times(0.1, None));
            out(b, "kr", lagged);
        }),
        def("ugenlags_demand", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "lag", &times(0.1, None)));
        }),
    ]
}

/// `Out.kr(0, [\x.kr(1).selector(0.1), \y.kr(1).selector(0.1, 0.2)])`.
fn two_lags(b: &mut SynthDefBuilder, selector: &str) {
    let x = kr(b, "x", 1.0);
    let first = send(b, x, selector, &times(0.1, None));
    let y = kr(b, "y", 1.0);
    let second = send(b, y, selector, &times(0.1, Some(0.2)));
    out(b, "kr", UGenInput::Array(vec![first, second]));
}

/// The cases of `ugenlags` of `varlag`, `slew` and the times, from
/// `ugenlags_varlag` to `ugenlags_varlag_audio`.
fn varlags() -> Vec<SynthDef> {
    let varlag = |b: &mut SynthDefBuilder, args: &[(&str, ArgValue)]| {
        let x = kr(b, "x", 1.0);
        let r = send(b, x, "varlag", args);
        out(b, "kr", r);
    };
    vec![
        def("ugenlags_varlag", vec![], |b| {
            varlag(b, &[("time", input(0.2)), ("curvature", input(2.0))]);
        }),
        def("ugenlags_varlag_defaults", vec![], |b| varlag(b, &[])),
        def("ugenlags_varlag_zero", vec![], |b| {
            varlag(b, &[("time", input(0.0))]);
        }),
        def("ugenlags_varlag_start", vec![], |b| {
            let args = [
                ("time", input(0.2)),
                ("curvature", input(0.0)),
                ("warp", ArgValue::from("lin")),
                ("start", input(0.5)),
            ];
            varlag(b, &args);
        }),
        def("ugenlags_slew", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "slew", &[]);
            let s = sine(b, "kr", 2.0);
            let second = send(b, s, "slew", &[("up", input(2.0)), ("down", input(3.0))]);
            out(b, "kr", UGenInput::Array(vec![first, second]));
        }),
        def("ugenlags_slew_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(
                b,
                s,
                "slew",
                &[("up", input(100.0)), ("down", input(100.0))],
            );
            out(b, "ar", r);
        }),
        def("ugenlags_ugen_time", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let t = kr(b, "t", 0.1);
            let r = send(b, x, "lag", &times(t, None));
            out(b, "kr", r);
        }),
        def("ugenlags_array_time", vec![], |b| {
            let x = kr(b, "x", 1.0);
            let r = send(b, x, "lag", &times(array(&[0.1, 0.2]), None));
            out(b, "kr", r);
        }),
        def("ugenlags_varlag_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "varlag", &[("time", input(0.2))]);
            out(b, "ar", r);
        }),
    ]
}

/// The cases of `ugenmisc` from `ugenmisc_blend` to `ugenmisc_dpoll`.
fn misc() -> Vec<SynthDef> {
    let blend =
        |that: UGenInput, frac: UGenInput| [("that", input(that)), ("blendFrac", input(frac))];
    vec![
        def("ugenmisc_blend", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let that = sine(b, "kr", 2.0);
            let r = send(b, s, "blend", &blend(that, 0.25.into()));
            out(b, "kr", r);
        }),
        def("ugenmisc_blend_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let that = sine(b, "ar", 2.0);
            let r = send(b, s, "blend", &[("that", input(that))]);
            out(b, "ar", r);
        }),
        def("ugenmisc_blend_number", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "blend", &blend(0.5.into(), 0.25.into()));
            out(b, "kr", r);
        }),
        def("ugenmisc_blend_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let that = sine(b, "kr", 2.0);
            let frac = kr(b, "f", 0.5);
            let r = send(b, s, "blend", &blend(that, frac));
            out(b, "kr", r);
        }),
        def("ugenmisc_minnyquist", vec![], |b| {
            let f = kr(b, "f", 1000.0);
            let r = send(b, f, "minNyquist", &[]);
            out(b, "kr", r);
        }),
        def("ugenmisc_degrad", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "degrad", &[]);
            let s = sine(b, "kr", 2.0);
            let second = send(b, s, "raddeg", &[]);
            out(b, "kr", UGenInput::Array(vec![first, second]));
        }),
        def("ugenmisc_moddif", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "moddif", &[("that", input(0.5))]);
            let r = rand(b, 0.0, 1.0);
            let second = send(b, r, "moddif", &[]);
            out(b, "kr", UGenInput::Array(vec![first, second]));
            let s = sine(b, "ar", 440.0);
            let third = send(b, s, "moddif", &[("that", input(0.5)), ("mod", input(2.0))]);
            out(b, "ar", third);
        }),
        def("ugenmisc_moddif_demand", vec![], |b| {
            let args = [("that", input(1.0)), ("mod", input(3.0))];
            demand_out(b, |b, d| send(b, d, "moddif", &args));
        }),
        def("ugenmisc_sanitize", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "sanitize", &[]);
            out(b, "kr", first);
            let s = sine(b, "ar", 440.0);
            let second = send(b, s, "sanitize", &[]);
            out(b, "ar", second);
        }),
        def("ugenmisc_poll", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, s, "poll", &[]);
            out(b, "kr", r);
        }),
        def("ugenmisc_poll_args", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let trig = impulse(b, "kr", 2.0);
            let args = [
                ("trig", input(trig)),
                ("label", ArgValue::from("x")),
                ("trigid", input(3.0)),
            ];
            let r = send(b, s, "poll", &args);
            out(b, "kr", r);
        }),
        def("ugenmisc_dpoll", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "dpoll", &[]));
        }),
    ]
}

/// The cases of `ugenmisc` from `ugenmisc_checkbadvalues` to
/// `ugenmisc_snap_demand`.
fn checks() -> Vec<SynthDef> {
    vec![
        def("ugenmisc_checkbadvalues", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "checkBadValues", &[]);
            out(b, "kr", first);
            let s = sine(b, "ar", 440.0);
            let second = send(
                b,
                s,
                "checkBadValues",
                &[("id", input(1.0)), ("post", input(0.0))],
            );
            out(b, "ar", second);
        }),
        def("ugenmisc_checkbadvalues_scalar", vec![], |b| {
            let r = rand(b, 0.0, 1.0);
            let checked = send(b, r, "checkBadValues", &[]);
            out(b, "kr", checked);
        }),
        def("ugenmisc_checkbadvalues_demand", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "checkBadValues", &[]));
        }),
        def("ugenmisc_snap", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, s, "snap", &[]);
            let s = sine(b, "kr", 2.0);
            let args = [
                ("resolution", input(0.5)),
                ("margin", input(0.1)),
                ("strength", input(0.5)),
            ];
            let second = send(b, s, "softRound", &args);
            out(b, "kr", UGenInput::Array(vec![first, second]));
        }),
        def("ugenmisc_snap_scalar", vec![], |b| {
            let r = rand(b, 0.0, 1.0);
            let snapped = send(b, r, "snap", &[]);
            out(b, "kr", snapped);
        }),
        def("ugenmisc_snap_demand", vec![], |b| {
            demand_out(b, |b, d| send(b, d, "snap", &[]));
        }),
    ]
}

/// The cases of `ugenmisc` from `ugenmisc_binaryvalue` to
/// `ugenmisc_snap_audio`.
fn more_misc() -> Vec<SynthDef> {
    let one = |selector: &'static str, args: &'static [(&'static str, f64)]| {
        move |b: &mut SynthDefBuilder| {
            let s = sine(b, "kr", 1.0);
            let args: Vec<(&str, ArgValue)> = args
                .iter()
                .map(|&(name, value)| (name, input(value)))
                .collect();
            let r = send(b, s, selector, &args);
            out(b, "kr", r);
        }
    };
    vec![
        def("ugenmisc_binaryvalue", vec![], one("binaryValue", &[])),
        def("ugenmisc_signs", vec![], |b| {
            let signs = ["isPositive", "isNegative", "isStrictlyPositive"]
                .into_iter()
                .zip(1..=3)
                .map(|(selector, freq)| {
                    let s = sine(b, "kr", f64::from(freq));
                    send(b, s, selector, &[])
                })
                .collect();
            out(b, "kr", UGenInput::Array(signs));
        }),
        def(
            "ugenmisc_lazy_equals",
            vec![],
            one("|==|", &[("that", 0.5)]),
        ),
        def(
            "ugenmisc_degreetokey",
            vec![],
            one("degreeToKey", &[("scale", 0.0)]),
        ),
        def("ugenmisc_degreetokey_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let args = [("scale", input(0.0)), ("stepsPerOctave", input(7.0))];
            let r = send(b, s, "degreeToKey", &args);
            out(b, "kr", r);
        }),
        def("ugenmisc_if", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let test = b.binary_op(">", s, 0.0).unwrap();
            let r = send(
                b,
                test,
                "if",
                &[("trueUGen", input(200.0)), ("falseUGen", input(400.0))],
            );
            out(b, "kr", r);
        }),
        def("ugenmisc_if_ugen", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let test = b.binary_op(">", s, 0.0).unwrap();
            let truth = sine(b, "kr", 2.0);
            let r = send(
                b,
                test,
                "if",
                &[("trueUGen", input(truth)), ("falseUGen", input(400.0))],
            );
            out(b, "kr", r);
        }),
        def("ugenmisc_snap_audio", vec![], |b| {
            let s = sine(b, "ar", 440.0);
            let r = send(b, s, "snap", &[("resolution", input(0.5))]);
            out(b, "ar", r);
        }),
    ]
}

/// The elements of a nested array, in order, as `flat` gives them.
fn flat(input: UGenInput) -> Vec<UGenInput> {
    match input {
        UGenInput::Array(items) => items.into_iter().flat_map(flat).collect(),
        other => vec![other],
    }
}

/// The cases of `ugenreceivers` with an array, from `ugenreceivers_range` to
/// `ugenreceivers_sanitize`.
fn arrays() -> Vec<SynthDef> {
    let two_sines = |b: &mut SynthDefBuilder| {
        let first = sine(b, "kr", 1.0);
        let second = sine(b, "kr", 2.0);
        UGenInput::Array(vec![first, second])
    };
    vec![
        def("ugenreceivers_range", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let p = pulse(b, "kr", 1.0);
            let r = send(b, vec![s, p], "range", &lo_hi(0.0, array(&[1.0, 2.0])));
            out(b, "kr", r);
        }),
        def("ugenreceivers_wraps", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, vec![s], "range", &lo_hi(0.0, array(&[1.0, 2.0, 3.0])));
            out(b, "kr", r);
        }),
        def("ugenreceivers_nested", vec![], |b| {
            let inner = two_sines(b);
            let third = sine(b, "kr", 3.0);
            let r = send(b, vec![inner, third], "range", &lo_hi(2.0, 4.0));
            out(b, "kr", UGenInput::Array(flat(r)));
        }),
        def("ugenreceivers_number", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(b, vec![s, 0.5.into()], "lag", &times(0.2, None));
            out(b, "kr", r);
        }),
        def("ugenreceivers_linlin", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = send(
                b,
                vec![s, 0.25.into()],
                "linlin",
                &map([0.0, 1.0, 200.0, 800.0]),
            );
            out(b, "kr", r);
        }),
        def("ugenreceivers_unary", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let first = send(b, vec![s, 0.5.into()], "degrad", &[]);
            let s = sine(b, "kr", 2.0);
            let saw = by_name(b, "LFSaw", "kr", &[("freq", input(2.0))]);
            let second = send(b, vec![s, saw], "isPositive", &[]);
            out(
                b,
                "kr",
                UGenInput::Array([flat(first), flat(second)].concat()),
            );
        }),
        def("ugenreceivers_lazy_equals", vec![], |b| {
            let pair = two_sines(b);
            let third = sine(b, "kr", 3.0);
            let r = send(b, pair, "|==|", &[("that", input(third))]);
            out(b, "kr", r);
        }),
        def("ugenreceivers_minnyquist", vec![], |b| {
            let f = kr(b, "f", 1000.0);
            let g = kr(b, "g", 2000.0);
            let r = send(b, vec![f, g], "minNyquist", &[]);
            out(b, "kr", r);
        }),
        def("ugenreceivers_poll", vec![], |b| {
            let pair = two_sines(b);
            let r = send(b, pair, "poll", &[]);
            out(b, "kr", r);
        }),
        def("ugenreceivers_dpoll", vec![], |b| {
            let trig = impulse(b, "kr", 1.0);
            let first = dseq(b, &[1.0, 2.0]);
            let second = dseq(b, &[3.0, 4.0]);
            let polled = send(b, vec![first, second], "dpoll", &[]);
            let args = [
                ("trig", input(trig)),
                ("reset", input(0.0)),
                ("demandUGens", input(polled)),
            ];
            let demand = by_name(b, "Demand", "kr", &args);
            out(b, "kr", demand);
        }),
        def("ugenreceivers_degreetokey", vec![], |b| {
            let pair = two_sines(b);
            let r = send(b, pair, "degreeToKey", &[("scale", input(0.0))]);
            out(b, "kr", r);
        }),
        def("ugenreceivers_checkbadvalues", vec![], |b| {
            let pair = two_sines(b);
            let args = [("id", input(0.0)), ("post", input(array(&[1.0, 2.0])))];
            let r = send(b, pair, "checkBadValues", &args);
            out(b, "kr", r);
        }),
        def("ugenreceivers_sanitize", vec![], |b| {
            let pair = two_sines(b);
            let r = send(b, pair, "sanitize", &[]);
            out(b, "kr", r);
        }),
    ]
}

/// `number.selector(args)`, which computes a number or adds UGens.
fn number(b: &mut SynthDefBuilder, x: f64, selector: &str, args: &[(&str, ArgValue)]) -> UGenInput {
    send(b, x, selector, args)
}

/// The arguments `inMin`, `inMax`, `outMin` and `outMax`, then `curve`.
fn curve_map(ranges: [f64; 4], curve: f64) -> Vec<(&'static str, ArgValue)> {
    map_with(ranges, &[("curve", input(curve))])
}

/// The cases of `ugenreceivers` with a number, from `ugenreceivers_numbers`
/// to `ugenreceivers_number_lazy_equals`.
fn numbers() -> Vec<SynthDef> {
    vec![
        def("ugenreceivers_numbers", vec![], |b| {
            let values = vec![
                number(b, 0.25, "linlin", &map([0.0, 1.0, 200.0, 800.0])),
                number(b, 0.5, "linexp", &map([0.0, 1.0, 20.0, 20000.0])),
                number(b, 0.5, "explin", &map([0.01, 1.0, 0.0, 1.0])),
                number(b, 0.5, "expexp", &map([0.01, 1.0, 20.0, 20000.0])),
                number(b, 0.5, "lincurve", &curve_map([0.0, 1.0, 0.0, 1.0], -4.0)),
                number(b, 0.5, "curvelin", &curve_map([0.0, 1.0, 0.0, 1.0], -4.0)),
                number(b, 0.75, "bilin", &bilin_numbers()),
                number(b, 0.25, "bilin", &bilin_numbers()),
            ];
            out(b, "kr", UGenInput::Array(values));
        }),
        def("ugenreceivers_clips", vec![], |b| {
            let values = vec![
                number(b, 1.5, "linlin", &map([0.0, 1.0, 200.0, 800.0])),
                number(
                    b,
                    -0.5,
                    "linlin",
                    &map_with([0.0, 1.0, 2.0, 4.0], &[("clip", ArgValue::from("max"))]),
                ),
                number(b, 1.5, "clip", &lo_hi(0.0, 1.0)),
                number(b, 1.5, "fold", &lo_hi(0.0, 1.0)),
                number(b, 1.5, "wrap", &lo_hi(0.0, 1.0)),
                number(b, 0.5, "lincurve", &curve_map([0.0, 1.0, 0.0, 1.0], 0.0001)),
            ];
            out(b, "kr", UGenInput::Array(values));
        }),
        def("ugenreceivers_number_ugen", vec![], |b| {
            let low = kr(b, "lo", 200.0);
            let args = [
                ("inMin", input(0.0)),
                ("inMax", input(1.0)),
                ("outMin", input(low)),
                ("outMax", input(800.0)),
            ];
            let r = number(b, 0.5, "linlin", &args);
            out(b, "kr", r);
        }),
        def("ugenreceivers_number_misc", vec![], |b| {
            let scale = array(&[0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0]);
            let values = vec![
                number(b, 0.5, "lag", &times(0.1, None)),
                number(b, 0.5, "varlag", &[("time", input(0.1))]),
                number(
                    b,
                    0.5,
                    "blend",
                    &[("that", input(1.0)), ("blendFrac", input(0.25))],
                ),
                number(b, 90.0, "degrad", &[]),
                number(b, 1.0, "raddeg", &[]),
                number(b, 0.5, "moddif", &[("that", input(0.9))]),
                number(b, 0.97, "snap", &[]),
                number(b, 0.97, "softRound", &[]),
                number(b, -0.5, "binaryValue", &[]),
                number(b, 2.5, "degreeToKey", &[("scale", input(scale))]),
            ];
            out(b, "kr", UGenInput::Array(values));
        }),
        def("ugenreceivers_number_blend", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = number(
                b,
                0.5,
                "blend",
                &[("that", input(s)), ("blendFrac", input(0.25))],
            );
            out(b, "kr", r);
        }),
        def("ugenreceivers_number_poll", vec![], |b| {
            let r = number(b, 0.5, "poll", &[]);
            out(b, "kr", r);
        }),
        def("ugenreceivers_number_lazy_equals", vec![], |b| {
            let s = sine(b, "kr", 1.0);
            let r = number(b, 0.5, "|==|", &[("that", input(s))]);
            out(b, "kr", r);
        }),
    ]
}

/// The arguments of `bilin(0.5, 0, 1, 0.2, 0, 1)`.
fn bilin_numbers() -> Vec<(&'static str, ArgValue)> {
    let names = [
        "inCenter",
        "inMin",
        "inMax",
        "outCenter",
        "outMin",
        "outMax",
    ];
    names
        .into_iter()
        .zip([0.5, 0.0, 1.0, 0.2, 0.0, 1.0])
        .map(|(name, value)| (name, input(value)))
        .collect()
}

/// `Env([0, 1, 0.5], [1, 1])`.
fn env(b: &mut SynthDefBuilder) -> Env {
    let args = [
        ("levels", input(array(&[0.0, 1.0, 0.5]))),
        ("times", input(array(&[1.0, 1.0]))),
    ];
    b.env("new", &args).unwrap()
}

/// `Out.kr(0, EnvGen.kr(env.selector(args), gate))`.
fn ranged_out(
    b: &mut SynthDefBuilder,
    env: &Env,
    selector: &str,
    args: &[(&str, ArgValue)],
    gate: Option<UGenInput>,
) {
    let ranged = b
        .send_env(env, selector, args)
        .unwrap_or_else(|error| panic!("{selector}: {error}"));
    let mut all = vec![("envelope", ArgValue::Env(ranged))];
    all.extend(gate.map(|gate| ("gate", input(gate))));
    let generated = by_name(b, "EnvGen", "kr", &all);
    out(b, "kr", generated);
}

/// The cases of `pollarrays`: `Array:poll` and `Array:dpoll` with arrays
/// among their arguments, and `Poll.new` of nested arrays.
fn poll_arrays() -> Vec<SynthDef> {
    let two_sines = |b: &mut SynthDefBuilder| {
        let first = sine(b, "kr", 1.0);
        let second = sine(b, "kr", 2.0);
        UGenInput::Array(vec![first, second])
    };
    let nested = |b: &mut SynthDefBuilder| {
        let first = sine(b, "ar", 1.0);
        let second = sine(b, "kr", 2.0);
        let third = sine(b, "kr", 3.0);
        UGenInput::Array(vec![UGenInput::Array(vec![first, second]), third])
    };
    let polled_out = |b: &mut SynthDefBuilder, args: &[(&str, ArgValue)]| {
        let pair = two_sines(b);
        let r = send(b, pair, "poll", args);
        out(b, "kr", r);
    };
    vec![
        def("pollarrays_trig", vec![], |b| {
            let pair = two_sines(b);
            let trigs = vec![impulse(b, "kr", 1.0), impulse(b, "kr", 2.0)];
            let r = send(b, pair, "poll", &[("trig", input(trigs))]);
            out(b, "kr", r);
        }),
        def("pollarrays_three", vec![], |b| {
            polled_out(b, &[("trig", input(array(&[1.0, 2.0, 3.0])))]);
        }),
        def("pollarrays_trigid", vec![], |b| {
            polled_out(b, &[("trigid", input(array(&[1.0, 2.0])))]);
        }),
        def("pollarrays_label", vec![], |b| {
            let label = ArgValue::Text("x".to_owned());
            polled_out(b, &[("trig", input(5.0)), ("label", label)]);
        }),
        def("pollarrays_nested", vec![], |b| {
            let receiver = nested(b);
            send(b, receiver, "poll", &[]);
        }),
        def("pollarrays_new_nested", vec![], |b| {
            let polled = nested(b);
            let args = [("trig", input(10.0)), ("in", input(polled))];
            by_name(b, "Poll", "new", &args);
        }),
        def("pollarrays_new_trigs", vec![], |b| {
            let polled = nested(b);
            let args = [("trig", input(array(&[1.0, 2.0]))), ("in", input(polled))];
            by_name(b, "Poll", "new", &args);
        }),
        def("pollarrays_dpoll_run", vec![], |b| {
            let sequences = vec![dseq(b, &[1.0, 2.0]), dseq(b, &[3.0])];
            dpolled_out(b, sequences, &[("run", input(array(&[1.0, 0.0])))]);
        }),
        def("pollarrays_dpoll_trigid", vec![], |b| {
            let sequences = vec![dseq(b, &[1.0, 2.0]), dseq(b, &[3.0])];
            dpolled_out(b, sequences, &[("trigid", input(array(&[7.0, 8.0])))]);
        }),
        def("pollarrays_dpoll_nested", vec![], |b| {
            let inner = UGenInput::Array(vec![dseq(b, &[1.0, 2.0]), dseq(b, &[3.0])]);
            let sequences = vec![inner, dseq(b, &[4.0])];
            dpolled_out(b, sequences, &[]);
        }),
    ]
}

/// `Out.kr(0, Demand.kr(Impulse.kr(1), 0, sequences.dpoll(args)))`.
fn dpolled_out(b: &mut SynthDefBuilder, sequences: Vec<UGenInput>, args: &[(&str, ArgValue)]) {
    let trig = impulse(b, "kr", 1.0);
    let polled = send(b, sequences, "dpoll", args);
    let args = [
        ("trig", input(trig)),
        ("reset", input(0.0)),
        ("demandUGens", input(polled)),
    ];
    let demand = by_name(b, "Demand", "kr", &args);
    out(b, "kr", demand);
}

/// The cases of `envranges`.
fn env_ranges() -> Vec<SynthDef> {
    vec![
        def("envranges_range", vec![], |b| {
            let e = env(b);
            ranged_out(b, &e, "range", &lo_hi(2.0, 4.0), None);
        }),
        def("envranges_exprange", vec![], |b| {
            let e = env(b);
            ranged_out(b, &e, "exprange", &lo_hi(0.01, 1.0), None);
        }),
        def("envranges_curverange", vec![], |b| {
            let e = env(b);
            let args = [
                ("lo", input(0.0)),
                ("hi", input(1.0)),
                ("curve", input(-4.0)),
            ];
            ranged_out(b, &e, "curverange", &args, None);
        }),
        def("envranges_defaults", vec![], |b| {
            let e = b.env("perc", &[]).unwrap();
            ranged_out(b, &e, "range", &[], None);
        }),
        def("envranges_ugen", vec![], |b| {
            let e = env(b);
            let lo = kr(b, "lo", 2.0);
            ranged_out(b, &e, "range", &lo_hi(lo, 4.0), None);
        }),
        def("envranges_adsr", vec![], |b| {
            let e = b.env("adsr", &[]).unwrap();
            let ranged = b.send_env(&e, "range", &lo_hi(2.0, 4.0)).unwrap();
            let gate = kr(b, "gate", 1.0);
            let all = [("envelope", ArgValue::Env(ranged)), ("gate", input(gate))];
            let generated = by_name(b, "EnvGen", "kr", &all);
            out(b, "kr", generated);
        }),
    ]
}
