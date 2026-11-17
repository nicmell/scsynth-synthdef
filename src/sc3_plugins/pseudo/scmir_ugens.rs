//! The pseudo-UGens of the plugin `SCMIRUGens`: `FeatureSave` and
//! `SpectralEntropy`.

use super::{multi_new, output_count};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, elements, number};
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `FeatureSave.kr`: `features.size.max(1)`, `trig` and the elements of
/// `features`. `size` is 0 for a value that is not an array, and a nil
/// `features` adds no element.
pub(super) fn feature_save(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let features = args.optional("features")?;
    let count = match &features {
        Some(UGenInput::Array(items)) => items.len().max(1),
        _ => 1,
    };
    let mut names = vec!["features", "trig"];
    let mut inputs = vec![number(count), args.input("trig")?];
    for feature in features.map(elements).unwrap_or_default() {
        names.push("features");
        inputs.push(feature);
    }
    multi_new(
        builder,
        ("FeatureSave", Rate::Control),
        &names,
        &inputs,
        &|_| Ok(1),
    )
}

/// `SpectralEntropy.kr`: `numbands` outputs, at most `fftsize`.
pub(super) fn spectral_entropy(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let fftsize = args.constant("fftsize")?;
    let mut numbands = args.constant("numbands")?;
    if numbands > fftsize {
        numbands = fftsize;
    }
    let count = output_count(args, "numbands", numbands, numbands)?;
    let inputs = [args.input("fft")?, fftsize.into(), numbands.into()];
    let names = ["fft", "fftsize", "numbands"];
    multi_new(
        builder,
        ("SpectralEntropy", Rate::Control),
        &names,
        &inputs,
        &|_| Ok(count),
    )
}
