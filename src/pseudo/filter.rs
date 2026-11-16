//! The pseudo-UGens of `Filter.sc`.

use super::{Args, lang};
use crate::builder::{NewUGen, SynthDefBuilder};
use crate::catalog;
use crate::check_inputs;
use crate::env::{Curve, Env, bad, shape_named};
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `Changed.ar` and `Changed.kr`: `HPZ1(input).abs > threshold`.
pub(super) fn changed(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let slope = builder.ugen("HPZ1", args.selector(), &[("in", args.value("input")?)])?;
    let size = lang::unary(builder, "abs", slope)?;
    lang::binary(builder, ">", size, args.input("threshold")?)
}

/// `VarLag.ar` and `VarLag.kr`. An input at scalar rate, or a `time` of 0,
/// gives back `in.madd(mul, add)` before the expansion. Else `multiNew`
/// expands `in`, `time`, `curvature`, `warp` and `start`, and `madd`
/// applies to the result.
pub(super) fn var_lag(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let input = args.input("in")?;
    let time = args.input("time")?;
    let (mul, add) = (args.input("mul")?, args.input("add")?);
    if lang::rate(&input) == Some(Rate::Scalar) || time == UGenInput::Number(0.0) {
        return builder.madd(input, mul, add);
    }
    let wire = [input, time, args.input("curvature")?];
    let start = args.optional("start")?;
    let lagged = var_lag_new(builder, args.selector(), wire, args.curve("warp")?, start)?;
    builder.madd(lagged, mul, add)
}

/// `VarLag.multiNew`: the expansion of `in`, `time` and `curvature` of
/// `wire`, the warp and the start, then `new1` of each instance with the
/// selector of its rate, `ar` or `kr`.
pub(super) fn var_lag_new(
    builder: &mut SynthDefBuilder,
    selector: &str,
    wire: [UGenInput; 3],
    warp: Curve,
    start: Option<UGenInput>,
) -> Result<UGenInput, BuildError> {
    let mut warps = Vec::new();
    let warp = leaves(warp, &mut warps);
    let mut names = vec!["in", "time", "curvature", "warp"];
    let mut values = Vec::from(wire);
    values.push(warp);
    if let Some(start) = start {
        names.push("start");
        values.push(start);
    }
    builder.expand("VarLag", &names, &values, true, &mut |builder, instance| {
        let warp = match instance[3] {
            UGenInput::Number(index) => warps.get(super::count_to_usize(index)),
            _ => None,
        };
        let warp = warp.ok_or_else(|| BuildError::WrongKind {
            class: "VarLag".to_owned(),
            argument: "warp".to_owned(),
            expected: "a curve",
        })?;
        let start = instance.get(4).unwrap_or(&instance[0]).clone();
        new1(builder, selector, &instance[..3], warp, start)
    })
}

/// The curve in its shape, each leaf replaced by its index in `warps`.
fn leaves(curve: Curve, warps: &mut Vec<Curve>) -> UGenInput {
    match curve {
        Curve::Array(curves) => UGenInput::Array(
            curves
                .into_iter()
                .map(|curve| leaves(curve, warps))
                .collect(),
        ),
        leaf => {
            warps.push(leaf);
            super::number(warps.len() - 1)
        }
    }
}

/// `VarLag.new1` of one instance: `in`, `time` and `curvature`, the warp
/// and the start. `curve = Env.shapeNames[warp] ? warp`: a curve of 1 makes
/// a real `VarLag`, any other the `EnvGen` of `Env([start, in], [time],
/// warp)`, with the curve and the curvature at indices 6 and 7.
fn new1(
    builder: &mut SynthDefBuilder,
    selector: &str,
    wire: &[UGenInput],
    warp: &Curve,
    start: UGenInput,
) -> Result<UGenInput, BuildError> {
    let (input, time, curvature) = (wire[0].clone(), wire[1].clone(), wire[2].clone());
    let curve = match warp {
        Curve::Shape(name) => shape_named(name)
            .map(UGenInput::Number)
            .ok_or_else(|| bad("VarLag", &format!("no shape is named {name}")))?,
        Curve::Value(value) => value.clone(),
        Curve::Array(_) => return Err(bad("VarLag", "a warp is one curve")),
    };
    if curve == UGenInput::Number(1.0) {
        let rate = if selector == "ar" {
            Rate::Audio
        } else {
            Rate::Control
        };
        let inputs = [input, time, start];
        if let Some(class) = catalog::lookup("VarLag") {
            check_inputs::check(class.check_inputs, rate, &inputs).map_err(|message| {
                BuildError::CheckInputs {
                    class: "VarLag".to_owned(),
                    message,
                }
            })?;
        }
        return builder.add(NewUGen::new("VarLag", rate).inputs(inputs).outputs([rate]));
    }
    let env = Env::from_parts(
        vec![start, input.clone()],
        vec![time.clone()],
        warp.clone(),
        None,
        None,
        UGenInput::Number(0.0),
    )?;
    let UGenInput::Array(mut envelope) = env.as_array()? else {
        return Err(bad(
            "VarLag",
            "the envelope of one instance has one channel",
        ));
    };
    envelope[6] = curve;
    envelope[7] = curvature;
    let changed = builder.ugen("Changed", selector, &[("input", input.into())])?;
    let impulse = builder.ugen("Impulse", selector, &[("freq", 0.0.into())])?;
    let mut trigger = lang::binary(builder, "+", changed, impulse)?;
    if lang::rate(&time) != Some(Rate::Scalar) {
        let changed = builder.ugen("Changed", "kr", &[("input", time.into())])?;
        trigger = lang::binary(builder, "+", trigger, changed)?;
    }
    builder.ugen(
        "EnvGen",
        selector,
        &[
            ("envelope", UGenInput::Array(envelope).into()),
            ("gate", trigger.into()),
        ],
    )
}
