//! The instance methods of `UGen` that build graphs, such as `range`, and the
//! methods that a number or an array answers in their place. A body sends the
//! messages that its method sends in sclang, in the same order.

use super::{Args, lang, sclang_class};
use crate::builder::{NewUGen, SynthDefBuilder, bind_optional};
use crate::catalog::{self, SignalRange, UGenMethod};
use crate::check_inputs;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::{ArgValue, UGenInput};

/// What one method adds for its receiver.
type Body = fn(&mut SynthDefBuilder, UGenInput, &Args) -> Result<UGenInput, BuildError>;

/// The body of each method, by the class that owns it and its selector.
static BODIES: &[(&str, &str, Body)] = &[
    ("DUGen", "expexp", demand_expexp),
    ("DUGen", "explin", explin),
    ("DUGen", "linexp", demand_linexp),
    ("DUGen", "linlin", demand_linlin),
    ("Float", "clip", number_clip),
    ("Float", "fold", number_fold),
    ("Float", "wrap", number_wrap),
    ("LFGauss", "range", lf_gauss_range),
    ("SequenceableCollection", "bilin", multi_channel_perform),
    ("SequenceableCollection", "bipolar", multi_channel_perform),
    ("SequenceableCollection", "clip", multi_channel_perform),
    ("SequenceableCollection", "curvelin", multi_channel_perform),
    (
        "SequenceableCollection",
        "curverange",
        multi_channel_perform,
    ),
    ("SequenceableCollection", "expexp", multi_channel_perform),
    ("SequenceableCollection", "explin", multi_channel_perform),
    ("SequenceableCollection", "exprange", multi_channel_perform),
    ("SequenceableCollection", "fold", multi_channel_perform),
    ("SequenceableCollection", "lincurve", multi_channel_perform),
    ("SequenceableCollection", "linexp", multi_channel_perform),
    ("SequenceableCollection", "linlin", multi_channel_perform),
    ("SequenceableCollection", "prune", multi_channel_perform),
    ("SequenceableCollection", "range", multi_channel_perform),
    ("SequenceableCollection", "unipolar", multi_channel_perform),
    ("SequenceableCollection", "wrap", multi_channel_perform),
    ("SimpleNumber", "bilin", number_bilin),
    ("SimpleNumber", "curvelin", number_curvelin),
    ("SimpleNumber", "expexp", number_expexp),
    ("SimpleNumber", "explin", number_explin),
    ("SimpleNumber", "lincurve", number_lincurve),
    ("SimpleNumber", "linexp", number_linexp),
    ("SimpleNumber", "linlin", number_linlin),
    ("UGen", "bilin", bilin),
    ("UGen", "bipolar", bipolar),
    ("UGen", "clip", clip),
    ("UGen", "curvelin", curvelin),
    ("UGen", "curverange", curverange),
    ("UGen", "expexp", expexp),
    ("UGen", "explin", explin),
    ("UGen", "exprange", exprange),
    ("UGen", "fold", fold),
    ("UGen", "lincurve", lincurve),
    ("UGen", "linexp", linexp),
    ("UGen", "linlin", linlin),
    ("UGen", "prune", prune),
    ("UGen", "range", range),
    ("UGen", "unipolar", unipolar),
    ("UGen", "wrap", wrap),
];

/// Sends the message of `method` to `receiver`, with the values of its
/// arguments in their order, to the method that the receiver answers: that
/// of `Float` for a number, of `Array` for an array, and of the class of its
/// UGen for an output.
pub(crate) fn send(
    builder: &mut SynthDefBuilder,
    receiver: UGenInput,
    method: &'static UGenMethod,
    values: Vec<Option<ArgValue>>,
) -> Result<UGenInput, BuildError> {
    let selector = method.method.selector;
    let owner = match &receiver {
        UGenInput::Number(_) => method
            .number
            .ok_or_else(|| bad_method(selector, "a Float does not understand the message"))?,
        UGenInput::Array(items) if items.is_empty() => {
            return Err(bad_method(selector, "the receiver is an empty array"));
        }
        UGenInput::Array(_) => method
            .array
            .ok_or_else(|| bad_method(selector, "an Array does not understand the message"))?,
        UGenInput::Output(_) => owner_of(builder, &receiver, method),
    };
    let body = BODIES
        .iter()
        .find(|&&(class, name, _)| class == owner && name == selector)
        .map(|&(_, _, body)| body)
        .ok_or_else(|| BuildError::NotAddable {
            class: owner.to_owned(),
            selector: selector.to_owned(),
            reason: "the crate has no body for the method".to_owned(),
        })?;
    let args = Args {
        class: owner,
        method: &method.method,
        values,
    };
    body(builder, receiver, &args)
}

/// Sends `selector` to `receiver` as a method of sclang sends it: `given`
/// holds the values of the first arguments, and the others take the
/// defaults of `UGen`.
fn send_message(
    builder: &mut SynthDefBuilder,
    receiver: UGenInput,
    selector: &str,
    given: Vec<Option<ArgValue>>,
) -> Result<UGenInput, BuildError> {
    let method = catalog::ugen_method(selector)
        .ok_or_else(|| bad_method(selector, "it is no method of UGen that builds a graph"))?;
    let mut values = bind_optional("UGen", &method.method, &[])?;
    for (slot, value) in values.iter_mut().zip(given) {
        *slot = value;
    }
    send(builder, receiver, method, values)
}

/// The class whose method an output answers: the first of the class of its
/// UGen and its superclasses that has a method of its own, else `UGen`.
fn owner_of(builder: &SynthDefBuilder, receiver: &UGenInput, method: &UGenMethod) -> &'static str {
    catalog::lookup(&sclang_class(builder, receiver))
        .and_then(|class| {
            std::iter::once(class.name)
                .chain(class.superclasses.iter().copied())
                .find_map(|name| {
                    method
                        .overrides
                        .iter()
                        .copied()
                        .find(|&owner| owner == name)
                })
        })
        .unwrap_or("UGen")
}

/// `signalRange` of an output: that of the class of its UGen. An output of a
/// `MultiOutUGen` is an `OutputProxy`, which is bipolar.
fn signal_range(builder: &SynthDefBuilder, receiver: &UGenInput) -> SignalRange {
    catalog::lookup(&sclang_class(builder, receiver))
        .map_or(SignalRange::Bipolar, |class| class.signal_range)
}

fn bad_method(selector: &str, message: &str) -> BuildError {
    BuildError::BadMethod {
        selector: selector.to_owned(),
        message: message.to_owned(),
    }
}

fn rate_of(value: &UGenInput) -> Rate {
    lang::rate(value).unwrap_or(Rate::Scalar)
}

/// The number of a receiver that is a number.
fn number_of(receiver: &UGenInput, selector: &str) -> Result<f64, BuildError> {
    match *receiver {
        UGenInput::Number(number) => Ok(number),
        _ => Err(bad_method(selector, "the receiver is not a number")),
    }
}

/// `inMin`, `inMax`, `outMin` and `outMax` of `linlin` and its kin.
fn map_args(args: &Args) -> Result<[UGenInput; 4], BuildError> {
    Ok([
        args.input("inMin")?,
        args.input("inMax")?,
        args.input("outMin")?,
        args.input("outMax")?,
    ])
}

/// `this.prune(inMin, inMax, clip)`.
fn prune_to(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    in_min: &UGenInput,
    in_max: &UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let clip = args.text("clip")?.map(ArgValue::Text);
    let given = vec![
        Some(ArgValue::from(in_min.clone())),
        Some(ArgValue::from(in_max.clone())),
        clip,
    ];
    send_message(builder, this, "prune", given)
}

/// `multiNew` of `class` at `rate`: the expansion of `multiNewList`, then a
/// UGen of one output for each instance, with the rule of its `checkInputs`.
fn multi_new(
    builder: &mut SynthDefBuilder,
    class: &str,
    rate: Rate,
    names: &[&str],
    inputs: &[UGenInput],
) -> Result<UGenInput, BuildError> {
    let ugen = catalog::lookup(class).ok_or_else(|| BuildError::UnknownClass(class.to_owned()))?;
    builder.expand(class, names, inputs, true, &mut |builder, instance| {
        check_inputs::check(ugen.check_inputs, rate, instance).map_err(|message| {
            BuildError::CheckInputs {
                class: class.to_owned(),
                message,
            }
        })?;
        builder.add(
            NewUGen::new(class, rate)
                .inputs(instance.to_vec())
                .width_first(ugen.is_width_first),
        )
    })
}

/// The `methodSelectorForRate` of the receiver for a `Select`. At scalar or
/// demand rate it names a method that `Select` does not have as a UGen, and
/// sclang writes the definition without its `Out`.
fn select_selector(this: &UGenInput, selector: &str) -> Result<&'static str, BuildError> {
    match rate_of(this) {
        Rate::Audio => Ok("ar"),
        Rate::Control => Ok("kr"),
        rate => Err(bad_method(
            selector,
            &format!("Select has no method for the rate {rate:?}, so sclang loses the Out"),
        )),
    }
}

/// -1 for a bipolar receiver and 0 for a unipolar one: the low end of its
/// range for `exprange` and `curverange`.
fn low_end(builder: &SynthDefBuilder, this: &UGenInput) -> UGenInput {
    match signal_range(builder, this) {
        SignalRange::Bipolar => (-1.0).into(),
        SignalRange::Unipolar => 0.0.into(),
    }
}

/// `UGen:range`: a `MulAdd` from the `signalRange` of the receiver to `lo`
/// and `hi`.
fn range(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let (lo, hi) = (args.input("lo")?, args.input("hi")?);
    let span = lang::binary(builder, "-", hi, lo.clone())?;
    let (mul, add) = match signal_range(builder, &this) {
        SignalRange::Bipolar => {
            let mul = lang::binary(builder, "*", span, 0.5.into())?;
            let add = lang::binary(builder, "+", mul.clone(), lo)?;
            (mul, add)
        }
        SignalRange::Unipolar => (span, lo),
    };
    builder.madd(this, mul, add)
}

/// `LFGauss:range`: `linlin` from the least value of its width, which
/// `minval` computes from the input `width` of the `LFGauss`.
fn lf_gauss_range(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let width = builder
        .input_of(&this, 1)
        .ok_or_else(|| bad_method("range", "the LFGauss has no width"))?;
    let squared = lang::unary(builder, "squared", width)?;
    let scaled = lang::binary(builder, "*", (-2.0).into(), squared)?;
    let exponent = lang::binary(builder, "/", 1.0.into(), scaled)?;
    let minval = lang::unary(builder, "exp", exponent)?;
    let given = vec![
        Some(ArgValue::from(minval)),
        Some(ArgValue::from(1.0)),
        Some(args.value("lo")?),
        Some(args.value("hi")?),
    ];
    send_message(builder, this, "linlin", given)
}

/// `UGen:exprange`: `linexp` from the range of the receiver, without a clip.
fn exprange(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let low = low_end(builder, &this);
    let given = vec![
        Some(ArgValue::from(low)),
        Some(ArgValue::from(1.0)),
        Some(args.value("lo")?),
        Some(args.value("hi")?),
        None,
    ];
    send_message(builder, this, "linexp", given)
}

/// `UGen:curverange`: `lincurve` from the range of the receiver, without a
/// clip.
fn curverange(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let low = low_end(builder, &this);
    let given = vec![
        Some(ArgValue::from(low)),
        Some(ArgValue::from(1.0)),
        Some(args.value("lo")?),
        Some(args.value("hi")?),
        Some(args.value("curve")?),
        None,
    ];
    send_message(builder, this, "lincurve", given)
}

/// `UGen:unipolar`: `range(0, mul)`.
fn unipolar(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let given = vec![Some(ArgValue::from(0.0)), Some(args.value("mul")?)];
    send_message(builder, this, "range", given)
}

/// `UGen:bipolar`: `range(mul.neg, mul)`.
fn bipolar(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mul = args.input("mul")?;
    let low = lang::unary(builder, "neg", mul.clone())?;
    send_message(
        builder,
        this,
        "range",
        vec![Some(ArgValue::from(low)), Some(ArgValue::from(mul))],
    )
}

/// `UGen:prune`: `clip` for the type `minmax`, `max` for `min`, `min` for
/// `max`, and the receiver for any other type.
fn prune(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    match args.text("type")?.as_deref() {
        Some("minmax") => {
            let given = vec![
                args.optional("min")?.map(ArgValue::Input),
                args.optional("max")?.map(ArgValue::Input),
            ];
            send_message(builder, this, "clip", given)
        }
        Some("min") => lang::binary(builder, "max", this, args.input("min")?),
        Some("max") => lang::binary(builder, "min", this, args.input("max")?),
        _ => Ok(this),
    }
}

/// `UGen:linlin`: `LinLin.ar` for a receiver at audio rate, else
/// `LinLin.kr`, of the receiver pruned.
fn linlin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let selector = if rate_of(&this) == Rate::Audio {
        "ar"
    } else {
        "kr"
    };
    let pruned = prune_to(builder, this, &in_min, &in_max, args)?;
    builder.ugen(
        "LinLin",
        selector,
        &[
            ("in", pruned.into()),
            ("srclo", in_min.into()),
            ("srchi", in_max.into()),
            ("dstlo", out_min.into()),
            ("dsthi", out_max.into()),
        ],
    )
}

/// `UGen:linexp`: a `LinExp` at the rate of the receiver, of the receiver
/// pruned.
fn linexp(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let rate = rate_of(&this);
    let pruned = prune_to(builder, this, &in_min, &in_max, args)?;
    multi_new(
        builder,
        "LinExp",
        rate,
        &["in", "srclo", "srchi", "dstlo", "dsthi"],
        &[pruned, in_min, in_max, out_min, out_max],
    )
}

/// `UGen:explin` and `DUGen:explin`: the arithmetic of
/// [`logarithmic`] on the receiver pruned.
fn explin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let pruned = prune_to(builder, this, &in_min, &in_max, args)?;
    logarithmic(builder, pruned, [in_min, in_max, out_min, out_max])
}

/// `UGen:expexp`: the arithmetic of [`exponential_of_log`] on the receiver
/// pruned.
fn expexp(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    exponential_of_log(
        builder,
        [in_min.clone(), in_max.clone(), out_min, out_max],
        |builder| prune_to(builder, this, &in_min, &in_max, args),
    )
}

/// `UGen:lincurve`: `linlin` for a number curve under 0.125, the arithmetic
/// of [`curved`] for a curve at scalar rate, and else a `Select` of the two
/// by `abs(curve) >= 0.125`.
fn lincurve(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let curve = args.input("curve")?;
    if matches!(curve, UGenInput::Number(number) if number.abs() < 0.125) {
        return send_linlin(builder, this, args);
    }
    let ranges = [in_min.clone(), in_max.clone(), out_min, out_max];
    let result = curved(builder, curve.clone(), ranges, |builder| {
        prune_to(builder, this.clone(), &in_min, &in_max, args)
    })?;
    curve_select(builder, this, curve, result, args)
}

/// `UGen:curvelin`: as [`lincurve`], with the arithmetic of
/// [`curved_back`].
fn curvelin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let curve = args.input("curve")?;
    if matches!(curve, UGenInput::Number(number) if number.abs() < 0.125) {
        return send_linlin(builder, this, args);
    }
    let ranges = [in_min.clone(), in_max.clone(), out_min, out_max];
    let result = curved_back(builder, curve.clone(), ranges, |builder| {
        prune_to(builder, this.clone(), &in_min, &in_max, args)
    })?;
    curve_select(builder, this, curve, result, args)
}

/// `this.linlin(inMin, inMax, outMin, outMax, clip)`, with the arguments of
/// `lincurve` or `curvelin`.
fn send_linlin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let clip = args.text("clip")?.map(ArgValue::Text);
    let given = vec![
        Some(args.value("inMin")?),
        Some(args.value("inMax")?),
        Some(args.value("outMin")?),
        Some(args.value("outMax")?),
        clip,
    ];
    send_message(builder, this, "linlin", given)
}

/// The end of `lincurve` and `curvelin`: `result` for a curve at scalar
/// rate, else a `Select` of `linlin` and `result` by `abs(curve) >= 0.125`.
fn curve_select(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    curve: UGenInput,
    result: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if rate_of(&curve) == Rate::Scalar {
        return Ok(result);
    }
    let selector = select_selector(&this, args.selector())?;
    let magnitude = lang::unary(builder, "abs", curve)?;
    let which = lang::binary(builder, ">=", magnitude, 0.125.into())?;
    let linear = send_linlin(builder, this, args)?;
    builder.ugen(
        "Select",
        selector,
        &[
            ("which", which.into()),
            ("array", UGenInput::Array(vec![linear, result]).into()),
        ],
    )
}

/// `UGen:bilin`: a `Select` by `this < inCenter` of the `linlin` above the
/// center and the `linlin` below it.
fn bilin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let selector = select_selector(&this, "bilin")?;
    let in_center = args.input("inCenter")?;
    let which = lang::binary(builder, "<", this.clone(), in_center.clone())?;
    let clip = args.text("clip")?.map(ArgValue::Text);
    let above = vec![
        Some(ArgValue::from(in_center.clone())),
        Some(args.value("inMax")?),
        Some(args.value("outCenter")?),
        Some(args.value("outMax")?),
        clip.clone(),
    ];
    let above = send_message(builder, this.clone(), "linlin", above)?;
    let below = vec![
        Some(args.value("inMin")?),
        Some(ArgValue::from(in_center)),
        Some(args.value("outMin")?),
        Some(args.value("outCenter")?),
        clip,
    ];
    let below = send_message(builder, this, "linlin", below)?;
    builder.ugen(
        "Select",
        selector,
        &[
            ("which", which.into()),
            ("array", UGenInput::Array(vec![above, below]).into()),
        ],
    )
}

/// `UGen:clip`: a `Clip` by the `methodSelectorForRate` of `Clip`, and
/// `max(lo, min(hi, this))` at demand rate.
fn clip(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    lang::clip(builder, this, args.input("lo")?, args.input("hi")?)
}

/// `UGen:fold`: a `Fold` by the `methodSelectorForRate` of `Fold`.
fn fold(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    fold_or_wrap(builder, "Fold", this, args)
}

/// `UGen:wrap`: a `Wrap` by the `methodSelectorForRate` of `Wrap`.
fn wrap(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    fold_or_wrap(builder, "Wrap", this, args)
}

/// `fold` and `wrap`, which sclang does not implement at demand rate.
fn fold_or_wrap(
    builder: &mut SynthDefBuilder,
    class: &str,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = rate_of(&this);
    if rate == Rate::Demand {
        return Err(bad_method(
            args.selector(),
            "NotYetImplementedError at demand rate",
        ));
    }
    builder.ugen(
        class,
        lang::selector_for_rate(rate),
        &[
            ("in", this.into()),
            ("lo", args.value("lo")?),
            ("hi", args.value("hi")?),
        ],
    )
}

/// `DUGen:linlin`: the arithmetic of [`linear`] on the receiver pruned.
fn demand_linlin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let pruned = prune_to(builder, this, &in_min, &in_max, args)?;
    linear(builder, pruned, [in_min, in_max, out_min, out_max])
}

/// `DUGen:linexp`: the arithmetic of [`exponential`] on the receiver, then
/// `prune` of the result to the output range.
fn demand_linexp(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let result = exponential(
        builder,
        this,
        [in_min, in_max, out_min.clone(), out_max.clone()],
    )?;
    prune_to(builder, result, &out_min, &out_max, args)
}

/// `DUGen:expexp`:
/// `pow(outMax / outMin, log(pruned / log(inMax / inMin)) * outMin)`. The
/// type of its `prune` is `clip / inMin`, which keeps a symbol and throws on
/// nil.
fn demand_expexp(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [in_min, in_max, out_min, out_max] = map_args(args)?;
    let base = lang::binary(builder, "/", out_max, out_min.clone())?;
    let clip = args
        .text("clip")?
        .ok_or_else(|| bad_method("expexp", "DUGen:expexp divides nil by inMin"))?;
    let given = vec![
        Some(ArgValue::from(in_min.clone())),
        Some(ArgValue::from(in_max.clone())),
        Some(ArgValue::Text(clip)),
    ];
    let pruned = send_message(builder, this, "prune", given)?;
    let range = lang::binary(builder, "/", in_max, in_min)?;
    let log_range = lang::unary(builder, "log", range)?;
    let ratio = lang::binary(builder, "/", pruned, log_range)?;
    let log = lang::unary(builder, "log", ratio)?;
    let exponent = lang::binary(builder, "*", log, out_min)?;
    lang::binary(builder, "pow", base, exponent)
}

/// `Float:clip`: `sc_clip`.
fn number_clip(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    lang::clip(builder, this, args.input("lo")?, args.input("hi")?)
}

/// `Float:fold`: `sc_fold`.
fn number_fold(
    _builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [x, lo, hi] = primitive_numbers(this, args, "_FoldFloat")?;
    Ok(lang::fold(x, lo, hi).into())
}

/// `Float:wrap`.
fn number_wrap(
    _builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [x, lo, hi] = primitive_numbers(this, args, "_WrapFloat")?;
    Ok(lang::wrap(x, lo, hi).into())
}

/// The receiver, `lo` and `hi` of a primitive of `Float`, which fails on an
/// argument that is not a number.
fn primitive_numbers(
    this: UGenInput,
    args: &Args,
    primitive: &str,
) -> Result<[f64; 3], BuildError> {
    match (this, args.input("lo")?, args.input("hi")?) {
        (UGenInput::Number(x), UGenInput::Number(lo), UGenInput::Number(hi)) => Ok([x, lo, hi]),
        _ => Err(bad_method(
            args.selector(),
            &format!("Primitive '{primitive}' failed. Wrong type."),
        )),
    }
}

/// The clip of `SimpleNumber:linlin` and its kin, by its type: `outMin` at or
/// below `inMin`, and `outMax` at or above `inMax`.
fn clipped(this: f64, args: &Args) -> Result<Option<UGenInput>, BuildError> {
    let (low, high) = match args.text("clip")?.as_deref() {
        Some("minmax") => (true, true),
        Some("min") => (true, false),
        Some("max") => (false, true),
        _ => (false, false),
    };
    let selector = args.selector();
    if low && compared(selector, this, &args.input("inMin")?, |x, bound| x <= bound)? {
        return args.input("outMin").map(Some);
    }
    if high && compared(selector, this, &args.input("inMax")?, |x, bound| x >= bound)? {
        return args.input("outMax").map(Some);
    }
    Ok(None)
}

/// The test of an `if` that compares a number with `that`, which must be a
/// number: sclang throws "Non Boolean in test." on the UGen that a
/// comparison with an output gives.
fn compared(
    selector: &str,
    this: f64,
    that: &UGenInput,
    compare: fn(f64, f64) -> bool,
) -> Result<bool, BuildError> {
    match *that {
        UGenInput::Number(that) => Ok(compare(this, that)),
        _ => Err(bad_method(selector, "Non Boolean in test.")),
    }
}

/// `SimpleNumber:linlin`: [`linear`], after the clip.
fn number_linlin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if let Some(bound) = clipped(number_of(&this, "linlin")?, args)? {
        return Ok(bound);
    }
    linear(builder, this, map_args(args)?)
}

/// `SimpleNumber:linexp`: [`exponential`], after the clip.
fn number_linexp(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if let Some(bound) = clipped(number_of(&this, "linexp")?, args)? {
        return Ok(bound);
    }
    exponential(builder, this, map_args(args)?)
}

/// `SimpleNumber:explin`: [`logarithmic`], after the clip.
fn number_explin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if let Some(bound) = clipped(number_of(&this, "explin")?, args)? {
        return Ok(bound);
    }
    logarithmic(builder, this, map_args(args)?)
}

/// `SimpleNumber:expexp`: [`exponential_of_log`], after the clip.
fn number_expexp(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if let Some(bound) = clipped(number_of(&this, "expexp")?, args)? {
        return Ok(bound);
    }
    exponential_of_log(builder, map_args(args)?, |_| Ok(this))
}

/// `SimpleNumber:lincurve`: after the clip, [`linear`] for a curve under
/// 0.001, else [`curved`].
fn number_lincurve(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if let Some(bound) = clipped(number_of(&this, "lincurve")?, args)? {
        return Ok(bound);
    }
    let curve = args.input("curve")?;
    if small_curve(&curve, "lincurve")? {
        return linear(builder, this, map_args(args)?);
    }
    curved(builder, curve, map_args(args)?, |_| Ok(this))
}

/// `SimpleNumber:curvelin`: after the clip, [`linear`] for a curve under
/// 0.001, else [`curved_back`].
fn number_curvelin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    if let Some(bound) = clipped(number_of(&this, "curvelin")?, args)? {
        return Ok(bound);
    }
    let curve = args.input("curve")?;
    if small_curve(&curve, "curvelin")? {
        return linear(builder, this, map_args(args)?);
    }
    curved_back(builder, curve, map_args(args)?, |_| Ok(this))
}

/// `abs(curve) < 0.001`, the test of `SimpleNumber:lincurve` and
/// `curvelin`.
fn small_curve(curve: &UGenInput, selector: &str) -> Result<bool, BuildError> {
    match *curve {
        UGenInput::Number(curve) => Ok(curve.abs() < 0.001),
        _ => Err(bad_method(selector, "Non Boolean in test.")),
    }
}

/// `SimpleNumber:bilin`: after the clip, `linlin` without a clip above
/// `inCenter` or below it.
fn number_bilin(
    builder: &mut SynthDefBuilder,
    this: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let x = number_of(&this, "bilin")?;
    if let Some(bound) = clipped(x, args)? {
        return Ok(bound);
    }
    let in_center = args.input("inCenter")?;
    let names = if compared("bilin", x, &in_center, |x, center| x >= center)? {
        ["inCenter", "inMax", "outCenter", "outMax"]
    } else {
        ["inMin", "inCenter", "outMin", "outCenter"]
    };
    let mut given = names
        .iter()
        .map(|name| args.value(name).map(Some))
        .collect::<Result<Vec<_>, _>>()?;
    given.push(Some(ArgValue::from("none")));
    send_message(builder, this, "linlin", given)
}

/// `(x - inMin) / (inMax - inMin) * (outMax - outMin) + outMin`.
fn linear(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    [in_min, in_max, out_min, out_max]: [UGenInput; 4],
) -> Result<UGenInput, BuildError> {
    let shifted = lang::binary(builder, "-", x, in_min.clone())?;
    let width = lang::binary(builder, "-", in_max, in_min)?;
    let scaled = lang::binary(builder, "/", shifted, width)?;
    let span = lang::binary(builder, "-", out_max, out_min.clone())?;
    let product = lang::binary(builder, "*", scaled, span)?;
    lang::binary(builder, "+", product, out_min)
}

/// `pow(outMax / outMin, (x - inMin) / (inMax - inMin)) * outMin`.
fn exponential(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    [in_min, in_max, out_min, out_max]: [UGenInput; 4],
) -> Result<UGenInput, BuildError> {
    let base = lang::binary(builder, "/", out_max, out_min.clone())?;
    let shifted = lang::binary(builder, "-", x, in_min.clone())?;
    let width = lang::binary(builder, "-", in_max, in_min)?;
    let exponent = lang::binary(builder, "/", shifted, width)?;
    let power = lang::binary(builder, "pow", base, exponent)?;
    lang::binary(builder, "*", power, out_min)
}

/// `log(x / inMin) / log(inMax / inMin) * (outMax - outMin) + outMin`.
fn logarithmic(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    [in_min, in_max, out_min, out_max]: [UGenInput; 4],
) -> Result<UGenInput, BuildError> {
    let ratio = lang::binary(builder, "/", x, in_min.clone())?;
    let log = lang::unary(builder, "log", ratio)?;
    let range = lang::binary(builder, "/", in_max, in_min)?;
    let log_range = lang::unary(builder, "log", range)?;
    let scaled = lang::binary(builder, "/", log, log_range)?;
    let span = lang::binary(builder, "-", out_max, out_min.clone())?;
    let product = lang::binary(builder, "*", scaled, span)?;
    lang::binary(builder, "+", product, out_min)
}

/// `pow(outMax / outMin, log(x / inMin) / log(inMax / inMin)) * outMin`,
/// where `x` gives its value after `outMax / outMin`.
fn exponential_of_log(
    builder: &mut SynthDefBuilder,
    [in_min, in_max, out_min, out_max]: [UGenInput; 4],
    x: impl FnOnce(&mut SynthDefBuilder) -> Result<UGenInput, BuildError>,
) -> Result<UGenInput, BuildError> {
    let base = lang::binary(builder, "/", out_max, out_min.clone())?;
    let x = x(builder)?;
    let ratio = lang::binary(builder, "/", x, in_min.clone())?;
    let log = lang::unary(builder, "log", ratio)?;
    let range = lang::binary(builder, "/", in_max, in_min)?;
    let log_range = lang::unary(builder, "log", range)?;
    let exponent = lang::binary(builder, "/", log, log_range)?;
    let power = lang::binary(builder, "pow", base, exponent)?;
    lang::binary(builder, "*", power, out_min)
}

/// `b - (a * pow(grow, scaled))` of `lincurve`, with `grow = exp(curve)`,
/// `a = (outMax - outMin) / (1 - grow)` and `b = outMin + a`. `scaled` maps
/// the value that `x` gives after them from the input range to 0 to 1.
fn curved(
    builder: &mut SynthDefBuilder,
    curve: UGenInput,
    [in_min, in_max, out_min, out_max]: [UGenInput; 4],
    x: impl FnOnce(&mut SynthDefBuilder) -> Result<UGenInput, BuildError>,
) -> Result<UGenInput, BuildError> {
    let grow = lang::unary(builder, "exp", curve)?;
    let span = lang::binary(builder, "-", out_max, out_min.clone())?;
    let denominator = lang::binary(builder, "-", 1.0.into(), grow.clone())?;
    let a = lang::binary(builder, "/", span, denominator)?;
    let b = lang::binary(builder, "+", out_min, a.clone())?;
    let x = x(builder)?;
    let shifted = lang::binary(builder, "-", x, in_min.clone())?;
    let width = lang::binary(builder, "-", in_max, in_min)?;
    let scaled = lang::binary(builder, "/", shifted, width)?;
    let power = lang::binary(builder, "pow", grow, scaled)?;
    let product = lang::binary(builder, "*", a, power)?;
    lang::binary(builder, "-", b, product)
}

/// `log((b - x) / a) * (outMax - outMin) / curve + outMin` of `curvelin`,
/// with `grow = exp(curve)`, `a = (inMax - inMin) / (1 - grow)` and
/// `b = inMin + a`. `x` gives its value after them.
fn curved_back(
    builder: &mut SynthDefBuilder,
    curve: UGenInput,
    [in_min, in_max, out_min, out_max]: [UGenInput; 4],
    x: impl FnOnce(&mut SynthDefBuilder) -> Result<UGenInput, BuildError>,
) -> Result<UGenInput, BuildError> {
    let grow = lang::unary(builder, "exp", curve.clone())?;
    let span = lang::binary(builder, "-", in_max, in_min.clone())?;
    let denominator = lang::binary(builder, "-", 1.0.into(), grow)?;
    let a = lang::binary(builder, "/", span, denominator)?;
    let b = lang::binary(builder, "+", in_min, a.clone())?;
    let x = x(builder)?;
    let gap = lang::binary(builder, "-", b, x)?;
    let ratio = lang::binary(builder, "/", gap, a)?;
    let log = lang::unary(builder, "log", ratio)?;
    let out_span = lang::binary(builder, "-", out_max, out_min.clone())?;
    let product = lang::binary(builder, "*", log, out_span)?;
    let quotient = lang::binary(builder, "/", product, curve)?;
    lang::binary(builder, "+", quotient, out_min)
}

/// `SequenceableCollection:multiChannelPerform`: a `flop` of the receiver
/// and the arguments, then the message to each row. An array wraps, and an
/// empty one gives nil.
fn multi_channel_perform(
    builder: &mut SynthDefBuilder,
    receiver: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mut columns = vec![Some(ArgValue::Input(receiver))];
    columns.extend(args.values.iter().cloned());
    let size = columns
        .iter()
        .map(|value| match value {
            Some(ArgValue::Input(UGenInput::Array(items))) => items.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(1);
    let mut results = Vec::with_capacity(size);
    for row in 0..size {
        let mut values: Vec<Option<ArgValue>> = columns
            .iter()
            .map(|value| match value {
                Some(ArgValue::Input(UGenInput::Array(items))) => {
                    (!items.is_empty()).then(|| ArgValue::Input(items[row % items.len()].clone()))
                }
                other => other.clone(),
            })
            .collect();
        let Some(ArgValue::Input(element)) = values.remove(0) else {
            return Err(bad_method(
                args.selector(),
                "nil does not understand the message",
            ));
        };
        results.push(send_message(builder, element, args.selector(), values)?);
    }
    Ok(UGenInput::Array(results))
}
