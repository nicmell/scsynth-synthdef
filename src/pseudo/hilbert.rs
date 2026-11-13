//! The pseudo-UGens of `Hilbert.sc`.

use super::Args;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `HilbertFIR.ar`: `in` delayed by the duration of `buffer`, and its phase
/// shifted by 90 degrees through `FFT`, `PV_PhaseShift90` and `IFFT`.
pub(super) fn hilbert_fir(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let input = args.value("in")?;
    let buffer = args.value("buffer")?;
    let chain = builder.ugen(
        "FFT",
        "new",
        &[("buffer", buffer.clone()), ("in", input.clone())],
    )?;
    let shifted = builder.ugen("PV_PhaseShift90", "new", &[("buffer", chain.into())])?;
    let delay = builder.ugen("BufDur", "kr", &[("bufnum", buffer)])?;
    let source = builder.ugen(
        "DelayN",
        "ar",
        &[
            ("in", input),
            ("maxdelaytime", delay.clone().into()),
            ("delaytime", delay.into()),
        ],
    )?;
    let quadrature = builder.ugen("IFFT", "new", &[("buffer", shifted.into())])?;
    Ok(UGenInput::Array(vec![source, quadrature]))
}
