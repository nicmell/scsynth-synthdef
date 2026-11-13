//! The pseudo-UGens of `Splay.sc`.

use super::mix::mix;
use super::{Args, elements, lang, number};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::{ArgValue, UGenInput};

/// `Splay.ar` and `Splay.kr`: the expansion of `multiNewList` over `spread`,
/// `level`, `center`, `levelComp` and each element of `inArray`, then
/// `new1` for each instance.
pub(super) fn splay(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let level_comp = args.value("levelComp")?;
    let mut names = vec!["spread", "level", "center"];
    let mut values = vec![
        args.input("spread")?,
        args.input("level")?,
        args.input("center")?,
    ];
    let comp_expands = match &level_comp {
        ArgValue::Input(input) => {
            names.push("levelComp");
            values.push(input.clone());
            true
        }
        ArgValue::Bool(_) => false,
        _ => return Err(args.wrong_kind("levelComp", "a boolean, a number or an output")),
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

/// `LevelComp.new`: the factor of the level of `n` inputs. `true` gives the
/// equal power of audio rate or the equal amplitude of control rate, `false`
/// gives 1, and a number or a UGen is the exponent of `n.reciprocal`.
fn level_comp_new(
    builder: &mut SynthDefBuilder,
    level_comp: &ArgValue,
    audio: bool,
    n: usize,
) -> Result<UGenInput, BuildError> {
    match level_comp {
        ArgValue::Bool(true) => {
            let reciprocal = lang::unary(builder, "reciprocal", number(n))?;
            if audio {
                lang::unary(builder, "sqrt", reciprocal)
            } else {
                Ok(reciprocal)
            }
        }
        ArgValue::Input(exponent) => {
            let reciprocal = lang::unary(builder, "reciprocal", number(n))?;
            let clipped = lang::clip(builder, exponent.clone(), 0.0, 1.0)?;
            lang::binary(builder, "pow", reciprocal, clipped)
        }
        _ => Ok(UGenInput::Number(1.0)),
    }
}
