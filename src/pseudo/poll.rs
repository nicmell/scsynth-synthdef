//! The bodies of `Poll.sc`.

use super::{Args, lang};
use crate::builder::{NewUGen, SynthDefBuilder, symbol_codes};
use crate::catalog;
use crate::check_inputs;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `Poll.ar`, `Poll.kr` and `Poll.new`: a `Poll` for each instance of
/// `trig`, `in` and `trigid`, with the label left out of the expansion. The
/// call gives back `in`.
pub(super) fn poll(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let input = args.input("in")?;
    let label = args.text("label")?;
    let values = [args.input("trig")?, input.clone(), args.input("trigid")?];
    builder.expand(
        "Poll",
        &["trig", "in", "trigid"],
        &values,
        true,
        &mut |builder, instance| {
            let rate = match args.selector() {
                "ar" => Rate::Audio,
                "kr" => Rate::Control,
                _ => lang::rate(&instance[1]).unwrap_or(Rate::Scalar),
            };
            new_poll(builder, rate, instance, label.as_deref())
        },
    )?;
    Ok(input)
}

/// `Poll:new1` for one instance of `trig`, `in` and `trigid`: the default
/// label, control rate for scalar rate, and an `Impulse` for a number
/// trigger.
fn new_poll(
    builder: &mut SynthDefBuilder,
    rate: Rate,
    instance: &[UGenInput],
    label: Option<&str>,
) -> Result<UGenInput, BuildError> {
    let (trig, input, trigid) = (&instance[0], &instance[1], &instance[2]);
    let label = label.map_or_else(
        || format!("UGen({})", sclang_class(builder, input)),
        str::to_owned,
    );
    let rate = if rate == Rate::Scalar {
        Rate::Control
    } else {
        rate
    };
    let trig = match *trig {
        UGenInput::Number(freq) => builder.ugen(
            "Impulse",
            lang::selector_for_rate(rate),
            &[("freq", freq.into()), ("phase", 0.0.into())],
        )?,
        ref other => other.clone(),
    };
    let mut inputs = vec![trig, input.clone(), trigid.clone()];
    inputs.extend(symbol_codes(&label));
    let rule = catalog::lookup("Poll").map(|class| class.check_inputs);
    if let Some(rule) = rule {
        check_inputs::check(rule, rate, &inputs).map_err(|message| BuildError::CheckInputs {
            class: "Poll".to_owned(),
            message,
        })?;
    }
    builder.add(NewUGen::new("Poll", rate).inputs(inputs))
}

/// The class of `input` in sclang, for a default label: `Integer` for a whole
/// number in the range of an Integer, `Float` for another number,
/// `OutputProxy` for an output of a `MultiOutUGen`, and else the class of
/// the UGen of the output.
fn sclang_class(builder: &SynthDefBuilder, input: &UGenInput) -> String {
    match *input {
        UGenInput::Number(number)
            if number.fract() == 0.0 && number.abs() <= f64::from(i32::MAX) =>
        {
            "Integer".to_owned()
        }
        UGenInput::Number(_) => "Float".to_owned(),
        ref output => {
            let class = builder.class_of(output).unwrap_or("UGen");
            let multi = catalog::lookup(class)
                .is_some_and(|ugen| ugen.superclasses.contains(&"MultiOutUGen"));
            if multi {
                "OutputProxy".to_owned()
            } else {
                class.to_owned()
            }
        }
    }
}
