//! The bodies of `Poll.sc`.

use super::{Args, elements, lang, sclang_class};
use crate::builder::{NewUGen, SynthDefBuilder, symbol_codes};
use crate::catalog;
use crate::check_inputs;
use crate::env::flop;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `Poll.ar`, `Poll.kr` and `Poll.new`, with one label. The call gives back
/// `in`.
pub(super) fn poll(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let input = args.input("in")?;
    let rate = match args.selector() {
        "ar" => Some(Rate::Audio),
        "kr" => Some(Rate::Control),
        _ => None,
    };
    let label = args.text("label")?;
    let values = [args.input("trig")?, input.clone(), args.input("trigid")?];
    polls(builder, rate, &values, label.as_slice())?;
    Ok(input)
}

/// The `Poll`s of `trig`, `in` and `trigid`: one level of `multiNewList`,
/// then a `Poll` for each instance of each row. A row runs at `rate`, or for
/// `Poll.new` at the rate of its element of `in`, which sclang takes before
/// the expansion. The label of a row wraps over `labels`, and without one
/// each `Poll` gets the label of `Poll:new1`.
pub(super) fn polls(
    builder: &mut SynthDefBuilder,
    rate: Option<Rate>,
    values: &[UGenInput; 3],
    labels: &[String],
) -> Result<(), BuildError> {
    for (index, row) in elements(flop(values)).into_iter().enumerate() {
        let row = elements(row);
        let rate = rate.unwrap_or_else(|| lang::rate(&row[1]).unwrap_or(Rate::Scalar));
        let label = labels.get(index % labels.len().max(1)).map(String::as_str);
        builder.expand(
            "Poll",
            &["trig", "in", "trigid"],
            &row,
            true,
            &mut |builder, instance| new_poll(builder, rate, instance, label),
        )?;
    }
    Ok(())
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
