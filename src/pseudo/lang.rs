//! The arithmetic of sclang on the values of a build. Two numbers compute as
//! the language computes them, in f64. An array goes element by element, and
//! the shorter of two arrays wraps. Anything else goes to `BinaryOpUGen.new`
//! or `UnaryOpUGen.new`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
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
    use super::{number_binary, number_unary};
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
}
