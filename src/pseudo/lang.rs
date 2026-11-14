//! The arithmetic of sclang on the values of a build. Two numbers compute as
//! the language computes them, in f64. An array goes element by element, and
//! the shorter of two arrays wraps. Anything else goes to `BinaryOpUGen.new`
//! or `UnaryOpUGen.new`.

use crate::builder::{NewUGen, SynthDefBuilder};
use crate::error::BuildError;
use crate::rate::{Rate, least_by_name};
use crate::ugen_input::UGenInput;

/// `x op y` in sclang, for an operator of `BinaryOpUGen` by its name.
pub(crate) fn binary(
    builder: &mut SynthDefBuilder,
    op: &str,
    x: UGenInput,
    y: UGenInput,
) -> Result<UGenInput, BuildError> {
    let size = match (&x, &y) {
        (UGenInput::Number(a), UGenInput::Number(b)) => {
            return number_binary(op, *a, *b)
                .map(UGenInput::Number)
                .ok_or_else(|| BuildError::UnknownOperator(op.to_owned()));
        }
        (UGenInput::Array(xs), UGenInput::Array(ys)) if !xs.is_empty() && !ys.is_empty() => {
            xs.len().max(ys.len())
        }
        (UGenInput::Array(items), other) | (other, UGenInput::Array(items))
            if !items.is_empty() && !matches!(other, UGenInput::Array(_)) =>
        {
            items.len()
        }
        _ => return builder.binary_op(op, x, y),
    };
    (0..size)
        .map(|index| binary(builder, op, element(&x, index), element(&y, index)))
        .collect::<Result<_, _>>()
        .map(UGenInput::Array)
}

/// `x.op` in sclang, for an operator of `UnaryOpUGen` by its name.
pub(crate) fn unary(
    builder: &mut SynthDefBuilder,
    op: &str,
    x: UGenInput,
) -> Result<UGenInput, BuildError> {
    match x {
        UGenInput::Number(number) => number_unary(op, number)
            .map(UGenInput::Number)
            .ok_or_else(|| BuildError::UnknownOperator(op.to_owned())),
        UGenInput::Array(items) if !items.is_empty() => items
            .into_iter()
            .map(|item| unary(builder, op, item))
            .collect::<Result<_, _>>()
            .map(UGenInput::Array),
        other => builder.unary_op(op, other),
    }
}

/// `Collection:sum`: 0, plus each element in order.
pub(crate) fn sum(
    builder: &mut SynthDefBuilder,
    items: Vec<UGenInput>,
) -> Result<UGenInput, BuildError> {
    items
        .into_iter()
        .try_fold(UGenInput::Number(0.0), |total, item| {
            binary(builder, "+", total, item)
        })
}

/// The rate of a value, as `rate` gives it in sclang. A number runs at scalar
/// rate. An array of one element has the rate of the element. Another array
/// has the least rate of its elements by name, and an empty array has none.
pub(crate) fn rate(value: &UGenInput) -> Option<Rate> {
    match value {
        UGenInput::Number(_) => Some(Rate::Scalar),
        UGenInput::Output(output) => Some(output.rate()),
        UGenInput::Array(items) if items.len() == 1 => rate(&items[0]),
        UGenInput::Array(items) => {
            least_by_name(items.iter().map(|item| rate(item).unwrap_or(Rate::Scalar)))
        }
    }
}

/// `x.clip(lo, hi)`: `sc_clip` of a number, a `Clip` at the rate of a UGen,
/// or `max(lo, min(hi, x))` at demand rate. An array clips element by
/// element.
pub(crate) fn clip(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    lo: f64,
    hi: f64,
) -> Result<UGenInput, BuildError> {
    match x {
        UGenInput::Number(number) => {
            let below = if hi < number { hi } else { number };
            Ok(UGenInput::Number(if below < lo { lo } else { below }))
        }
        UGenInput::Array(items) if !items.is_empty() => items
            .into_iter()
            .map(|item| clip(builder, item, lo, hi))
            .collect::<Result<_, _>>()
            .map(UGenInput::Array),
        UGenInput::Output(output) if output.rate() == Rate::Demand => {
            let below = binary(builder, "min", hi.into(), output.into())?;
            binary(builder, "max", lo.into(), below)
        }
        other => {
            let selector = selector_for_rate(rate(&other).unwrap_or(Rate::Scalar));
            builder.ugen(
                "Clip",
                selector,
                &[("in", other.into()), ("lo", lo.into()), ("hi", hi.into())],
            )
        }
    }
}

/// `x.moddif(y, mod)`. For a number, `SimpleNumber:moddif`:
/// `mod * 0.5 - absdif(absdif(x, y) % mod, mod * 0.5)`. For a UGen, a
/// `ModDif` at its rate. An array goes element by element, and the shorter
/// array wraps.
pub(crate) fn moddif(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    y: UGenInput,
    modulus: UGenInput,
) -> Result<UGenInput, BuildError> {
    match x {
        UGenInput::Number(_) => {
            let distance = binary(builder, "absdif", x, y)?;
            let reduced = binary(builder, "mod", distance, modulus.clone())?;
            let half = binary(builder, "*", modulus, 0.5.into())?;
            let gap = binary(builder, "absdif", reduced, half.clone())?;
            binary(builder, "-", half, gap)
        }
        UGenInput::Array(ref items) if !items.is_empty() => {
            let size = [&x, &y, &modulus]
                .iter()
                .filter_map(|value| match value {
                    UGenInput::Array(items) => Some(items.len()),
                    _ => None,
                })
                .max()
                .unwrap_or(1);
            (0..size)
                .map(|index| {
                    let at = |value: &UGenInput| element(value, index);
                    moddif(builder, at(&x), at(&y), at(&modulus))
                })
                .collect::<Result<_, _>>()
                .map(UGenInput::Array)
        }
        UGenInput::Output(output) if output.rate() == Rate::Demand => builder.expand(
            "ModDif",
            &["x", "y", "mod"],
            &[x, y, modulus],
            true,
            &mut |builder, instance| {
                builder.add(NewUGen::new("ModDif", Rate::Demand).inputs(instance.to_vec()))
            },
        ),
        other => {
            let selector = selector_for_rate(rate(&other).unwrap_or(Rate::Scalar));
            builder.ugen(
                "ModDif",
                selector,
                &[
                    ("x", other.into()),
                    ("y", y.into()),
                    ("mod", modulus.into()),
                ],
            )
        }
    }
}

/// `asAudioRateInput`: a `DC` for a number, a `K2A` for a UGen that does not
/// run at audio rate, and each element of an array in turn.
#[allow(clippy::float_cmp)]
pub(crate) fn as_audio_rate_input(
    builder: &mut SynthDefBuilder,
    input: UGenInput,
) -> Result<UGenInput, BuildError> {
    match input {
        UGenInput::Number(number) => {
            // Any zero, -0.0 too, gives `Silent.ar`: `DC.ar(0)`.
            let level = if number == 0.0 { 0.0 } else { number };
            builder.ugen("DC", "ar", &[("in", level.into())])
        }
        UGenInput::Output(output) if output.rate() == Rate::Audio => Ok(output.into()),
        UGenInput::Output(output) => builder.ugen("K2A", "ar", &[("in", output.into())]),
        UGenInput::Array(items) => items
            .into_iter()
            .map(|item| as_audio_rate_input(builder, item))
            .collect::<Result<_, _>>()
            .map(UGenInput::Array),
    }
}

/// `a.blend(b, frac)`: `a + (frac * (b - a))` for a number. For a UGen, an
/// `XFade2` when one side runs at audio rate, else a `LinXFade2`, with the
/// pan of `frac`. An array blends element by element.
pub(crate) fn blend(
    builder: &mut SynthDefBuilder,
    a: UGenInput,
    b: UGenInput,
    frac: f64,
) -> Result<UGenInput, BuildError> {
    match a {
        UGenInput::Number(_) => {
            let difference = binary(builder, "-", b, a.clone())?;
            let scaled = binary(builder, "*", frac.into(), difference)?;
            binary(builder, "+", a, scaled)
        }
        UGenInput::Array(ref items) if !items.is_empty() => {
            let size = match &b {
                UGenInput::Array(others) => items.len().max(others.len()),
                _ => items.len(),
            };
            (0..size)
                .map(|index| blend(builder, element(&a, index), element(&b, index), frac))
                .collect::<Result<_, _>>()
                .map(UGenInput::Array)
        }
        other => {
            let (own, that) = (rate(&other), rate(&b));
            let unsupported = |rate| BuildError::UnsupportedRate {
                ugen: "UGen:blend".to_owned(),
                rate,
            };
            if own == Some(Rate::Demand) || that == Some(Rate::Demand) {
                return Err(unsupported(Rate::Demand));
            }
            let pan = linlin(frac, 0.0, 1.0, -1.0, 1.0);
            let (selector, class, a, b, pan) = if own == Some(Rate::Audio) {
                ("ar", "XFade2", other, b, pan)
            } else if that == Some(Rate::Audio) {
                ("ar", "XFade2", b, other, -pan)
            } else if own == Some(Rate::Control) {
                ("kr", "LinXFade2", other, b, pan)
            } else {
                return Err(unsupported(Rate::Scalar));
            };
            builder.ugen(
                class,
                selector,
                &[("inA", a.into()), ("inB", b.into()), ("pan", pan.into())],
            )
        }
    }
}

/// `SimpleNumber:linlin` with the clip `minmax`.
pub(crate) fn linlin(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    if x <= in_min {
        out_min
    } else if x >= in_max {
        out_max
    } else {
        (x - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
    }
}

/// `methodSelectorForRate` of a class that answers `ir`, as `Clip` and
/// `ModDif` do.
pub(crate) fn selector_for_rate(rate: Rate) -> &'static str {
    match rate {
        Rate::Audio => "ar",
        Rate::Control => "kr",
        Rate::Scalar => "ir",
        Rate::Demand => "new",
    }
}

/// Element `index` of an array, which wraps, or the value itself.
fn element(value: &UGenInput, index: usize) -> UGenInput {
    match value {
        UGenInput::Array(items) => items[index % items.len()].clone(),
        other => other.clone(),
    }
}

/// A binary operator on two Floats, as `doSpecialBinaryArithMsg` computes it
/// with the functions of `SC_InlineBinaryOp.h` and `SC_BoundsMacros.h`.
fn number_binary(op: &str, x: f64, y: f64) -> Option<f64> {
    Some(match op {
        "+" => x + y,
        "-" => x - y,
        "*" => x * y,
        "/" => x / y,
        "pow" => x.powf(y),
        "mod" => modulo(x, y),
        "round" => quantize(x, y, |quotient| (quotient + 0.5).floor()),
        "roundUp" => quantize(x, y, f64::ceil),
        "trunc" => quantize(x, y, f64::floor),
        "fold2" => fold(x, -y, y),
        "max" => {
            if x > y {
                x
            } else {
                y
            }
        }
        "min" => {
            if x < y {
                x
            } else {
                y
            }
        }
        "absdif" => (x - y).abs(),
        _ => return None,
    })
}

/// `sc_round`, `sc_roundUp` and `sc_trunc`: a multiple of `quant`, with the
/// rounding of `x / quant`.
#[allow(clippy::float_cmp)]
fn quantize(x: f64, quant: f64, round: impl Fn(f64) -> f64) -> f64 {
    if quant == 0.0 {
        x
    } else {
        round(x / quant) * quant
    }
}

/// A unary operator on a Float, as `doSpecialUnaryArithMsg` computes it.
fn number_unary(op: &str, x: f64) -> Option<f64> {
    Some(match op {
        "neg" => -x,
        "abs" => x.abs(),
        "reciprocal" => 1.0 / x,
        "sqrt" => x.sqrt(),
        "cos" => x.cos(),
        "sin" => x.sin(),
        _ => return None,
    })
}

/// `sc_mod`.
#[allow(clippy::float_cmp)]
fn modulo(input: f64, hi: f64) -> f64 {
    let shifted = if input >= hi {
        let reduced = input - hi;
        if reduced < hi {
            return reduced;
        }
        reduced
    } else if input < 0.0 {
        let raised = input + hi;
        if raised >= 0.0 {
            return raised;
        }
        raised
    } else {
        return input;
    };
    if hi == 0.0 {
        return 0.0;
    }
    // sclang 3.14.1 for macOS on arm64 computes `in - hi * floor(in / hi)`
    // with one fused multiply-add: 1000 % 3.7 is 0.9999999999999067.
    (-hi).mul_add((shifted / hi).floor(), shifted)
}

/// `sc_fold`.
#[allow(clippy::float_cmp)]
fn fold(input: f64, lo: f64, hi: f64) -> f64 {
    let x = input - lo;
    if input >= hi {
        let folded = hi + hi - input;
        if folded >= lo {
            return folded;
        }
    } else if input < lo {
        let folded = lo + lo - input;
        if folded < hi {
            return folded;
        }
    } else {
        return input;
    }
    if hi == lo {
        return lo;
    }
    let range = hi - lo;
    let range2 = range + range;
    // Fused, as in `modulo`.
    let c = (-range2).mul_add((x / range2).floor(), x);
    if c >= range { range2 - c + lo } else { c + lo }
}

#[cfg(test)]
mod tests {
    use super::{clip, linlin, moddif, number_binary, number_unary};
    use crate::builder::SynthDefBuilder;
    use crate::ugen_input::UGenInput;
    use serde_json::Value;

    const TABLE: &str = include_str!("../../tests/fixtures/arithmetic.json");

    fn float(hex: &Value) -> f64 {
        let bits = u64::from_str_radix(hex.as_str().expect("a hexadecimal float"), 16);
        f64::from_bits(bits.expect("16 hexadecimal digits"))
    }

    /// Whether `computed` is `expected`, bit for bit, or both are NaN. The
    /// libm of another platform can round `pow`, `cos` and `sin` one unit in
    /// the last place away from the libm of the macOS that wrote the table.
    fn same(op: &str, computed: f64, expected: f64) -> bool {
        let ulps = u64::from(matches!(op, "pow" | "cos" | "sin"));
        (computed.is_nan() && expected.is_nan())
            || computed.to_bits().abs_diff(expected.to_bits()) <= ulps
    }

    fn rows<'a>(table: &'a Value, section: &str) -> &'a [Value] {
        table[section].as_array().expect("a section of the table")
    }

    #[test]
    fn two_numbers_compute_as_sclang_computes_them() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        for row in rows(&table, "binary") {
            let op = row[0].as_str().expect("an operator");
            let (x, y, expected) = (float(&row[1]), float(&row[2]), float(&row[3]));
            let computed = number_binary(op, x, y).expect("the operator computes");
            assert!(
                same(op, computed, expected),
                "{x} {op} {y}: {computed}, not {expected}"
            );
        }
        assert_eq!(rows(&table, "binary").len(), 1053);
    }

    #[test]
    fn a_number_computes_as_sclang_computes_it() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        for row in rows(&table, "unary") {
            let op = row[0].as_str().expect("an operator");
            let (x, expected) = (float(&row[1]), float(&row[2]));
            let computed = number_unary(op, x).expect("the operator computes");
            assert!(
                same(op, computed, expected),
                "{x}.{op}: {computed}, not {expected}"
            );
        }
        assert_eq!(rows(&table, "unary").len(), 54);
    }

    #[test]
    fn clip_and_moddif_of_numbers_compute_as_sclang_computes_them() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        let mut builder = SynthDefBuilder::new("arithmetic", vec![]).expect("a build");
        let number = |value: UGenInput| match value {
            UGenInput::Number(number) => number,
            other => panic!("{other:?} is not a number"),
        };
        for row in rows(&table, "clip") {
            let [x, lo, hi, expected] = [0, 1, 2, 3].map(|column| float(&row[column]));
            let computed = number(clip(&mut builder, x.into(), lo, hi).expect("clip computes"));
            assert!(
                same("clip", computed, expected),
                "{x}.clip({lo}, {hi}): {computed}"
            );
        }
        for row in rows(&table, "moddif") {
            let [x, y, modulus, expected] = [0, 1, 2, 3].map(|column| float(&row[column]));
            let computed = moddif(&mut builder, x.into(), y.into(), modulus.into());
            let computed = number(computed.expect("moddif computes"));
            assert!(
                same("moddif", computed, expected),
                "{x}.moddif({y}, {modulus}): {computed}"
            );
        }
        assert_eq!(builder.finish().expect("an empty build").children.len(), 0);
    }

    #[test]
    fn linlin_of_numbers_computes_as_sclang_computes_it() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        for row in rows(&table, "linlin") {
            let [x, in_min, in_max, out_min, out_max, expected] =
                [0, 1, 2, 3, 4, 5].map(|column| float(&row[column]));
            let computed = linlin(x, in_min, in_max, out_min, out_max);
            assert!(same("linlin", computed, expected), "{x}.linlin: {computed}");
        }
        assert_eq!(rows(&table, "linlin").len(), 18);
    }
}
