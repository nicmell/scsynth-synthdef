//! The pseudo-UGens of the plugin `MCLDUGens`: `PulseDPW`, the FFT classes,
//! `MatchingP`, `MatchingPResynth`, `NearestN`, `SOMRd`, `Getenv` and
//! `TextVU`.

use std::cmp::Ordering;

use super::{multi_new, output_count};
use crate::builder::{NewUGen, SynthDefBuilder, symbol_codes};
use crate::catalog;
use crate::error::BuildError;
use crate::pseudo::{Args, float, lang, number, sclang_class};
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `PulseDPW.ar`, `PulseDPW.kr` and `PulseDPW.new`: the difference of two
/// `SawDPW` at the rate `rate`, the second from `(width + width).wrap(-1,
/// 1)`, with `madd`.
pub(super) fn pulse_dpw(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = match args.selector() {
        "new" => args.text("rate")?.ok_or_else(|| args.missing("rate"))?,
        selector => selector.to_owned(),
    };
    let freq = args.value("freq")?;
    let start = [("freq", freq.clone()), ("iphase", 0.0.into())];
    let first = builder.ugen("SawDPW", &rate, &start)?;
    let width = args.input("width")?;
    let doubled = lang::binary(builder, "+", width.clone(), width)?;
    let bounds = [("lo", (-1.0).into()), ("hi", 1.0.into())];
    let phase = builder.send(doubled, "wrap", &bounds)?;
    let second = builder.ugen("SawDPW", &rate, &[("freq", freq), ("iphase", phase.into())])?;
    let pulse = lang::binary(builder, "-", first, second)?;
    builder.madd(pulse, args.input("mul")?, args.input("add")?)
}

/// `FFTCentroid.kr`: `SpecCentroid.kr` of `buffer`. sclang warns that the
/// class is deprecated.
pub(super) fn fft_centroid(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    builder.ugen("SpecCentroid", "kr", &[("buffer", args.value("buffer")?)])
}

/// `PV_DiffMags.new`: `PV_MagSubtract.new` of the two chains. sclang warns
/// of the new name.
pub(super) fn pv_diff_mags(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    builder.ugen(
        "PV_MagSubtract",
        "new",
        &[
            ("bufferA", args.value("bufferA")?),
            ("bufferB", args.value("bufferB")?),
        ],
    )
}

/// `FFTSubbandPower.kr`.
pub(super) fn fft_subband_power(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    subbands(builder, args, "FFTSubbandPower", &["square", "scalemode"])
}

/// `FFTSubbandFlatness.kr`.
pub(super) fn fft_subband_flatness(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    subbands(builder, args, "FFTSubbandFlatness", &[])
}

/// The `FFTSubband` classes: `chain`, the size of the sorted `cutfreqs`,
/// the arguments `others` and the sorted `cutfreqs`, for one output more
/// than `cutfreqs` has. `sort` fails on a value that is not an array, and
/// when it compares an output, in an array of two or more.
fn subbands(
    builder: &mut SynthDefBuilder,
    args: &Args,
    class: &str,
    others: &[&str],
) -> Result<UGenInput, BuildError> {
    let refused = || args.wrong_kind("cutfreqs", "an array of numbers");
    let UGenInput::Array(mut cutfreqs) = args.input("cutfreqs")? else {
        return Err(refused());
    };
    if cutfreqs.len() > 1 {
        let mut numbers = Vec::with_capacity(cutfreqs.len());
        for item in &cutfreqs {
            match *item {
                UGenInput::Number(freq) => numbers.push(freq),
                _ => return Err(refused()),
            }
        }
        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        cutfreqs = numbers.into_iter().map(UGenInput::Number).collect();
    }
    let mut names = vec!["chain", "cutfreqs"];
    let mut inputs = vec![args.input("chain")?, number(cutfreqs.len())];
    for &name in others {
        names.push(name);
        inputs.push(args.input(name)?);
    }
    let bands = cutfreqs.len() + 1;
    for freq in cutfreqs {
        names.push("cutfreqs");
        inputs.push(freq);
    }
    multi_new(builder, (class, Rate::Control), &names, &inputs, &|_| {
        Ok(bands)
    })
}

/// `MatchingP.ar` and `MatchingP.kr`: `2 + (ntofind * 2)` outputs, from the
/// `ntofind` of each instance.
pub(super) fn matching_p(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let names = ["dict", "in", "dictsize", "ntofind", "hop", "method"];
    let mut inputs = Vec::with_capacity(names.len());
    for name in names {
        inputs.push(args.input(name)?);
    }
    let rate = rate_of(args);
    multi_new(
        builder,
        ("MatchingP", rate),
        &names,
        &inputs,
        &|instance| match instance.get(3) {
            Some(&UGenInput::Number(ntofind)) => {
                output_count(args, "ntofind", ntofind, 2.0 + ntofind * 2.0)
            }
            _ => Err(args.wrong_kind("ntofind", "a number")),
        },
    )
}

/// `MatchingPResynth.ar` and `MatchingPResynth.kr`: `activs` after half of
/// its size, among the other inputs.
pub(super) fn matching_p_resynth(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let activs = args.array("activs")?;
    let mut names = vec!["dict", "method", "activs", "trigger", "residual"];
    let mut inputs = vec![
        args.input("dict")?,
        args.input("method")?,
        (float(activs.len()) / 2.0).into(),
        args.input("trigger")?,
        args.input("residual")?,
    ];
    for activ in activs {
        names.push("activs");
        inputs.push(activ);
    }
    let rate = rate_of(args);
    multi_new(
        builder,
        ("MatchingPResynth", rate),
        &names,
        &inputs,
        &|_| Ok(1),
    )
}

/// `NearestN.kr`: `num * 3` outputs, from the `num` of each instance, with
/// the elements of `in` last.
pub(super) fn nearest_n(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mut names = vec!["treebuf", "gate", "num"];
    let mut inputs = vec![
        args.input("treebuf")?,
        args.input("gate")?,
        args.input("num")?,
    ];
    for element in args.array("in")? {
        names.push("in");
        inputs.push(element);
    }
    multi_new(
        builder,
        ("NearestN", Rate::Control),
        &names,
        &inputs,
        &|instance| match instance.get(2) {
            Some(&UGenInput::Number(num)) => output_count(args, "num", num, num * 3.0),
            _ => Err(args.wrong_kind("num", "a number")),
        },
    )
}

/// `SOMRd.ar` and `SOMRd.kr`: `numdims` outputs, from 1 to 4, with the
/// elements of `inputdata` last.
pub(super) fn som_rd(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let numdims = args.constant("numdims")?;
    if !(1.0..=4.0).contains(&numdims) {
        return Err(args.wrong_kind("numdims", "a number from 1 to 4"));
    }
    let mut names = vec!["bufnum", "netsize", "numdims", "gate"];
    let mut inputs = vec![
        args.input("bufnum")?,
        args.input("netsize")?,
        numdims.into(),
        args.input("gate")?,
    ];
    for element in args.array("inputdata")? {
        names.push("inputdata");
        inputs.push(element);
    }
    let count = output_count(args, "numdims", numdims, numdims)?;
    let rate = rate_of(args);
    multi_new(builder, ("SOMRd", rate), &names, &inputs, &|_| Ok(count))
}

/// `Getenv.new`: a `Getenv` at scalar rate of the size of `key`, then
/// `defaultval`, then the codes of `key`, which is `SHLVL` when nil.
pub(super) fn getenv(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let key = args.text("key")?.unwrap_or_else(|| "SHLVL".to_owned());
    let codes = symbol_codes(&key);
    let mut names = vec!["key", "defaultval"];
    let mut inputs = vec![codes[0].clone(), args.input("defaultval")?];
    for code in &codes[1..] {
        names.push("key");
        inputs.push(code.clone());
    }
    multi_new(builder, ("Getenv", Rate::Scalar), &names, &inputs, &|_| {
        Ok(1)
    })
}

/// `TextVU.ar`, `TextVU.kr` and `TextVU.new`: `TextVU:new1` for each
/// instance, and the call gives back `in`.
pub(super) fn text_vu(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let input = args.input("in")?;
    let label = args.text("label")?;
    let mut names = vec!["trig", "in", "width", "reset"];
    let mut values = vec![
        args.input("trig")?,
        input.clone(),
        args.input("width")?,
        args.input("reset")?,
    ];
    if let Some(ana) = args.optional("ana")? {
        names.push("ana");
        values.push(ana);
    }
    let selector = args.selector();
    builder.expand("TextVU", &names, &values, true, &mut |builder, instance| {
        text_vu_new1(builder, selector, instance, label.as_deref())
    })?;
    Ok(input)
}

/// `TextVU:new1` of one instance of `trig`, `in`, `width`, `reset` and
/// `ana`. A nil label names the class of `in`. A number `trig` becomes an
/// `Impulse.kr`. Then the `TextVU` enters the graph, and only then does
/// `ana` give its input: a nil `ana` is an `Amplitude` of `in`, whose
/// release time is always 5, since `trig` is no longer a number.
fn text_vu_new1(
    builder: &mut SynthDefBuilder,
    selector: &str,
    instance: &[UGenInput],
    label: Option<&str>,
) -> Result<UGenInput, BuildError> {
    let (trig, input) = (&instance[0], &instance[1]);
    let rate = match selector {
        "ar" => Rate::Audio,
        "kr" => Rate::Control,
        _ => lang::rate(input).unwrap_or(Rate::Scalar),
    };
    let label = label.map_or_else(
        || format!("UGen({})", sclang_class(builder, input)),
        str::to_owned,
    );
    let trig = match *trig {
        UGenInput::Number(freq) => builder.ugen(
            "Impulse",
            "kr",
            &[("freq", freq.into()), ("phase", 0.0.into())],
        )?,
        ref other => other.clone(),
    };
    let mut inputs = vec![trig, 0.0.into(), instance[2].clone(), instance[3].clone()];
    inputs.extend(symbol_codes(&label));
    let class =
        catalog::find("TextVU").ok_or_else(|| BuildError::UnknownClass("TextVU".to_owned()))?;
    let text_vu =
        builder.add_checked(class, NewUGen::new("TextVU", Rate::Control).inputs(inputs))?;
    let analysis = match instance.get(4) {
        Some(ana) => ana.clone(),
        None if rate == Rate::Audio => {
            let release = [("in", input.clone().into()), ("releaseTime", 5.0.into())];
            let amplitude = builder.ugen("Amplitude", "ar", &release)?;
            builder.ugen("A2K", "kr", &[("in", amplitude.into())])?
        }
        None => {
            let release = [("in", input.clone().into()), ("releaseTime", 5.0.into())];
            builder.ugen("Amplitude", "kr", &release)?
        }
    };
    if let UGenInput::Output(proxy) = &text_vu {
        builder.set_input(proxy, 1, &analysis)?;
    }
    Ok(text_vu)
}

/// The rate of the selector of a method that is `ar` or `kr`.
fn rate_of(args: &Args) -> Rate {
    if args.selector() == "ar" {
        Rate::Audio
    } else {
        Rate::Control
    }
}
