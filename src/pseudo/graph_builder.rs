//! The bodies of `GraphBuilder.sc`: `NamedControl`.

use super::{Args, in_out};
use crate::builder::SynthDefBuilder;
use crate::catalog;
use crate::error::BuildError;
use crate::rate::{Rate, least_by_name};
use crate::ugen_input::UGenInput;

/// A name of the table of `NamedControl` of a build, as `NamedControl.new`
/// keeps it: its values, its lags, its rate, whether its lags are fixed,
/// and its control.
#[derive(Clone, Debug)]
pub(crate) struct NamedEntry {
    name: String,
    values: Values,
    lags: Option<Lags>,
    rate: Option<String>,
    fixed_lag: bool,
    control: UGenInput,
}

/// Numbers in nested arrays, as the values of a `NamedControl`.
#[derive(Clone, Debug, PartialEq)]
enum Values {
    Number(f64),
    Array(Vec<Values>),
}

impl Values {
    fn of(input: &UGenInput) -> Result<Self, BuildError> {
        match input {
            UGenInput::Number(number) => Ok(Self::Number(*number)),
            UGenInput::Array(items) => items
                .iter()
                .map(Self::of)
                .collect::<Result<_, _>>()
                .map(Self::Array),
            UGenInput::Output(_) => Err(BuildError::WrongKind {
                class: CLASS.to_owned(),
                argument: "values".to_owned(),
                expected: "numbers",
            }),
        }
    }

    /// `asArray`.
    fn into_array(self) -> Self {
        match self {
            Self::Number(_) => Self::Array(vec![self]),
            array @ Self::Array(_) => array,
        }
    }

    /// The numbers in order, as `flat` gives them.
    fn flat(&self, numbers: &mut Vec<UGenInput>) {
        match self {
            Self::Number(number) => numbers.push(UGenInput::Number(*number)),
            Self::Array(items) => items.iter().for_each(|item| item.flat(numbers)),
        }
    }

    /// The size of the default of the name, `values.unbubble`.
    fn default_size(&self) -> usize {
        match self {
            Self::Array(items) => match items.as_slice() {
                [only] => only.size(),
                _ => items.len(),
            },
            Self::Number(_) => 1,
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Number(_) => 1,
            Self::Array(items) => items.len(),
        }
    }
}

/// The lags of a `NamedControl` after its `deepCollect`, which makes each 0
/// nil.
#[derive(Clone, Debug, PartialEq)]
enum Lags {
    Nil,
    Input(UGenInput),
    Array(Vec<Lags>),
}

impl Lags {
    fn of(input: &UGenInput) -> Self {
        match input {
            UGenInput::Number(number) if *number == 0.0 => Self::Nil,
            UGenInput::Array(items) => Self::Array(items.iter().map(Self::of).collect()),
            other => Self::Input(other.clone()),
        }
    }

    /// `lags.rate`: none for nil, the rate of the element of an array of
    /// one, and else the least rate by name, where nil counts as scalar.
    fn rate(&self) -> Option<Rate> {
        match self {
            Self::Nil => None,
            Self::Input(input) => input.rate(),
            Self::Array(items) => match items.as_slice() {
                [only] => only.rate(),
                _ => least_by_name(items.iter().map(|item| item.rate().unwrap_or(Rate::Scalar))),
            },
        }
    }

    /// The lags as an input of a `LagControl` or a `Lag`, which refuse nil.
    fn input(&self, name: &str) -> Result<UGenInput, BuildError> {
        match self {
            Self::Nil => Err(bad(format!(
                "the lags of {name} hold a 0, which sclang makes nil"
            ))),
            Self::Input(input) => Ok(input.clone()),
            Self::Array(items) => items
                .iter()
                .map(|item| item.input(name))
                .collect::<Result<_, _>>()
                .map(UGenInput::Array),
        }
    }
}

const CLASS: &str = "NamedControl";

/// `NamedControl.ar`, `kr`, `ir`, `tr` and `new`: `NamedControl.new`, with
/// the table of the build.
pub(super) fn named_control(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = match args.selector() {
        "ar" => Some("audio".to_owned()),
        "kr" => Some("control".to_owned()),
        "ir" => Some("scalar".to_owned()),
        "tr" => Some("trigger".to_owned()),
        _ => args.text("rate")?,
    };
    let name = args
        .text("name")?
        .ok_or_else(|| BuildError::MissingArgument {
            class: CLASS.to_owned(),
            argument: "name".to_owned(),
        })?;
    let fixed_lag = matches!(args.selector(), "kr" | "new") && args.boolean("fixedLag")?;
    let mut values = args
        .optional("values")?
        .as_ref()
        .map(Values::of)
        .transpose()?;
    // sclang reads the default of the spec only for no values, so an
    // unknown spec fails only then.
    if let (None, Some(spec)) = (&values, args.text("spec")?) {
        let spec = catalog::spec(&spec).ok_or_else(|| bad(format!("no spec is named {spec}")))?;
        values = Some(Values::Number(spec.default));
    }
    let lags = args
        .optional("lags")?
        .as_ref()
        .map(Lags::of)
        .filter(|lags| *lags != Lags::Nil);
    let fixed_lag = fixed_lag || lags.as_ref().and_then(Lags::rate) == Some(Rate::Scalar);
    let entry = entry(builder, &name, values, lags.as_ref(), rate, fixed_lag)?;
    match lags {
        Some(lags) if entry.fixed_lag => {
            if entry.lags.as_ref() != Some(&lags) {
                return Err(bad(format!("the name {name} takes other fixed lags")));
            }
            Ok(entry.control)
        }
        Some(lags) => {
            let lags = lags.input(&name)?;
            Ok(unbubble(in_out::lag(builder, entry.control, lags)?))
        }
        None => Ok(entry.control),
    }
}

/// The entry of `name` in the table: the one there, whose values and rate
/// must agree, or a new one with its control.
fn entry(
    builder: &mut SynthDefBuilder,
    name: &str,
    values: Option<Values>,
    lags: Option<&Lags>,
    rate: Option<String>,
    fixed_lag: bool,
) -> Result<NamedEntry, BuildError> {
    if let Some(entry) = builder
        .named_controls()
        .iter()
        .find(|entry| entry.name == name)
    {
        let entry = entry.clone();
        let values = values.map_or_else(|| entry.values.clone(), Values::into_array);
        if entry.values != values {
            return Err(bad(format!("the name {name} takes other default values")));
        }
        if rate.is_some() && entry.rate != rate {
            return Err(bad(format!("the name {name} takes another rate")));
        }
        return Ok(entry);
    }
    let values = values.unwrap_or(Values::Number(0.0)).into_array();
    let control = init(builder, name, &values, lags, rate.as_deref(), fixed_lag)?;
    let entry = NamedEntry {
        name: name.to_owned(),
        values,
        lags: lags.cloned(),
        rate,
        fixed_lag,
        control,
    };
    builder.push_named_control(entry.clone());
    Ok(entry)
}

/// `NamedControl:init`: the name, then a `LagControl` for fixed lags on a
/// name without a prefix, or else the control of the rate of the prefix or
/// of `rate`. The control takes the values flat, and gives back their
/// shape.
fn init(
    builder: &mut SynthDefBuilder,
    name: &str,
    values: &Values,
    lags: Option<&Lags>,
    rate: Option<&str>,
    fixed_lag: bool,
) -> Result<UGenInput, BuildError> {
    let prefix = match name.as_bytes() {
        [first, b'_', ..] => Some(*first),
        _ => None,
    };
    let mut numbers = Vec::new();
    values.flat(&mut numbers);
    if numbers.is_empty() {
        return Err(BuildError::EmptyInput {
            ugen: CLASS.to_owned(),
            argument: "values".to_owned(),
        });
    }
    builder.add_control_name(name, values.default_size());
    let control = match lags {
        Some(lags) if fixed_lag && prefix.is_none() => fixed(builder, name, &numbers, lags, rate)?,
        _ if prefix == Some(b'a') || rate == Some("audio") => {
            in_out::controls(builder, "AudioControl", Rate::Audio, &numbers)?
        }
        _ if prefix == Some(b't') || rate == Some("trigger") => {
            in_out::controls(builder, "TrigControl", Rate::Control, &numbers)?
        }
        _ if prefix == Some(b'i') || rate == Some("scalar") => {
            in_out::controls(builder, "Control", Rate::Scalar, &numbers)?
        }
        _ => in_out::controls(builder, "Control", Rate::Control, &numbers)?,
    };
    Ok(unbubble(reshape_like(control, values)))
}

/// The control of fixed lags: `LagControl.ar` at audio rate, else
/// `LagControl.kr`, which repeats a number lag for each value.
fn fixed(
    builder: &mut SynthDefBuilder,
    name: &str,
    numbers: &[UGenInput],
    lags: &Lags,
    rate: Option<&str>,
) -> Result<UGenInput, BuildError> {
    if rate == Some("audio") {
        let control = in_out::controls(builder, "AudioControl", Rate::Audio, numbers)?;
        return in_out::lag(builder, control, lags.input(name)?);
    }
    let lags = match lags {
        Lags::Input(UGenInput::Number(lag)) => vec![UGenInput::Number(*lag); numbers.len()],
        Lags::Array(items) => items
            .iter()
            .map(|lag| lag.input(name))
            .collect::<Result<_, _>>()?,
        other => vec![other.input(name)?],
    };
    in_out::lag_controls(builder, numbers, &lags)
}

/// `control.asArray.reshapeLike(values)`: the outputs in order, in the
/// shape of the values.
fn reshape_like(control: UGenInput, values: &Values) -> UGenInput {
    fn flatten(input: UGenInput, outputs: &mut Vec<UGenInput>) {
        match input {
            UGenInput::Array(items) => items.into_iter().for_each(|item| flatten(item, outputs)),
            output => outputs.push(output),
        }
    }
    fn shape(values: &Values, outputs: &[UGenInput], next: &mut usize) -> UGenInput {
        match values {
            Values::Number(_) => {
                let output = outputs.get(*next % outputs.len().max(1)).cloned();
                *next += 1;
                output.unwrap_or_else(|| UGenInput::Array(Vec::new()))
            }
            Values::Array(items) => UGenInput::Array(
                items
                    .iter()
                    .map(|item| shape(item, outputs, next))
                    .collect(),
            ),
        }
    }
    let mut outputs = Vec::new();
    flatten(control, &mut outputs);
    shape(values, &outputs, &mut 0)
}

/// `unbubble`: the element of an array of one.
fn unbubble(input: UGenInput) -> UGenInput {
    match input {
        UGenInput::Array(mut items) if items.len() == 1 => items.remove(0),
        other => other,
    }
}

fn bad(message: String) -> BuildError {
    BuildError::BadControl {
        class: CLASS.to_owned(),
        message,
    }
}
