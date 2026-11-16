//! The pseudo-UGens of the plugin `DEINDUGens`: `Greyhole.sc` and
//! `JPverb.sc`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::Args;
use crate::ugen_input::UGenInput;

/// `Greyhole.ar`: `GreyholeRaw.ar` of the first and the last element of
/// `in`, with the other arguments in the order of `GreyholeRaw`.
pub(super) fn greyhole(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let (first, last) = ends(args, "Greyhole")?;
    builder.ugen(
        "GreyholeRaw",
        "ar",
        &[
            ("in1", first.into()),
            ("in2", last.into()),
            ("damping", args.value("damp")?),
            ("delaytime", args.value("delayTime")?),
            ("diffusion", args.value("diff")?),
            ("feedback", args.value("feedback")?),
            ("moddepth", args.value("modDepth")?),
            ("modfreq", args.value("modFreq")?),
            ("size", args.value("size")?),
        ],
    )
}

/// `JPverb.ar`: `JPverbRaw.ar` of the first and the last element of `in`,
/// with the other arguments in the order of `JPverbRaw`.
pub(super) fn jp_verb(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let (first, last) = ends(args, "JPverb")?;
    builder.ugen(
        "JPverbRaw",
        "ar",
        &[
            ("in1", first.into()),
            ("in2", last.into()),
            ("damp", args.value("damp")?),
            ("earlydiff", args.value("earlyDiff")?),
            ("highband", args.value("highcut")?),
            ("highx", args.value("high")?),
            ("lowband", args.value("lowcut")?),
            ("lowx", args.value("low")?),
            ("mdepth", args.value("modDepth")?),
            ("mfreq", args.value("modFreq")?),
            ("midx", args.value("mid")?),
            ("size", args.value("size")?),
            ("t60", args.value("t60")?),
        ],
    )
}

/// The first and the last element of `in`, as `in.asArray` gives them to
/// `first` and `last`.
fn ends(args: &Args, class: &str) -> Result<(UGenInput, UGenInput), BuildError> {
    let channels = args.array("in")?;
    match (channels.first(), channels.last()) {
        (Some(first), Some(last)) => Ok((first.clone(), last.clone())),
        _ => Err(BuildError::EmptyInput {
            ugen: class.to_owned(),
            argument: "in".to_owned(),
        }),
    }
}
