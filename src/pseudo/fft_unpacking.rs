//! The pseudo-UGens of `FFTUnpacking.sc`, and the methods of `PV_ChainUGen`
//! that take a function.

use super::{Args, elements, float, lang, multi_new, number};
use crate::builder::{SynthDefBuilder, count_to_usize};
use crate::catalog::{self, RateMethod};
use crate::env::flop;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::{ArgValue, UGenInput};

/// `UnpackFFT.new`: an `Unpack1FFT` of the magnitude, then of the phase, of
/// each bin from `frombin` to `tobin`, interleaved.
pub(super) fn unpack_fft(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let chain = args.value("chain")?;
    let bufsize = args.constant("bufsize")?;
    let (magnitudes, phases) = unpack(builder, args, &chain, bufsize)?;
    Ok(UGenInput::Array(
        magnitudes
            .into_iter()
            .zip(phases)
            .flat_map(|(magnitude, phase)| [magnitude, phase])
            .collect(),
    ))
}

/// The magnitudes and the phases of the bins of `chain` from `frombin` to
/// `tobin` of `args`, as `UnpackFFT` gives them. `tobin` is at most
/// `bufsize / 2`.
fn unpack(
    builder: &mut SynthDefBuilder,
    args: &Args,
    chain: &ArgValue,
    bufsize: f64,
) -> Result<(Vec<UGenInput>, Vec<UGenInput>), BuildError> {
    let frombin = args.constant("frombin")?;
    let upper = bufsize / 2.0;
    let tobin = match args.optional("tobin")? {
        Some(UGenInput::Number(tobin)) if tobin < upper => tobin,
        None | Some(UGenInput::Number(_)) => upper,
        Some(_) => return Err(args.wrong_kind("tobin", "a number")),
    };
    let bins = series(frombin, tobin);
    let unpack = |builder: &mut SynthDefBuilder, measure: f64| {
        builder.ugen(
            "Unpack1FFT",
            "new",
            &[
                ("chain", chain.clone()),
                ("bufsize", bufsize.into()),
                ("binindex", bins.clone().into()),
                ("whichmeasure", measure.into()),
            ],
        )
    };
    let magnitudes = elements(unpack(builder, 0.0)?);
    Ok((magnitudes, elements(unpack(builder, 1.0)?)))
}

/// `PackFFT.new`: the chain, `bufsize`, `frombin`, `tobin`, `zeroothers`,
/// the size of `magsphases` and each of its values, as `multiNewList`
/// expands them. A nil `tobin` is `bufsize / 2`.
pub(super) fn pack_fft(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let chain = args.input("chain")?;
    let bufsize = args.input("bufsize")?;
    let magsphases = args.array("magsphases")?;
    pack(builder, args, chain, bufsize, magsphases)
}

/// `PackFFT.new` of `magsphases` into `chain`, with `frombin`, `tobin` and
/// `zeroothers` of `args`.
fn pack(
    builder: &mut SynthDefBuilder,
    args: &Args,
    chain: UGenInput,
    bufsize: UGenInput,
    magsphases: Vec<UGenInput>,
) -> Result<UGenInput, BuildError> {
    let tobin = match args.optional("tobin")? {
        Some(tobin) => tobin,
        None => lang::binary(builder, "/", bufsize.clone(), 2.0.into())?,
    };
    let mut names = vec!["chain", "bufsize", "frombin", "tobin", "zeroothers"];
    let mut values = vec![
        chain,
        bufsize,
        args.input("frombin")?,
        tobin,
        args.input("zeroothers")?,
        number(magsphases.len()),
    ];
    names.resize(values.len() + magsphases.len(), "magsphases");
    values.extend(magsphases);
    multi_new(builder, "PackFFT", Rate::Control, &names, &values)
}

/// `PV_ChainUGen:pvcalc`: `func` of the magnitudes and of the phases of the
/// bins of `chain`, packed into `chain` again.
pub(crate) fn pvcalc(
    builder: &mut SynthDefBuilder,
    chain: UGenInput,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
    func: impl FnOnce(&mut SynthDefBuilder, UGenInput, UGenInput) -> Result<UGenInput, BuildError>,
) -> Result<UGenInput, BuildError> {
    let args = pv_args(builder, &chain, method, values)?;
    let numframes = args.constant("numframes")?;
    let (magnitudes, phases) = unpack(builder, &args, &chain.clone().into(), numframes)?;
    let result = func(
        builder,
        UGenInput::Array(magnitudes),
        UGenInput::Array(phases.clone()),
    )?;
    pack(
        builder,
        &args,
        chain,
        numframes.into(),
        switched(result, phases),
    )
}

/// `PV_ChainUGen:pvcalc2`: `func` of the magnitudes and of the phases of the
/// bins of `chain`, then of `chain2`, packed into `chain`.
pub(crate) fn pvcalc2(
    builder: &mut SynthDefBuilder,
    chain: UGenInput,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
    func: impl FnOnce(
        &mut SynthDefBuilder,
        UGenInput,
        UGenInput,
        UGenInput,
        UGenInput,
    ) -> Result<UGenInput, BuildError>,
) -> Result<UGenInput, BuildError> {
    let args = pv_args(builder, &chain, method, values)?;
    let chain2 = args.input("chain2")?;
    let numframes = args.constant("numframes")?;
    let (magnitudes, phases) = unpack(builder, &args, &chain.clone().into(), numframes)?;
    let (magnitudes2, phases2) = unpack(builder, &args, &chain2.into(), numframes)?;
    let result = func(
        builder,
        UGenInput::Array(magnitudes),
        UGenInput::Array(phases.clone()),
        UGenInput::Array(magnitudes2),
        UGenInput::Array(phases2),
    )?;
    pack(
        builder,
        &args,
        chain,
        numframes.into(),
        switched(result, phases),
    )
}

/// `PV_ChainUGen:pvcollect`: `func` of the magnitude, the phase, the bin and
/// the index of each bin of `chain`, packed into `chain` again. A result of
/// one value gets the phase of its bin.
pub(crate) fn pvcollect(
    builder: &mut SynthDefBuilder,
    chain: UGenInput,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
    mut func: impl FnMut(
        &mut SynthDefBuilder,
        UGenInput,
        UGenInput,
        f64,
        f64,
    ) -> Result<UGenInput, BuildError>,
) -> Result<UGenInput, BuildError> {
    let args = pv_args(builder, &chain, method, values)?;
    let numframes = args.constant("numframes")?;
    let frombin = args.constant("frombin")?;
    let (magnitudes, phases) = unpack(builder, &args, &chain.clone().into(), numframes)?;
    let mut magsphases = Vec::new();
    for (index, (magnitude, phase)) in magnitudes.into_iter().zip(phases).enumerate() {
        let index = float(index);
        let mut result = elements(func(
            builder,
            magnitude,
            phase.clone(),
            index + frombin,
            index,
        )?);
        if result.len() == 1 {
            result.push(phase);
        }
        magsphases.extend(result);
    }
    pack(builder, &args, chain, numframes.into(), magsphases)
}

/// The arguments of a method of `PV_ChainUGen`, whose receiver `chain` must
/// be one output of a `PV_ChainUGen`.
fn pv_args(
    builder: &SynthDefBuilder,
    chain: &UGenInput,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
) -> Result<Args, BuildError> {
    let args = Args {
        class: "PV_ChainUGen",
        method,
        values,
    };
    let is_chain = builder
        .class_of(chain)
        .and_then(catalog::lookup)
        .is_some_and(|class| class.superclasses.contains(&"PV_ChainUGen"));
    if is_chain {
        Ok(args)
    } else {
        Err(args.wrong_kind("chain", "one output of a PV_ChainUGen"))
    }
}

/// What `pvcalc` packs: the result of `func` as the switch on its size
/// takes it, then `flop` and `flatten` of one level. One value gets the
/// phases after it, two are the magnitudes and the phases, and any other
/// size, the 0 of a UGen included, is the magnitudes.
fn switched(result: UGenInput, phases: Vec<UGenInput>) -> Vec<UGenInput> {
    let rows = match result {
        UGenInput::Array(mut items) if items.len() == 1 => {
            items.extend(phases);
            items
        }
        UGenInput::Array(items) if items.len() == 2 => items,
        other => vec![other, UGenInput::Array(phases)],
    };
    elements(flop(&rows))
        .into_iter()
        .flat_map(elements)
        .collect()
}

/// `(first..last)`, as `prSimpleNumberSeries` gives it for Floats: steps of
/// 1 or -1 toward `last`.
fn series(first: f64, last: f64) -> UGenInput {
    let step = if first < last { 1.0 } else { -1.0 };
    let size = count_to_usize(((last - first) / step).trunc()) + 1;
    UGenInput::Array(
        (0..size)
            .map(|index| UGenInput::Number(first + step * float(index)))
            .collect(),
    )
}
