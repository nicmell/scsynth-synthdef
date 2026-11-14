//! The pseudo-UGens of `FFTUnpacking.sc`.

use super::{Args, elements};
use crate::builder::{SynthDefBuilder, count_to_usize};
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `UnpackFFT.new`: an `Unpack1FFT` of the magnitude, then of the phase, of
/// each bin from `frombin` to `tobin`, interleaved.
pub(super) fn unpack_fft(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let chain = args.value("chain")?;
    let bufsize = args.constant("bufsize")?;
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
    let phases = elements(unpack(builder, 1.0)?);
    Ok(UGenInput::Array(
        magnitudes
            .into_iter()
            .zip(phases)
            .flat_map(|(magnitude, phase)| [magnitude, phase])
            .collect(),
    ))
}

/// `(first..last)`, as `prSimpleNumberSeries` gives it for Floats: steps of
/// 1 or -1 toward `last`.
fn series(first: f64, last: f64) -> UGenInput {
    let step = if first < last { 1.0 } else { -1.0 };
    let size = count_to_usize(((last - first) / step).trunc()) + 1;
    UGenInput::Array(
        (0..size)
            .map(|index| {
                UGenInput::Number(
                    first + step * f64::from(u32::try_from(index).unwrap_or(u32::MAX)),
                )
            })
            .collect(),
    )
}
