//! The pseudo-UGens of the plugin `TagSystemUGens`: `DbufTag` and `Dtag` of
//! `TagSystemUgens.sc`, and `Dfsm` of `SymbolicMachines.sc`.

use super::multi_new;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, elements, number};
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `DbufTag.new`.
pub(super) fn dbuf_tag(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    tag(builder, args, "DbufTag", "bufnum")
}

/// `Dtag.new`.
pub(super) fn dtag(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    tag(builder, args, "Dtag", "bufsize")
}

/// `DbufTag.new` or `Dtag.new`, whose first argument is `buffer`: the
/// inputs of `convertRules` after the others. They are the sizes of
/// `axiom` and `rules`, then `axiom`, the size of each rule and the rules
/// flattened by one level. `size` is 0 for a value that is not an array.
/// An array among the inputs would make several instances, and sclang
/// throws when it sets their `axiom`.
fn tag(
    builder: &mut SynthDefBuilder,
    args: &Args,
    class: &str,
    buffer: &str,
) -> Result<UGenInput, BuildError> {
    let axiom = args.optional("axiom")?;
    let UGenInput::Array(rules) = args.input("rules")? else {
        return Err(args.wrong_kind("rules", "an array"));
    };
    let mut names = vec![buffer, "v", "recycle", "mode", "axiom", "rules"];
    let mut inputs = vec![
        args.input(buffer)?,
        args.input("v")?,
        args.input("recycle")?,
        args.input("mode")?,
        number(axiom.as_ref().map_or(0, size)),
        number(rules.len()),
    ];
    for symbol in axiom.map(elements).unwrap_or_default() {
        names.push("axiom");
        inputs.push(symbol);
    }
    for rule in &rules {
        names.push("rules");
        inputs.push(number(size(rule)));
    }
    for symbol in rules.into_iter().flat_map(elements) {
        names.push("rules");
        inputs.push(symbol);
    }
    if let Some(index) = inputs
        .iter()
        .position(|input| matches!(input, UGenInput::Array(_)))
    {
        return Err(args.wrong_kind(names[index], "values that make one instance"));
    }
    multi_new(builder, (class, Rate::Demand), &names, &inputs, &|_| Ok(1))
}

/// `Dfsm.new`: the inputs of `convertRules` after `n` and `rgen`, which is
/// a `Dwhite.new` when nil. The rules, with an exit state of 0.0 when their
/// size is odd, rotate by one. Then the states, at the even places, come
/// after their count and the size of each next state, and the next states
/// follow, flattened.
pub(super) fn dfsm(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let UGenInput::Array(mut rules) = args.input("rules")? else {
        return Err(args.wrong_kind("rules", "an array"));
    };
    let rgen = match args.optional("rgen")? {
        Some(rgen) => rgen,
        None => builder.ugen("Dwhite", "new", &[])?,
    };
    if rules.len() % 2 == 1 {
        rules.push(0.0.into());
    }
    rules.rotate_right(1);
    let (states, next): (Vec<_>, Vec<_>) = rules
        .chunks(2)
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .unzip();
    let mut names = vec!["n", "rgen", "rules"];
    let mut inputs = vec![args.input("n")?, rgen, number(states.len())];
    for state in &next {
        names.push("rules");
        inputs.push(number(elements(state.clone()).len()));
    }
    for state in states.into_iter().chain(next.iter().flat_map(flat)) {
        names.push("rules");
        inputs.push(state);
    }
    multi_new(builder, ("Dfsm", Rate::Demand), &names, &inputs, &|_| Ok(1))
}

/// `size` of sclang: the count of the elements of an array, or 0.
fn size(value: &UGenInput) -> usize {
    match value {
        UGenInput::Array(items) => items.len(),
        _ => 0,
    }
}

/// `flat` of sclang: the elements of `value` at every depth, in order.
fn flat(value: &UGenInput) -> Vec<UGenInput> {
    match value {
        UGenInput::Array(items) => items.iter().flat_map(flat).collect(),
        other => vec![other.clone()],
    }
}
