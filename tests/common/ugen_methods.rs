//! The graphs of the golden files of the methods of `UGen` that build graphs,
//! built with `SynthDefBuilder::send`. `generator/fixtures.scd` holds the
//! sclang code of each.

use scsynth_synthdef::{ArgValue, SynthDef, SynthDefBuilder, UGenInput};

use super::controls::kr;
use super::pseudo::{def, dseq, impulse, input, out, sine};
use super::{array, by_name, element};

/// The definitions of a golden file of 0.12.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "ugenranges" => [ranges(), other_ranges()].concat(),
        "ugenmaps" => [maps(), curves(), bilins(), clips()].concat(),
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
