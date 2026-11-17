//! `UHJ.sc`: `BtoUHJ` and `UHJtoB`, on the two phases of `Shift90.filt`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// The `a0` of the six allpass `FOS` of each chain of `Shift90.filt`, whose
/// `b1` is `-a0`.
const SHIFTS: [[f64; 6]; 2] = [
    [
        -0.999_229,
        -0.994_158_79,
        -0.976_436_25,
        -0.908_711_82,
        -0.677_935_63,
        -0.079_210_14,
    ],
    [
        -0.997_327_03,
        -0.988_172_7,
        -0.953_364_81,
        -0.825_195_99,
        -0.439_051_06,
        0.494_956_51,
    ],
];

/// `BtoUHJ.ar`: the two channels of UHJ, from `w`, `x` and `y` and their
/// shifts.
pub(crate) fn b_to_uhj(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [w, wj] = shift90(builder, args.input("wsig")?)?;
    let [x, xj] = shift90(builder, args.input("xsig")?)?;
    let shifted = UGenInput::Array(Vec::from(shift90(builder, args.input("ysig")?)?));
    let y = match lang::binary(builder, "*", shifted, 0.655.into())? {
        UGenInput::Array(mut items) => items.swap_remove(0),
        other => other,
    };
    let w = lang::binary(builder, "*", 0.9397.into(), w)?;
    let wj = lang::binary(builder, "*", 0.342.into(), wj)?;
    let x = lang::binary(builder, "*", 0.1856.into(), x)?;
    let xj = lang::binary(builder, "*", 0.5099.into(), xj)?;
    let left = chain(builder, &w, [("+", &x), ("-", &wj), ("+", &xj), ("+", &y)])?;
    let right = chain(builder, &w, [("+", &x), ("+", &wj), ("-", &xj), ("-", &y)])?;
    lang::binary(
        builder,
        "*",
        UGenInput::Array(vec![left, right]),
        0.5.into(),
    )
}

/// `UHJtoB.ar`: `w`, `x` and `y`, from the two channels of UHJ and their
/// shifts.
pub(crate) fn uhj_to_b(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let [left, left_j] = shift90(builder, args.input("leftsig")?)?;
    let [right, right_j] = shift90(builder, args.input("rightsig")?)?;
    let w = weighted(
        builder,
        [
            (0.982, &left),
            (0.982, &right),
            (0.164, &left_j),
            (0.164, &right_j),
        ],
        ["+", "+", "-"],
    )?;
    let x = weighted(
        builder,
        [
            (0.419, &left),
            (0.419, &right),
            (0.828, &left_j),
            (0.828, &right_j),
        ],
        ["+", "-", "+"],
    )?;
    let y = weighted(
        builder,
        [
            (0.763, &left),
            (0.763, &right),
            (0.385, &left_j),
            (0.385, &right_j),
        ],
        ["-", "+", "+"],
    )?;
    lang::binary(builder, "*", UGenInput::Array(vec![w, x, y]), 0.5.into())
}

/// `Shift90.filt(in)`: `[ay6, ay12]`, two chains of six allpass `FOS`.
fn shift90(builder: &mut SynthDefBuilder, input: UGenInput) -> Result<[UGenInput; 2], BuildError> {
    let mut chains = [input.clone(), input];
    for (out, coefficients) in chains.iter_mut().zip(SHIFTS) {
        for a0 in coefficients {
            let values = [
                ("in", out.clone().into()),
                ("a0", a0.into()),
                ("a1", 1.0.into()),
                ("b1", (-a0).into()),
            ];
            *out = builder.ugen("FOS", "ar", &values)?;
        }
    }
    Ok(chains)
}

/// `first op x op y ...`, from the left, as sclang computes it.
fn chain<const N: usize>(
    builder: &mut SynthDefBuilder,
    first: &UGenInput,
    terms: [(&str, &UGenInput); N],
) -> Result<UGenInput, BuildError> {
    terms
        .into_iter()
        .try_fold(first.clone(), |total, (op, term)| {
            lang::binary(builder, op, total, term.clone())
        })
}

/// `(k0 * s0) op (k1 * s1) op (k2 * s2) op (k3 * s3)`, where sclang makes
/// each product right before its operator.
fn weighted(
    builder: &mut SynthDefBuilder,
    [(k0, s0), terms @ ..]: [(f64, &UGenInput); 4],
    ops: [&str; 3],
) -> Result<UGenInput, BuildError> {
    let mut total = lang::binary(builder, "*", k0.into(), s0.clone())?;
    for ((k, s), op) in terms.into_iter().zip(ops) {
        let product = lang::binary(builder, "*", k.into(), s.clone())?;
        total = lang::binary(builder, op, total, product)?;
    }
    Ok(total)
}
