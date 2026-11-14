//! The `new` of `Sum3` and `Sum4`, in `BasicOpsUGen.sc`: real UGens, which
//! fold and sort their inputs before they add one.

use super::{Args, lang};
use crate::builder::{NewUGen, SynthDefBuilder};
use crate::error::BuildError;
use crate::rate::{Rate, least_by_name};
use crate::ugen_input::UGenInput;

/// `Sum3.new`.
pub(super) fn sum3(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let inputs = [args.input("in0")?, args.input("in1")?, args.input("in2")?];
    sum3_new(builder, &inputs)
}

/// `Sum4.new`.
pub(super) fn sum4(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let inputs = [
        args.input("in0")?,
        args.input("in1")?,
        args.input("in2")?,
        args.input("in3")?,
    ];
    sum4_new(builder, &inputs)
}

/// `Sum3.new` of three inputs: the expansion of `multiNew`, then `new1`.
pub(super) fn sum3_new(
    builder: &mut SynthDefBuilder,
    inputs: &[UGenInput; 3],
) -> Result<UGenInput, BuildError> {
    builder.expand(
        "Sum3",
        &["in0", "in1", "in2"],
        inputs,
        true,
        &mut |builder, instance| {
            let [in0, in1, in2] = [&instance[0], &instance[1], &instance[2]].map(Clone::clone);
            sum3_leaf(builder, [in0, in1, in2])
        },
    )
}

/// `Sum4.new` of four inputs: the expansion of `multiNew`, then `new1`.
pub(super) fn sum4_new(
    builder: &mut SynthDefBuilder,
    inputs: &[UGenInput; 4],
) -> Result<UGenInput, BuildError> {
    builder.expand(
        "Sum4",
        &["in0", "in1", "in2", "in3"],
        inputs,
        true,
        &mut |builder, instance| {
            let [in0, in1, in2, in3] =
                [&instance[0], &instance[1], &instance[2], &instance[3]].map(Clone::clone);
            sum4_leaf(builder, [in0, in1, in2, in3])
        },
    )
}

/// `Sum3:new1`: a `+` for an input of 0, or a `Sum3`.
fn sum3_leaf(
    builder: &mut SynthDefBuilder,
    [in0, in1, in2]: [UGenInput; 3],
) -> Result<UGenInput, BuildError> {
    if is_zero(&in2) {
        return lang::binary(builder, "+", in0, in1);
    }
    if is_zero(&in1) {
        return lang::binary(builder, "+", in0, in2);
    }
    if is_zero(&in0) {
        return lang::binary(builder, "+", in1, in2);
    }
    add_sum(builder, "Sum3", vec![in0, in1, in2])
}

/// `Sum4:new1`: a `Sum3` of the other inputs for an input of 0, or a `Sum4`.
fn sum4_leaf(
    builder: &mut SynthDefBuilder,
    [in0, in1, in2, in3]: [UGenInput; 4],
) -> Result<UGenInput, BuildError> {
    if is_zero(&in0) {
        return sum3_leaf(builder, [in1, in2, in3]);
    }
    if is_zero(&in1) {
        return sum3_leaf(builder, [in0, in2, in3]);
    }
    if is_zero(&in2) {
        return sum3_leaf(builder, [in0, in1, in3]);
    }
    if is_zero(&in3) {
        return sum3_leaf(builder, [in0, in1, in2]);
    }
    add_sum(builder, "Sum4", vec![in0, in1, in2, in3])
}

/// A `Sum3` or a `Sum4` at the least rate of its inputs by name, with the
/// inputs in the order of the sort of sclang.
fn add_sum(
    builder: &mut SynthDefBuilder,
    class: &str,
    mut inputs: Vec<UGenInput>,
) -> Result<UGenInput, BuildError> {
    let rate = least_by_name(inputs.iter().map(rate_of)).unwrap_or(Rate::Scalar);
    sort_by_rate(&mut inputs);
    builder.add(NewUGen::new(class, rate).inputs(inputs))
}

/// `insertionSortRange` of `sort { |a, b| a.rate < b.rate }`, which moves an
/// input left past every input whose rate name is not less than its own. The
/// stable sort of Rust gives another order for inputs of one rate.
fn sort_by_rate(inputs: &mut [UGenInput]) {
    let rank = |input: &UGenInput| match rate_of(input) {
        Rate::Audio => 0,
        Rate::Control => 1,
        Rate::Demand => 2,
        Rate::Scalar => 3,
    };
    for i in 1..inputs.len() {
        let test = inputs[i].clone();
        let mut j = i;
        while j > 0 && rank(&inputs[j - 1]) >= rank(&test) {
            inputs[j] = inputs[j - 1].clone();
            j -= 1;
        }
        inputs[j] = test;
    }
}

fn rate_of(input: &UGenInput) -> Rate {
    input.rate().unwrap_or(Rate::Scalar)
}

#[allow(clippy::float_cmp)]
fn is_zero(input: &UGenInput) -> bool {
    matches!(input, UGenInput::Number(number) if *number == 0.0)
}
