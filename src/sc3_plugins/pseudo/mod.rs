//! The pseudo-UGens of sc3-plugins: a body for each rate method that the
//! catalogue of sc3-plugins marks `Addable::Pseudo`, with the `Args` of the
//! bodies of the core.

mod bhob_ugens;
mod deind_ugens;
mod josh_ugens;
mod nh_ugens;

use josh_ugens::{
    ambisonics, biquad_eq, feedback, free_verb1, joshfilts, lpc_synth, pan_x, phaser, sc2_compat,
    uhj, vocode, vocoder, warp_overlap,
};

use crate::builder::{NewUGen, SynthDefBuilder, count_to_usize};
use crate::catalog;
use crate::error::BuildError;
use crate::pseudo::{Args, Body, elements};
use crate::rate::Rate;
use crate::ugen_input::{ArgValue, UGenInput};

/// The body of each method that the catalogue of sc3-plugins marks
/// `Addable::Pseudo`, by class and selector.
pub(crate) static BODIES: &[(&str, &str, Body)] = &[
    ("BFFreeVerb", "ar", ambisonics::bf_free_verb),
    ("BFGVerb", "ar", ambisonics::bf_g_verb),
    ("BHPF", "ar", joshfilts::bhpf),
    ("BLPF", "ar", joshfilts::blpf),
    ("BhobHiShelf", "ar", bhob_ugens::hi_shelf),
    ("BhobLoShelf", "ar", bhob_ugens::lo_shelf),
    ("BtoUHJ", "ar", uhj::b_to_uhj),
    ("DelayWr", "ar", sc2_compat::delay_wr),
    ("Feedback", "ar", feedback::feedback),
    ("FreeVerb1", "ar", free_verb1::free_verb1),
    ("Greyhole", "ar", deind_ugens::greyhole),
    ("HighShelf", "ar", biquad_eq::high_shelf),
    ("JPverb", "ar", deind_ugens::jp_verb),
    ("LPCSynth", "ar", lpc_synth::lpc_synth),
    ("LPCVals", "ar", lpc_synth::lpc_vals),
    ("LPCVals", "kr", lpc_synth::lpc_vals),
    ("LRHPF", "ar", joshfilts::lrhpf),
    ("LRLPF", "ar", joshfilts::lrlpf),
    ("LowShelf", "ar", biquad_eq::low_shelf),
    ("NHHall", "ar", nh_ugens::nh_hall),
    ("Notch", "ar", biquad_eq::notch),
    ("PanX2D", "ar", pan_x::pan_x2d),
    ("PanX2D", "kr", pan_x::pan_x2d),
    ("Phaser", "ar", phaser::phaser),
    ("Phaser", "kr", phaser::phaser),
    ("RBJHighShelf", "ar", biquad_eq::high_shelf),
    ("RBJLowShelf", "ar", biquad_eq::low_shelf),
    ("RBJNotch", "ar", biquad_eq::notch),
    ("Rotate", "ar", ambisonics::rotate),
    ("TapN", "ar", sc2_compat::tap_n),
    ("Tilt", "ar", ambisonics::tilt),
    ("Tumble", "ar", ambisonics::tumble),
    ("UHJtoB", "ar", uhj::uhj_to_b),
    ("Vocode", "ar", vocode::vocode),
    ("VocodeBand", "ar", vocode::vocode_band),
    ("Vocoder", "ar", vocoder::vocoder),
    ("WarpOverlap", "ar", warp_overlap::warp_overlap),
    ("ZHPF", "ar", joshfilts::zhpf),
    ("ZLPF", "ar", joshfilts::zlpf),
];

/// `#a, b, ... = value` of sclang: the first `N` elements of `value`, the
/// result of an expansion over the arguments `names`. Past the end of an
/// array, sclang gives nil, which a later call throws on. The crate refuses
/// the first of `names` that holds an array, with `expected`.
fn first<const N: usize>(
    args: &Args,
    names: &[&str],
    expected: &'static str,
    value: UGenInput,
) -> Result<[UGenInput; N], BuildError> {
    let mut items = elements(value);
    items.truncate(N);
    <[UGenInput; N]>::try_from(items).map_err(|_| {
        let array = names
            .iter()
            .copied()
            .find(|name| matches!(args.value(name), Ok(ArgValue::Input(UGenInput::Array(_)))));
        let name = array.or_else(|| names.first().copied());
        args.wrong_kind(name.unwrap_or_default(), expected)
    })
}

/// `multiNew` of `class` at `rate`: the expansion of `multiNewList`, then
/// an instance for each, with `outputs` outputs at `rate`, after the rule of
/// its `checkInputs`.
fn multi_new(
    builder: &mut SynthDefBuilder,
    (class, rate): (&str, Rate),
    names: &[&str],
    inputs: &[UGenInput],
    outputs: usize,
) -> Result<UGenInput, BuildError> {
    let ugen = catalog::find(class).ok_or_else(|| BuildError::UnknownClass(class.to_owned()))?;
    builder.expand(class, names, inputs, true, &mut |builder, instance| {
        let new = NewUGen::new(class, rate)
            .inputs(instance.to_vec())
            .outputs(vec![rate; outputs]);
        builder.add_checked(ugen, new)
    })
}

/// An array of numbers, as an array literal of sclang gives it.
fn numbers(values: &[f64]) -> UGenInput {
    UGenInput::Array(values.iter().map(|&value| value.into()).collect())
}

/// The count of iterations of `count.do` in sclang, as for `Array.fill`: the
/// floor of a Float, and none below 1. The crate refuses a count above
/// 65535 for the argument `name`.
fn do_count(args: &Args, name: &str, count: f64) -> Result<usize, BuildError> {
    if count.is_nan() || count < 1.0 {
        return Ok(0);
    }
    if count > f64::from(u16::MAX) {
        return Err(args.wrong_kind(name, "a number up to 65535"));
    }
    Ok(count_to_usize(count.floor()))
}

#[cfg(test)]
mod tests {
    use super::BODIES;
    use crate::builder::SynthDefBuilder;
    use crate::catalog::{Addable, RateMethod};
    use crate::error::BuildError;
    use crate::sc3_plugins::{all, pseudo_all};
    use crate::ugen_input::UGenInput;

    /// A builder with a sine at audio rate.
    fn sine() -> (SynthDefBuilder, UGenInput) {
        let mut builder = SynthDefBuilder::new("reverb", vec![]).unwrap();
        let sine = builder.ugen("SinOsc", "ar", &[]).unwrap();
        (builder, sine)
    }

    #[test]
    fn greyhole_and_jp_verb_read_the_first_and_the_last_channel() {
        for (class, raw) in [("Greyhole", "GreyholeRaw"), ("JPverb", "JPverbRaw")] {
            let (mut builder, sine) = sine();
            builder.ugen(class, "ar", &[("in", sine.into())]).unwrap();
            let def = builder.finish().unwrap();
            let ugen = def.children.iter().find(|ugen| ugen.name == raw).unwrap();
            assert_eq!(ugen.inputs[0], ugen.inputs[1], "{class}");
            assert_eq!(ugen.outputs.len(), 2, "{class}");
        }
    }

    #[test]
    fn nh_hall_takes_two_channels_and_gives_two() {
        let (mut builder, sine) = sine();
        let error = builder
            .ugen("NHHall", "ar", &[("in", sine.clone().into())])
            .unwrap_err();
        assert!(matches!(error, BuildError::WrongKind { ref argument, .. } if argument == "in"));
        let stereo = UGenInput::Array(vec![sine.clone(), sine]);
        let hall = builder
            .ugen("NHHall", "ar", &[("in", stereo.into())])
            .unwrap();
        assert!(matches!(hall, UGenInput::Array(ref outputs) if outputs.len() == 2));
    }

    #[test]
    fn each_pseudo_method_has_a_body_and_each_body_its_method() {
        let classes: Vec<(&str, &[RateMethod])> = all()
            .iter()
            .map(|class| (class.name, class.methods))
            .chain(pseudo_all().iter().map(|class| (class.name, class.methods)))
            .collect();
        let mut methods: Vec<(&str, &str)> = classes
            .iter()
            .flat_map(|&(class, methods)| {
                methods
                    .iter()
                    .filter(|method| method.addable == Addable::Pseudo)
                    .map(move |method| (class, method.selector))
            })
            .collect();
        methods.sort_unstable();
        let mut bodies: Vec<(&str, &str)> = BODIES
            .iter()
            .map(|&(class, selector, _)| (class, selector))
            .collect();
        bodies.sort_unstable();
        assert_eq!(methods, bodies);
    }
}
