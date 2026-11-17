//! `Ambisonics.sc`: `Rotate`, `Tilt` and `Tumble`, which make no instance
//! of their class, and the reverbs `BFFreeVerb` and `BFGVerb`.

use super::super::first;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// The factor of the angles of `Rotate`, `Tilt` and `Tumble`.
// The literal of `Ambisonics.sc`, 1/π to 14 places: a product of a number
// by `FRAC_1_PI` can round to another f64.
#[allow(clippy::approx_constant)]
const ANGLE: f64 = -0.318_309_886_183_79;

/// The channels of B-format, which `B2A` takes.
const FORMAT: [&str; 4] = ["w", "x", "y", "z"];

/// `Rotate.ar`: `w`, then `x` and `y` through `Rotate2`, then `z`.
pub(crate) fn rotate(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let [x, y] = rotate2(builder, args, ["x", "y", "rotate"])?;
    Ok(UGenInput::Array(vec![
        args.input("w")?,
        x,
        y,
        args.input("z")?,
    ]))
}

/// `Tilt.ar`: `w`, then `x` and `z` through `Rotate2`, with `y` between.
pub(crate) fn tilt(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let [x, z] = rotate2(builder, args, ["x", "z", "tilt"])?;
    Ok(UGenInput::Array(vec![
        args.input("w")?,
        x,
        args.input("y")?,
        z,
    ]))
}

/// `Tumble.ar`: `w` and `x`, then `y` and `z` through `Rotate2`, by the
/// angle `tilt`.
pub(crate) fn tumble(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let [y, z] = rotate2(builder, args, ["y", "z", "tilt"])?;
    Ok(UGenInput::Array(vec![
        args.input("w")?,
        args.input("x")?,
        y,
        z,
    ]))
}

/// `#a, b = Rotate2.ar(a, b, angle * -0.31830988618379)`, for the arguments
/// `names`: the two channels and the angle.
fn rotate2(
    builder: &mut SynthDefBuilder,
    args: &Args,
    names: [&str; 3],
) -> Result<[UGenInput; 2], BuildError> {
    let [x, y, angle] = names;
    let pos = lang::binary(builder, "*", args.input(angle)?, ANGLE.into())?;
    let rotated = builder.ugen(
        "Rotate2",
        "ar",
        &[
            ("x", args.value(x)?),
            ("y", args.value(y)?),
            ("pos", pos.into()),
        ],
    )?;
    first(
        args,
        &names,
        "a signal, or an array of two or more",
        rotated,
    )
}

/// `BFFreeVerb.ar`: a `FreeVerb` on each channel of `B2A`, each mixed with
/// two others, back through `A2B`.
pub(crate) fn bf_free_verb(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let channels = b2a(builder, args)?;
    let wet = builder.ugen(
        "FreeVerb",
        "ar",
        &[
            ("in", UGenInput::Array(Vec::from(channels)).into()),
            ("mix", args.value("mix")?),
            ("room", args.value("room")?),
            ("damp", args.value("damp")?),
            ("mul", args.value("mul")?),
            ("add", args.value("add")?),
        ],
    )?;
    let [a, b, c, d] = first(args, &FORMAT, "a signal, or an array of four or more", wet)?;
    let diffuse = args.input("diffuse")?;
    let a = diffused(builder, a, &b, &c, &diffuse)?;
    let b = diffused(builder, b, &c, &d, &diffuse)?;
    let c = diffused(builder, c, &d, &a, &diffuse)?;
    let d = diffused(builder, d, &a, &b, &diffuse)?;
    a2b(builder, [a, b, c, d])
}

/// `x + (y + z * diffuse)`.
fn diffused(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    y: &UGenInput,
    z: &UGenInput,
    diffuse: &UGenInput,
) -> Result<UGenInput, BuildError> {
    let sum = lang::binary(builder, "+", y.clone(), z.clone())?;
    let scaled = lang::binary(builder, "*", sum, diffuse.clone())?;
    lang::binary(builder, "+", x, scaled)
}

/// The arguments of `BFGVerb.ar` that reach `GVerb`.
const GVERB: [&str; 9] = [
    "roomsize",
    "revtime",
    "damping",
    "inputbw",
    "drylevel",
    "earlyreflevel",
    "taillevel",
    "maxroomsize",
    "mul",
];

/// `BFGVerb.ar`: a `GVerb` on each channel of `B2A`, after a `DelayN` of a
/// `Rand`, mixed with the others, back through `A2B`, plus `add`.
pub(crate) fn bf_g_verb(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let channels = b2a(builder, args)?;
    let mut times = Vec::with_capacity(4);
    for _ in 0..4 {
        let range = [("lo", 0.0.into()), ("hi", 0.001.into())];
        times.push(builder.ugen("Rand", "new", &range)?);
    }
    let delayed = builder.ugen(
        "DelayN",
        "ar",
        &[
            ("in", UGenInput::Array(Vec::from(channels)).into()),
            ("maxdelaytime", 0.001.into()),
            ("delaytime", UGenInput::Array(times).into()),
        ],
    )?;
    let [a, b, c, d] = first(
        args,
        &FORMAT,
        "a signal, or an array of four or more",
        delayed,
    )?;
    let [ar, al] = g_verb(builder, args, a)?;
    let [br, bl] = g_verb(builder, args, b)?;
    let [cr, cl] = g_verb(builder, args, c)?;
    let [dr, dl] = g_verb(builder, args, d)?;
    let diffuse = args.input("diffuse")?;
    let a = spread(builder, [&ar, &al, &br, &cl], &diffuse)?;
    let b = spread(builder, [&br, &bl, &cr, &dl], &diffuse)?;
    let c = spread(builder, [&cr, &cl, &dr, &al], &diffuse)?;
    let d = spread(builder, [&dr, &dl, &ar, &bl], &diffuse)?;
    let format = a2b(builder, [a, b, c, d])?;
    lang::binary(builder, "+", format, args.input("add")?)
}

/// `#right, left = GVerb.ar(channel, ...)`, with the arguments of
/// `BFGVerb.ar` and a `spread` of 45.
fn g_verb(
    builder: &mut SynthDefBuilder,
    args: &Args,
    channel: UGenInput,
) -> Result<[UGenInput; 2], BuildError> {
    let mut values = vec![("in", channel.into()), ("spread", 45.0.into())];
    for name in GVERB {
        values.push((name, args.value(name)?));
    }
    let verb = builder.ugen("GVerb", "ar", &values)?;
    let names: Vec<&str> = FORMAT.into_iter().chain(GVERB).collect();
    first(args, &names, "a signal, or an array of two or more", verb)
}

/// `w + x + (y + z * diffuse)`.
fn spread(
    builder: &mut SynthDefBuilder,
    [w, x, y, z]: [&UGenInput; 4],
    diffuse: &UGenInput,
) -> Result<UGenInput, BuildError> {
    let sum = lang::binary(builder, "+", w.clone(), x.clone())?;
    let other = lang::binary(builder, "+", y.clone(), z.clone())?;
    let scaled = lang::binary(builder, "*", other, diffuse.clone())?;
    lang::binary(builder, "+", sum, scaled)
}

/// `#a, b, c, d = B2A.ar(w, x, y, z)`.
fn b2a(builder: &mut SynthDefBuilder, args: &Args) -> Result<[UGenInput; 4], BuildError> {
    let mut values = Vec::with_capacity(4);
    for name in FORMAT {
        values.push((name, args.value(name)?));
    }
    let channels = builder.ugen("B2A", "ar", &values)?;
    first(
        args,
        &FORMAT,
        "a signal, or an array of four or more",
        channels,
    )
}

/// `A2B.ar(a, b, c, d)`.
fn a2b(
    builder: &mut SynthDefBuilder,
    [a, b, c, d]: [UGenInput; 4],
) -> Result<UGenInput, BuildError> {
    builder.ugen(
        "A2B",
        "ar",
        &[
            ("a", a.into()),
            ("b", b.into()),
            ("c", c.into()),
            ("d", d.into()),
        ],
    )
}
