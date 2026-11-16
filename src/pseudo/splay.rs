//! The pseudo-UGens of `Splay.sc`.

use super::mix::mix;
use super::ugen_methods::send_message;
use super::{Args, elements, float, lang, number};
use crate::builder::SynthDefBuilder;
use crate::env::flop;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::{ArgValue, UGenInput};

/// `Splay.ar` and `Splay.kr`: the expansion of `multiNewList` over `spread`,
/// `level`, `center`, `levelComp` and each element of `inArray`, then
/// `new1` for each instance.
pub(super) fn splay(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let level_comp = level_comp(args)?;
    let mut names = vec!["spread", "level", "center"];
    let mut values = vec![
        args.input("spread")?,
        args.input("level")?,
        args.input("center")?,
    ];
    let comp_expands = if let ArgValue::Input(input) = &level_comp {
        names.push("levelComp");
        values.push(input.clone());
        true
    } else {
        false
    };
    for input in args.array("inArray")? {
        names.push("inArray");
        values.push(input);
    }
    let selector = args.selector();
    builder.expand("Splay", &names, &values, true, &mut |builder, instance| {
        let (comp, inputs) = if comp_expands {
            (ArgValue::Input(instance[3].clone()), &instance[4..])
        } else {
            (level_comp.clone(), &instance[3..])
        };
        let [spread, level, center] = [&instance[0], &instance[1], &instance[2]].map(Clone::clone);
        splay_new1(builder, selector, [spread, level, center], &comp, inputs)
    })
}

/// `Splay:new1`: a `Pan2` for each input, at positions spread from -1 to 1
/// around `center`, then `Mix`, times the level.
fn splay_new1(
    builder: &mut SynthDefBuilder,
    selector: &str,
    [spread, level, center]: [UGenInput; 3],
    level_comp: &ArgValue,
    inputs: &[UGenInput],
) -> Result<UGenInput, BuildError> {
    let n = inputs.len().max(2);
    let indices = UGenInput::Array((0..n).map(number).collect());
    let step = lang::binary(builder, "/", 2.0.into(), number(n - 1))?;
    let scaled = lang::binary(builder, "*", indices, step)?;
    let shifted = lang::binary(builder, "-", scaled, 1.0.into())?;
    let spread_out = lang::binary(builder, "*", shifted, spread)?;
    let positions = lang::binary(builder, "+", spread_out, center)?;
    let comp = level_comp_new(builder, level_comp, selector == "ar", n)?;
    let level = lang::binary(builder, "*", level, comp)?;
    let pans = builder.ugen(
        "Pan2",
        selector,
        &[
            ("in", UGenInput::Array(inputs.to_vec()).into()),
            ("pos", positions.into()),
        ],
    )?;
    let mixed = mix(builder, &elements(pans))?;
    lang::binary(builder, "*", mixed, level)
}

/// `SplayAz.ar` and `SplayAz.kr`: a `PanAz` for each input, at positions
/// spread around `center`, then a `Mix` for each of the `numChans`
/// channels.
pub(super) fn splay_az(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let inputs = args.array("inArray")?;
    let n = inputs.len().max(1);
    let fraction = lang::binary(builder, "/", number(n - 1), number(n))?;
    let norm_spread = lang::binary(builder, "*", fraction, args.input("spread")?)?;
    let center = args.input("center")?;
    let positions = if n == 1 {
        center
    } else {
        let low = lang::binary(builder, "-", center.clone(), norm_spread.clone())?;
        let high = lang::binary(builder, "+", center, norm_spread)?;
        resamp1(builder, &[low, high], n)?
    };
    let selector = args.selector();
    let comp = level_comp_new(builder, &level_comp(args)?, selector == "ar", n)?;
    let level = lang::binary(builder, "*", args.input("level")?, comp)?;
    let pans = builder.ugen(
        "PanAz",
        selector,
        &[
            ("numChans", args.value("numChans")?),
            ("in", UGenInput::Array(inputs).into()),
            ("pos", positions.into()),
            ("level", level.into()),
            ("width", args.value("width")?),
            ("orientation", args.value("orientation")?),
        ],
    )?;
    let mut channels = Vec::new();
    for channel in elements(flop(&elements(pans))) {
        channels.push(mix(builder, &elements(channel))?);
    }
    Ok(UGenInput::Array(channels))
}

/// `resamp1`: `size` values that blend the neighbours of `values` from the
/// first to the last, as `blendAt` does.
fn resamp1(
    builder: &mut SynthDefBuilder,
    values: &[UGenInput],
    size: usize,
) -> Result<UGenInput, BuildError> {
    let factor = float(values.len() - 1) / float((size - 1).max(1));
    (0..size)
        .map(|i| {
            let index = float(i) * factor;
            let low = index.ceil() - 1.0;
            let (a, b) = (clip_at(values, low), clip_at(values, low + 1.0));
            blend(builder, a, b, (index - low).abs())
        })
        .collect::<Result<_, _>>()
        .map(UGenInput::Array)
}

/// `a.blend(b, frac)`, which refuses `a` or `b` at scalar or demand rate
/// with `UnsupportedRate`.
fn blend(
    builder: &mut SynthDefBuilder,
    a: UGenInput,
    b: UGenInput,
    frac: f64,
) -> Result<UGenInput, BuildError> {
    let demand = [&a, &b]
        .into_iter()
        .any(|value| lang::rate(value) == Some(Rate::Demand));
    let given = vec![Some(ArgValue::from(b)), Some(ArgValue::from(frac))];
    send_message(builder, a, "blend", given).map_err(|error| match error {
        BuildError::BadMethod { .. } => BuildError::UnsupportedRate {
            ugen: "UGen:blend".to_owned(),
            rate: if demand { Rate::Demand } else { Rate::Scalar },
        },
        other => other,
    })
}

/// `clipAt`: the element at `position`, a whole number, within the bounds of
/// `values`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn clip_at(values: &[UGenInput], position: f64) -> UGenInput {
    let index = if position <= 0.0 {
        0
    } else {
        position as usize
    };
    values[index.min(values.len() - 1)].clone()
}

/// The `levelComp` of `args`: a boolean, a number or an output. `LevelComp`
/// clips any other value, which sclang 3.14.1 throws on.
fn level_comp(args: &Args) -> Result<ArgValue, BuildError> {
    match args.value("levelComp")? {
        value @ (ArgValue::Bool(_) | ArgValue::Input(_)) => Ok(value),
        _ => Err(args.wrong_kind("levelComp", "a boolean, a number or an output")),
    }
}

/// `LevelComp.new`: the factor of the level of `n` inputs. `true` gives the
/// equal power of audio rate or the equal amplitude of control rate, `false`
/// gives 1, and a number or a UGen is the exponent of `n.reciprocal`.
fn level_comp_new(
    builder: &mut SynthDefBuilder,
    level_comp: &ArgValue,
    audio: bool,
    n: usize,
) -> Result<UGenInput, BuildError> {
    let reciprocal = lang::unary(builder, "reciprocal", number(n))?;
    match level_comp {
        ArgValue::Bool(true) if audio => lang::unary(builder, "sqrt", reciprocal),
        ArgValue::Bool(true) => Ok(reciprocal),
        ArgValue::Input(exponent) => {
            let bounds = vec![Some(ArgValue::from(0.0)), Some(ArgValue::from(1.0))];
            let clipped = send_message(builder, exponent.clone(), "clip", bounds)?;
            lang::binary(builder, "pow", reciprocal, clipped)
        }
        _ => Ok(UGenInput::Number(1.0)),
    }
}
