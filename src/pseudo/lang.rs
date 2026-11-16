//! The arithmetic of sclang on the values of a build. Two numbers compute as
//! the language computes them, in f64. An array goes element by element, and
//! the shorter of two arrays wraps. Anything else goes to `BinaryOpUGen.new`
//! or `UnaryOpUGen.new`.

use crate::builder::SynthDefBuilder;
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
/// or `max(lo, min(hi, x))` at demand rate. The primitive of a number fails
/// on bounds that are not numbers.
pub(crate) fn clip(
    builder: &mut SynthDefBuilder,
    x: UGenInput,
    lo: UGenInput,
    hi: UGenInput,
) -> Result<UGenInput, BuildError> {
    match x {
        UGenInput::Number(number) => match (lo, hi) {
            (UGenInput::Number(lo), UGenInput::Number(hi)) => {
                let below = if hi < number { hi } else { number };
                Ok(UGenInput::Number(if below < lo { lo } else { below }))
            }
            _ => Err(BuildError::BadMethod {
                selector: "clip".to_owned(),
                message: "Primitive '_ClipFloat' failed. Wrong type.".to_owned(),
            }),
        },
        UGenInput::Output(output) if output.rate() == Rate::Demand => {
            let below = binary(builder, "min", hi, output.into())?;
            binary(builder, "max", lo, below)
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

/// `replaceZeroesWithSilence`: the zeros of the top level of `items`, -0.0
/// too, become the channels of one `Silent.ar`, and each array inside goes
/// the same way. Without a zero at the top level, nothing changes, not even
/// in the arrays inside.
pub(crate) fn replace_zeroes_with_silence(
    builder: &mut SynthDefBuilder,
    items: Vec<UGenInput>,
) -> Result<Vec<UGenInput>, BuildError> {
    let is_zero = |item: &UGenInput| matches!(item, UGenInput::Number(number) if *number == 0.0);
    let zeros = items.iter().filter(|item| is_zero(item)).count();
    if zeros == 0 {
        return Ok(items);
    }
    let silent = builder.ugen(
        "Silent",
        "ar",
        &[("numChannels", super::number(zeros).into())],
    )?;
    let mut channels = super::elements(silent).into_iter();
    items
        .into_iter()
        .map(|item| match item {
            item if is_zero(&item) => Ok(channels.next().unwrap_or(item)),
            UGenInput::Array(inner) => {
                replace_zeroes_with_silence(builder, inner).map(UGenInput::Array)
            }
            other => Ok(other),
        })
        .collect()
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
#[allow(clippy::float_cmp)]
fn number_unary(op: &str, x: f64) -> Option<f64> {
    Some(match op {
        "neg" => -x,
        "abs" => x.abs(),
        "reciprocal" => 1.0 / x,
        "sqrt" => x.sqrt(),
        "cos" => x.cos(),
        "sin" => x.sin(),
        "exp" => x.exp(),
        "log" => x.ln(),
        "squared" => x * x,
        "sign" => {
            if x > 0.0 {
                1.0
            } else if x == 0.0 {
                0.0
            } else {
                -1.0
            }
        }
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
    // One fused multiply-add, as sclang 3.14.1 on arm64 computes it (see the
    // README and tests/fixtures/arithmetic.json).
    (-hi).mul_add((shifted / hi).floor(), shifted)
}

/// `Float:wrap`: `sc_mod(x - lo, hi - lo) + lo`, in f64.
pub(crate) fn wrap(x: f64, lo: f64, hi: f64) -> f64 {
    modulo(x - lo, hi - lo) + lo
}

/// `sc_fold`, which `Float:fold` and the operator `fold2` compute.
#[allow(clippy::float_cmp)]
pub(crate) fn fold(input: f64, lo: f64, hi: f64) -> f64 {
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
    use super::super::ugen_methods::send_message;
    use super::{fold, number_binary, number_unary, wrap};
    use crate::builder::SynthDefBuilder;
    use crate::ugen_input::{ArgValue, UGenInput};
    use serde_json::Value;

    /// `x.selector(args)` of numbers, as `send` sends it.
    fn sent(builder: &mut SynthDefBuilder, x: f64, selector: &str, args: &[f64]) -> f64 {
        let given = args.iter().map(|&arg| Some(ArgValue::from(arg))).collect();
        match send_message(builder, x.into(), selector, given) {
            Ok(UGenInput::Number(number)) => number,
            other => panic!("{x}.{selector}{args:?}: {other:?}"),
        }
    }

    const TABLE: &str = include_str!("../../tests/fixtures/arithmetic.json");

    fn float(hex: &Value) -> f64 {
        let bits = u64::from_str_radix(hex.as_str().expect("a hexadecimal float"), 16);
        f64::from_bits(bits.expect("16 hexadecimal digits"))
    }

    /// Whether `computed` is `expected`, bit for bit, or both are NaN. The
    /// libm of another platform can round `pow`, `cos` and `sin` one unit in
    /// the last place away from the libm of the macOS that wrote the table.
    fn same(op: &str, computed: f64, expected: f64) -> bool {
        let ulps = u64::from(matches!(op, "pow" | "cos" | "sin" | "exp" | "log"));
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
        assert_eq!(rows(&table, "unary").len(), 90);
    }

    #[test]
    fn clip_and_moddif_of_numbers_compute_as_sclang_computes_them() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        let mut builder = SynthDefBuilder::new("arithmetic", vec![]).expect("a build");
        for row in rows(&table, "clip") {
            let [x, lo, hi, expected] = [0, 1, 2, 3].map(|column| float(&row[column]));
            let computed = sent(&mut builder, x, "clip", &[lo, hi]);
            assert!(
                same("clip", computed, expected),
                "{x}.clip({lo}, {hi}): {computed}"
            );
        }
        for row in rows(&table, "moddif") {
            let [x, y, modulus, expected] = [0, 1, 2, 3].map(|column| float(&row[column]));
            let computed = sent(&mut builder, x, "moddif", &[y, modulus]);
            assert!(
                same("moddif", computed, expected),
                "{x}.moddif({y}, {modulus}): {computed}"
            );
        }
        assert_eq!(builder.finish().expect("an empty build").children.len(), 0);
    }

    #[test]
    fn fold_and_wrap_of_numbers_compute_as_sclang_computes_them() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        for (section, function) in [("fold", fold as fn(f64, f64, f64) -> f64), ("wrap", wrap)] {
            for row in rows(&table, section) {
                let [x, lo, hi, expected] = [0, 1, 2, 3].map(|column| float(&row[column]));
                let computed = function(x, lo, hi);
                assert!(
                    same(section, computed, expected),
                    "{x}.{section}({lo}, {hi}): {computed}"
                );
            }
            assert_eq!(rows(&table, section).len(), 27);
        }
    }

    #[test]
    fn linlin_of_numbers_computes_as_sclang_computes_it() {
        let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
        let mut builder = SynthDefBuilder::new("arithmetic", vec![]).expect("a build");
        for row in rows(&table, "linlin") {
            let [x, in_min, in_max, out_min, out_max, expected] =
                [0, 1, 2, 3, 4, 5].map(|column| float(&row[column]));
            let computed = sent(
                &mut builder,
                x,
                "linlin",
                &[in_min, in_max, out_min, out_max],
            );
            assert!(same("linlin", computed, expected), "{x}.linlin: {computed}");
        }
        assert_eq!(rows(&table, "linlin").len(), 18);
    }
}
