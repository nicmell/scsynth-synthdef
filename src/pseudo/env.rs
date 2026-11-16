//! The constructors of `Env.sc`: a body for each class method of `Env` that
//! makes an envelope, the ranges of an envelope, and the array of
//! `asArrayForInterpolation`.

use super::{Args, elements, lang, number};
use crate::builder::SynthDefBuilder;
use crate::catalog::RateMethod;
use crate::env::{Curve, Env, bad, curve_value, flop, shape_number};
use crate::error::BuildError;
use crate::ugen_input::{ArgValue, UGenInput};

/// -100.dbamp of sclang 3.14.1, the last level of `Env.cutoff` at `\exp`:
/// the bits of `10 ** (-100 * 0.05)` (measured).
const MINUS_100_DB: f64 = 1e-5;

/// What a class method of `Env` makes.
type Body = fn(&mut SynthDefBuilder, &Args) -> Result<Env, BuildError>;

/// The body of each class method of `Env` that makes an envelope.
static BODIES: &[(&str, Body)] = &[
    ("adsr", adsr),
    ("asr", asr),
    ("circle", circle),
    ("cutoff", cutoff),
    ("dadsr", dadsr),
    ("linen", linen),
    ("new", new),
    ("newClear", new_clear),
    ("pairs", pairs),
    ("perc", perc),
    ("sine", sine),
    ("step", step),
    ("triangle", triangle),
    ("xyc", xyc),
];

/// `Env:range`, `exprange` and `curverange`: a copy of the envelope, with its
/// levels mapped by `linlin`, `linexp` or `lincurve` from `levels.minItem`
/// to `levels.maxItem`.
pub(crate) fn range(
    builder: &mut SynthDefBuilder,
    env: &Env,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
) -> Result<Env, BuildError> {
    let name = format!("Env:{}", method.selector);
    let low = extreme(&env.levels, &name, |level, least| level < least)?;
    let high = extreme(&env.levels, &name, |level, greatest| level > greatest)?;
    let map = match method.selector {
        "range" => "linlin",
        "exprange" => "linexp",
        _ => "lincurve",
    };
    let mut given = vec![Some(ArgValue::from(low)), Some(ArgValue::from(high))];
    given.extend(values);
    let levels = UGenInput::Array(env.levels.clone());
    let mapped = super::ugen_methods::send_message(builder, levels, map, given)?;
    Ok(Env {
        levels: elements(mapped),
        ..env.clone()
    })
}

/// `minItem` or `maxItem` of the levels: the first level that no later one
/// beats. sclang throws "Non Boolean in test." on a comparison that is not of
/// two numbers.
fn extreme(
    levels: &[UGenInput],
    name: &str,
    beats: fn(f64, f64) -> bool,
) -> Result<UGenInput, BuildError> {
    let mut levels = levels.iter();
    let mut best = levels
        .next()
        .cloned()
        .ok_or_else(|| bad(name, "the envelope has no levels"))?;
    for level in levels {
        match (level, &best) {
            (&UGenInput::Number(level), &UGenInput::Number(current)) => {
                if beats(level, current) {
                    best = UGenInput::Number(level);
                }
            }
            _ => return Err(bad(name, "Non Boolean in test.")),
        }
    }
    Ok(best)
}

/// Runs the body of the class method `method` of `Env` on its bound
/// arguments.
pub(crate) fn build(
    builder: &mut SynthDefBuilder,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
) -> Result<Env, BuildError> {
    let body = BODIES
        .iter()
        .find(|(selector, _)| *selector == method.selector)
        .map(|&(_, body)| body)
        .ok_or_else(|| BuildError::NotAddable {
            class: "Env".to_owned(),
            selector: method.selector.to_owned(),
            reason: "the crate has no body for the method".to_owned(),
        })?;
    body(
        builder,
        &Args {
            class: "Env",
            method,
            values,
        },
    )
}

/// `asArrayForInterpolation`: the array of each channel, as `IEnvGen` reads
/// it.
pub(crate) fn array_for_interpolation(
    builder: &mut SynthDefBuilder,
    env: &Env,
) -> Result<UGenInput, BuildError> {
    let mut total = UGenInput::Number(0.0);
    for time in &env.times {
        total = lang::binary(builder, "+", total, time.clone())?;
    }
    let mut contents = vec![
        env.offset.clone(),
        env.level(0)?,
        number(env.times.len()),
        total,
    ];
    for (index, time) in env.times.iter().enumerate() {
        let curve = env.curve(index)?;
        contents.push(time.clone());
        contents.push(shape_number(curve)?);
        contents.push(curve_value(curve));
        contents.push(env.level(index + 1)?);
    }
    Ok(flop(&contents))
}

fn lin() -> Curve {
    Curve::from("lin")
}

fn zero() -> UGenInput {
    UGenInput::Number(0.0)
}

/// A release or a loop node: a number, or none.
fn node(args: &Args, name: &str) -> Result<Option<f64>, BuildError> {
    match args.optional(name)? {
        None => Ok(None),
        Some(UGenInput::Number(number)) => Ok(Some(number)),
        Some(_) => Err(args.wrong_kind(name, "a number")),
    }
}

/// `Env.new`.
fn new(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    Env::from_parts(
        args.array("levels")?,
        args.array("times")?,
        args.curve("curve")?,
        node(args, "releaseNode")?,
        node(args, "loopNode")?,
        args.input("offset")?,
    )
}

/// `Env.newClear`: levels of 0 and times of 1, in each channel.
fn new_clear(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let segments = if args.input("numSegments")? == UGenInput::Number(0.0) {
        0
    } else {
        args.count("numSegments")?
    };
    let channels = args.count("numChannels")?;
    if channels == 1 {
        return Env::from_parts(
            vec![zero(); segments + 1],
            vec![1.0.into(); segments],
            lin(),
            None,
            None,
            zero(),
        );
    }
    let channel = |value: f64| UGenInput::Array(vec![value.into(); channels]);
    Env::from_parts(
        vec![channel(0.0); segments + 1],
        vec![channel(1.0); segments],
        Curve::Array(vec![Curve::Array(vec![lin(); channels]); segments]),
        None,
        None,
        zero(),
    )
}

/// `Env.triangle`.
fn triangle(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    halves(builder, args, lin())
}

/// `Env.sine`.
fn sine(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    halves(builder, args, Curve::from("sine"))
}

/// `Env.triangle` and `Env.sine`: two segments of half the duration.
fn halves(builder: &mut SynthDefBuilder, args: &Args, curve: Curve) -> Result<Env, BuildError> {
    let half = lang::binary(builder, "*", args.input("dur")?, 0.5.into())?;
    Env::from_parts(
        vec![zero(), args.input("level")?, zero()],
        vec![half.clone(), half],
        curve,
        None,
        None,
        zero(),
    )
}

/// `Env.perc`.
fn perc(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    Env::from_parts(
        vec![zero(), args.input("level")?, zero()],
        vec![args.input("attackTime")?, args.input("releaseTime")?],
        args.curve("curve")?,
        None,
        None,
        zero(),
    )
}

/// `Env.linen`.
fn linen(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let level = args.input("level")?;
    Env::from_parts(
        vec![zero(), level.clone(), level, zero()],
        vec![
            args.input("attackTime")?,
            args.input("sustainTime")?,
            args.input("releaseTime")?,
        ],
        args.curve("curve")?,
        None,
        None,
        zero(),
    )
}

/// `Env.asr`.
fn asr(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    Env::from_parts(
        vec![zero(), args.input("sustainLevel")?, zero()],
        vec![args.input("attackTime")?, args.input("releaseTime")?],
        args.curve("curve")?,
        Some(1.0),
        None,
        zero(),
    )
}

/// The levels of `Env.adsr` and `Env.dadsr` after `peakLevel`, with
/// `peakLevel * sustainLevel`, and the bias added to `levels`.
fn biased(
    builder: &mut SynthDefBuilder,
    args: &Args,
    zeros: usize,
) -> Result<Vec<UGenInput>, BuildError> {
    let peak = args.input("peakLevel")?;
    let sustain = lang::binary(builder, "*", peak.clone(), args.input("sustainLevel")?)?;
    let mut levels = vec![zero(); zeros];
    levels.extend([peak, sustain, zero()]);
    let levels = lang::binary(builder, "+", UGenInput::Array(levels), args.input("bias")?)?;
    Ok(elements(levels))
}

/// `Env.adsr`.
fn adsr(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let levels = biased(builder, args, 1)?;
    Env::from_parts(
        levels,
        vec![
            args.input("attackTime")?,
            args.input("decayTime")?,
            args.input("releaseTime")?,
        ],
        args.curve("curve")?,
        Some(2.0),
        None,
        zero(),
    )
}

/// `Env.dadsr`.
fn dadsr(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let levels = biased(builder, args, 2)?;
    Env::from_parts(
        levels,
        vec![
            args.input("delayTime")?,
            args.input("attackTime")?,
            args.input("decayTime")?,
            args.input("releaseTime")?,
        ],
        args.curve("curve")?,
        Some(3.0),
        None,
        zero(),
    )
}

/// `Env.step`: the first level again, and the shape step.
fn step(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let levels = args.array("levels")?;
    let times = args.array("times")?;
    if levels.len() != times.len() {
        return Err(bad("Env.step", "the levels and the times differ in size"));
    }
    let first = levels
        .first()
        .cloned()
        .ok_or_else(|| bad("Env.step", "an envelope needs a level"))?;
    Env::from_parts(
        std::iter::once(first).chain(levels).collect(),
        times,
        Curve::from("step"),
        node(args, "releaseNode")?,
        node(args, "loopNode")?,
        args.input("offset")?,
    )
}

/// `Env.cutoff`.
fn cutoff(_: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let curve = args.curve("curve")?;
    let last = if shape_number(&curve)? == UGenInput::Number(2.0) {
        MINUS_100_DB
    } else {
        0.0
    };
    Env::from_parts(
        vec![args.input("level")?, last.into()],
        vec![args.input("releaseTime")?],
        curve,
        Some(0.0),
        None,
        zero(),
    )
}

/// A point of `Env.xyc`: a time, a level and a curve.
type Point = (UGenInput, UGenInput, Curve);

/// The point of a triplet, or of a pair with `curve`, as
/// `arr[0..1] ++ (arr[2] ? curve)` takes it.
fn point(
    args: &Args,
    name: &str,
    item: UGenInput,
    curve: Option<&Curve>,
) -> Result<Point, BuildError> {
    let UGenInput::Array(items) = item else {
        return Err(args.wrong_kind(name, "arrays of a time, a level and a curve"));
    };
    match items.as_slice() {
        [time, level] => Ok((
            time.clone(),
            level.clone(),
            curve.cloned().unwrap_or_else(lin),
        )),
        [time, level, third, ..] => Ok((time.clone(), level.clone(), Curve::from(third.clone()))),
        _ => Err(args.wrong_kind(name, "arrays of a time, a level and a curve")),
    }
}

/// `Env.xyc`.
fn xyc(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let points = args
        .array("xyc")?
        .into_iter()
        .map(|item| point(args, "xyc", item, None))
        .collect::<Result<_, _>>()?;
    from_points(builder, points)
}

/// `Env.pairs`: the curve of the pair `i` is `curve.asArray.wrapAt(i)`, as
/// `+++` laces it.
fn pairs(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let curves = match args.optional_curve("curve")? {
        None => Vec::new(),
        Some(Curve::Array(curves)) => curves,
        Some(curve) => vec![curve],
    };
    let points = args
        .array("pairs")?
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let curve = (!curves.is_empty()).then(|| &curves[index % curves.len()]);
            point(args, "pairs", item, curve)
        })
        .collect::<Result<_, _>>()?;
    from_points(builder, points)
}

/// The envelope of the points of `Env.xyc`: sorted by time when every time
/// is a number, the differences of the times, and the first time as the
/// offset. The curve of the last point goes unused.
fn from_points(builder: &mut SynthDefBuilder, mut points: Vec<Point>) -> Result<Env, BuildError> {
    let time = |point: &Point| match point.0 {
        UGenInput::Number(number) => Some(number),
        _ => None,
    };
    if points.iter().all(|point| time(point).is_some()) {
        points.sort_by(|a, b| time(a).unwrap_or(0.0).total_cmp(&time(b).unwrap_or(0.0)));
    }
    let offset = points
        .first()
        .map(|point| point.0.clone())
        .ok_or_else(|| bad("Env.xyc", "an envelope needs a point"))?;
    let mut times = Vec::with_capacity(points.len().saturating_sub(1));
    for pair in points.windows(2) {
        times.push(lang::binary(
            builder,
            "-",
            pair[1].0.clone(),
            pair[0].0.clone(),
        )?);
    }
    let levels = points.iter().map(|point| point.1.clone()).collect();
    let curves = Curve::Array(points.into_iter().map(|point| point.2).collect());
    Env::from_parts(levels, times, curves, None, None, offset)
}

/// `Env.circle`: the envelope of all but the last time and curve, then the
/// instance `circle` with them. It adds `Impulse`, `Delay1` and `Latch` for
/// `first0Then1`.
fn circle(builder: &mut SynthDefBuilder, args: &Args) -> Result<Env, BuildError> {
    let levels = args.array("levels")?;
    let times = args.array("times")?;
    let curves = match args.curve("curve")? {
        Curve::Array(curves) => curves,
        curve => vec![curve],
    };
    if levels.is_empty() || times.is_empty() || curves.is_empty() {
        return Err(bad(
            "Env.circle",
            "a circle needs a level, a time and a curve",
        ));
    }
    let size = levels.len();
    let times: Vec<UGenInput> = (0..size)
        .map(|index| times[index % times.len()].clone())
        .collect();
    let curves: Vec<Curve> = (0..size)
        .map(|index| curves[index % curves.len()].clone())
        .collect();
    let env = Env::from_parts(
        levels,
        times[..size - 1].to_vec(),
        Curve::Array(curves[..size - 1].to_vec()),
        None,
        None,
        zero(),
    )?;
    let impulse = builder.ugen("Impulse", "kr", &[("freq", 0.0.into())])?;
    let delayed = builder.ugen(
        "Delay1",
        "kr",
        &[("in", impulse.into()), ("x1", 0.0.into())],
    )?;
    let first0_then1 = builder.ugen(
        "Latch",
        "kr",
        &[("in", 1.0.into()), ("trig", delayed.into())],
    )?;
    let first_time = lang::binary(builder, "*", first0_then1, times[size - 1].clone())?;
    let mut circled_levels = vec![env.level(0)?];
    circled_levels.extend(env.levels);
    circled_levels.push(zero());
    let mut circled_curves = vec![curves[size - 1].clone()];
    circled_curves.extend_from_slice(&curves[..size - 1]);
    circled_curves.push(lin());
    let mut circled_times = vec![first_time];
    circled_times.extend(env.times);
    circled_times.push(f64::INFINITY.into());
    let release_node = circled_levels.len() - 2;
    Ok(Env {
        levels: circled_levels,
        times: circled_times,
        curves: Curve::Array(circled_curves),
        release_node: Some(f64::from(u32::try_from(release_node).unwrap_or(u32::MAX))),
        loop_node: Some(0.0),
        offset: zero(),
    })
}
