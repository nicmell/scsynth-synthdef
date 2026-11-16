//! The refusals of the methods of `PV_ChainUGen` that take a function. Each
//! is a graph that sclang 3.14.1 throws on, in `generator/fixtures.scd`.

use scsynth_synthdef::{ArgValue, BuildError, SynthDefBuilder, UGenInput};

/// `FFT(LocalBuf(frames), SinOsc.ar)`.
fn chain(builder: &mut SynthDefBuilder, frames: impl Into<UGenInput>) -> UGenInput {
    let buffer = builder
        .ugen("LocalBuf", "new", &[("numFrames", frames.into().into())])
        .unwrap();
    let sound = builder.ugen("SinOsc", "ar", &[]).unwrap();
    builder
        .ugen(
            "FFT",
            "new",
            &[("buffer", buffer.into()), ("in", sound.into())],
        )
        .unwrap()
}

/// `{ |m, p| [m, p] }`: the magnitudes and the phases as they come.
fn identity()
-> impl FnOnce(&mut SynthDefBuilder, UGenInput, UGenInput) -> Result<UGenInput, BuildError> {
    |_, magnitudes, phases| Ok(UGenInput::Array(vec![magnitudes, phases]))
}

/// The argument that a `WrongKind` names, or none for another error.
fn wrong(error: &BuildError) -> Option<&str> {
    match error {
        BuildError::WrongKind { argument, .. } => Some(argument),
        _ => None,
    }
}

/// `\name.kr(value)`.
fn control(builder: &mut SynthDefBuilder, name: &str, value: f64) -> UGenInput {
    builder
        .ugen(
            "NamedControl",
            "kr",
            &[("name", name.into()), ("values", value.into())],
        )
        .unwrap()
}

#[test]
fn pvcalc_refuses_numframes_as_an_output() {
    // refused_pvcalc_numframes.
    let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
    let chain = chain(&mut builder, 8.0);
    let frames = control(&mut builder, "n", 8.0);
    let error = builder
        .pvcalc(chain, &[("numframes", frames.into())], identity())
        .unwrap_err();
    assert_eq!(wrong(&error), Some("numframes"), "{error}");
}

#[test]
fn pvcollect_refuses_frombin_as_an_output() {
    // refused_pvcollect_frombin.
    let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
    let chain = chain(&mut builder, 8.0);
    let frombin = control(&mut builder, "f", 0.0);
    let args = [
        ("numframes", ArgValue::from(8.0)),
        ("frombin", frombin.into()),
    ];
    let error = builder
        .pvcollect(chain, &args, |_, magnitude, _, _, _| Ok(magnitude))
        .unwrap_err();
    assert_eq!(wrong(&error), Some("frombin"), "{error}");
}

#[test]
fn pvcalc_refuses_a_value_that_is_no_chain() {
    // refused_pvcalc_not_chain.
    let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
    let sound = builder.ugen("SinOsc", "ar", &[]).unwrap();
    let error = builder
        .pvcalc(sound, &[("numframes", 8.0.into())], identity())
        .unwrap_err();
    assert_eq!(wrong(&error), Some("chain"), "{error}");
}

#[test]
fn pvcalc_refuses_an_array_of_chains() {
    // refused_pvcalc_chains.
    let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
    let chains = chain(&mut builder, UGenInput::Array(vec![8.0.into(), 8.0.into()]));
    let error = builder
        .pvcalc(chains, &[("numframes", 8.0.into())], identity())
        .unwrap_err();
    assert_eq!(wrong(&error), Some("chain"), "{error}");
}

#[test]
fn pvcalc_takes_no_func_by_name() {
    let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
    let chain = chain(&mut builder, 8.0);
    let args = [("numframes", ArgValue::from(8.0)), ("func", 1.0.into())];
    let error = builder.pvcalc(chain, &args, identity()).unwrap_err();
    assert_eq!(wrong(&error), Some("func"), "{error}");
}
