//! The bodies of `InOut.sc`.

use super::ugen_methods::send_message;
use super::{Args, elements};
use crate::builder::{NewUGen, SynthDefBuilder};
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::{ArgValue, UGenInput};

/// `Control.kr`, `Control.ir`, `TrigControl.kr`, `TrigControl.ir` and
/// `AudioControl.ar`: the control UGen of the class at the rate of the
/// method.
pub(super) fn control(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let rate = match args.selector() {
        "ar" => Rate::Audio,
        "kr" => Rate::Control,
        _ => Rate::Scalar,
    };
    controls(builder, args.class, rate, &args.array("values")?)
}

/// `LagControl.kr`: a number lag serves each value.
pub(super) fn lag_control_kr(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    lag_controls(builder, &args.array("values")?, args.input("lags")?)
}

/// `LagControl.ar`: an `AudioControl` of the values, and its `lag`.
pub(super) fn lag_control_ar(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let control = controls(builder, "AudioControl", Rate::Audio, &args.array("values")?)?;
    lag(builder, control, args.input("lags")?)
}

/// The control UGens of `class` for `values`, as `multiNewList` expands
/// them. Each instance takes the next parameters.
pub(super) fn controls(
    builder: &mut SynthDefBuilder,
    class: &'static str,
    rate: Rate,
    values: &[UGenInput],
) -> Result<UGenInput, BuildError> {
    let names = vec!["values"; values.len()];
    builder.expand(class, &names, values, true, &mut |builder, instance| {
        let numbers = numbers(class, instance)?;
        builder.control_ugen(class, rate, &numbers, &[])
    })
}

/// The `LagControl`s of `values`, with one lag per value, or a number lag
/// for each: each group of 16 expands as `multiNewList` expands it. The
/// outputs of the groups follow one another in one array.
pub(super) fn lag_controls(
    builder: &mut SynthDefBuilder,
    values: &[UGenInput],
    lags: UGenInput,
) -> Result<UGenInput, BuildError> {
    let lags = match lags {
        UGenInput::Number(lag) => vec![UGenInput::Number(lag); values.len()],
        lags => elements(lags),
    };
    if values.len() != lags.len() {
        return Err(BuildError::BadControl {
            class: "LagControl".to_owned(),
            message: format!(
                "{} values and {} lags, where sclang gives nil",
                values.len(),
                lags.len()
            ),
        });
    }
    if values.is_empty() {
        return Err(empty("LagControl"));
    }
    let mut outputs = Vec::with_capacity(values.len());
    for (values, lags) in values.chunks(16).zip(lags.chunks(16)) {
        let mut names = vec!["values"; values.len()];
        names.resize(values.len() + lags.len(), "lags");
        let items: Vec<UGenInput> = values.iter().chain(lags).cloned().collect();
        let group = builder.expand(
            "LagControl",
            &names,
            &items,
            true,
            &mut |builder, instance| {
                let (values, lags) = instance.split_at(instance.len() / 2);
                let numbers = numbers("LagControl", values)?;
                builder.control_ugen("LagControl", Rate::Control, &numbers, lags)
            },
        )?;
        outputs.extend(elements(group));
    }
    Ok(UGenInput::Array(outputs))
}

/// `UGen:lag` with one lag time, as `send` sends it.
pub(super) fn lag(
    builder: &mut SynthDefBuilder,
    input: UGenInput,
    lags: UGenInput,
) -> Result<UGenInput, BuildError> {
    send_message(builder, input, "lag", vec![Some(ArgValue::from(lags))])
}

/// The numbers of an instance of control values: a control UGen takes no
/// output, and at least one value.
fn numbers(class: &str, values: &[UGenInput]) -> Result<Vec<f64>, BuildError> {
    if values.is_empty() {
        return Err(empty(class));
    }
    values
        .iter()
        .map(|value| match value {
            UGenInput::Number(number) => Ok(*number),
            _ => Err(BuildError::WrongKind {
                class: class.to_owned(),
                argument: "values".to_owned(),
                expected: "numbers",
            }),
        })
        .collect()
}

fn empty(class: &str) -> BuildError {
    BuildError::EmptyInput {
        ugen: class.to_owned(),
        argument: "values".to_owned(),
    }
}

/// `LocalIn.ar` and `LocalIn.kr`: `multiNew` takes the count and each
/// element of `default` apart. Each instance has as many outputs as its
/// count, and its defaults wrap to the count.
pub(super) fn local_in(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = if args.selector() == "ar" {
        Rate::Audio
    } else {
        Rate::Control
    };
    let defaults = args.array("default")?;
    let mut names = vec!["numChannels"];
    names.resize(defaults.len() + 1, "default");
    let mut values = vec![args.input("numChannels")?];
    values.extend(defaults);
    builder.expand(
        "LocalIn",
        &names,
        &values,
        true,
        &mut |builder, instance| {
            let channels = args.count_of("numChannels", &instance[0])?;
            let defaults = &instance[1..];
            if defaults.is_empty() {
                return Err(BuildError::EmptyInput {
                    ugen: "LocalIn".to_owned(),
                    argument: "default".to_owned(),
                });
            }
            let inputs: Vec<UGenInput> = (0..channels)
                .map(|channel| defaults[channel % defaults.len()].clone())
                .collect();
            builder.add(
                NewUGen::new("LocalIn", rate)
                    .inputs(inputs)
                    .outputs(vec![rate; channels]),
            )
        },
    )
}
