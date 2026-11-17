//! The cases of the bodies of sc3-plugins in `bodies.json`. sclang built
//! each one into `bodies.scsyndef` through `generator/sc3_plugins/fixtures.scd`,
//! and the crate builds it from the same case.
//!
//! A case has a name, a call (a class, a selector and the arguments by name)
//! or a send (a receiver, a selector of a method of `UGen` and the
//! arguments by name), and the rate of an `Out` of the result, or none. A value is a number, an
//! array, `{"control": [name, default]}`, `{"ugen": [class, selector,
//! arguments]}`, `{"symbol": text}` or `{"bool": value}`. A refused case
//! breaks in sclang, and the crate must refuse it.

use scsynth_synthdef::catalog::{Addable, RateMethod};
use scsynth_synthdef::sc3_plugins;
use scsynth_synthdef::{
    Arg, ArgValue, BuildError, SynthDef, SynthDefBuilder, UGenInput, read_def_file,
};
use serde_json::Value;

use super::common::arg;
use super::golden;

const CASES: &str = include_str!("../fixtures/sc3_plugins/bodies.json");

fn cases() -> Vec<Value> {
    let cases: Value = serde_json::from_str(CASES).expect("the cases are JSON");
    cases["cases"].as_array().expect("a list of cases").clone()
}

fn text(value: &Value) -> &str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("{value} is not a text"))
}

fn list(value: &Value) -> &[Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{value} is not a list"))
}

fn refused(case: &Value) -> bool {
    case["refused"].as_bool() == Some(true)
}

/// The controls of a value, in the order in which sclang meets them: the
/// arguments of the graph function.
fn controls(value: &Value, found: &mut Vec<Arg>) {
    match value {
        Value::Array(items) => items.iter().for_each(|item| controls(item, found)),
        Value::Object(object) => {
            if let Some(control) = object.get("control") {
                let default = control[1].as_f64().expect("a default");
                found.push(Arg::new(text(&control[0]), default));
            } else if let Some(call) = object.get("ugen") {
                arguments(call, found);
            }
        }
        _ => {}
    }
}

/// The controls of the arguments of a call.
fn arguments(call: &Value, found: &mut Vec<Arg>) {
    for pair in list(&call[2]) {
        controls(&pair[1], found);
    }
}

/// A value of a case, built where the case puts it.
fn value(builder: &mut SynthDefBuilder, value: &Value) -> ArgValue {
    match value {
        Value::Number(number) => number.as_f64().expect("a number").into(),
        Value::Array(items) => {
            let items: Vec<UGenInput> = items.iter().map(|item| input(builder, item)).collect();
            items.into()
        }
        Value::Object(object) => match object.iter().next() {
            Some((tag, inner)) if tag == "control" => arg(builder, text(&inner[0])).into(),
            Some((tag, inner)) if tag == "ugen" => call(builder, inner)
                .unwrap_or_else(|error| panic!("{inner}: {error}"))
                .into(),
            Some((tag, inner)) if tag == "symbol" => text(inner).into(),
            Some((tag, inner)) if tag == "bool" => inner.as_bool().expect("a boolean").into(),
            _ => panic!("{value} is no value of a case"),
        },
        _ => panic!("{value} is no value of a case"),
    }
}

/// A value that goes among the inputs: a number, an output or an array.
fn input(builder: &mut SynthDefBuilder, spec: &Value) -> UGenInput {
    match value(builder, spec) {
        ArgValue::Input(input) => input,
        _ => panic!("{spec} is no input"),
    }
}

/// The call of a case, with the arguments by name in their order.
fn call(builder: &mut SynthDefBuilder, call: &Value) -> Result<UGenInput, BuildError> {
    let values: Vec<(&str, ArgValue)> = list(&call[2])
        .iter()
        .map(|pair| (text(&pair[0]), value(builder, &pair[1])))
        .collect();
    builder.ugen(text(&call[0]), text(&call[1]), &values)
}

/// The send of a case: the receiver, then the arguments by name in their
/// order, to the method of `UGen`.
fn send(builder: &mut SynthDefBuilder, send: &Value) -> Result<UGenInput, BuildError> {
    let receiver = input(builder, &send[0]);
    let values: Vec<(&str, ArgValue)> = list(&send[2])
        .iter()
        .map(|pair| (text(&pair[0]), value(builder, &pair[1])))
        .collect();
    builder.send(receiver, text(&send[1]), &values)
}

/// The definition of a case, or the error of the crate.
fn definition(case: &Value) -> Result<SynthDef, BuildError> {
    let mut found = Vec::new();
    if let Some(sent) = case.get("send") {
        controls(&sent[0], &mut found);
        arguments(sent, &mut found);
    } else {
        arguments(&case["call"], &mut found);
    }
    let mut builder = SynthDefBuilder::new(text(&case["name"]), found)?;
    let result = match case.get("send") {
        Some(sent) => send(&mut builder, sent)?,
        None => call(&mut builder, &case["call"])?,
    };
    if let Some(rate) = case["out"].as_str() {
        let channels = [("bus", 0.0.into()), ("channelsArray", result.into())];
        builder.ugen("Out", rate, &channels)?;
    }
    builder.finish()
}

#[test]
fn every_case_of_the_bodies_writes_the_bytes_of_sclang() {
    let file = read_def_file(&golden("bodies")).unwrap();
    let cases: Vec<Value> = cases().into_iter().filter(|case| !refused(case)).collect();
    assert_eq!(cases.len(), file.synth_defs.len());
    for (case, sclang) in cases.iter().zip(&file.synth_defs) {
        let name = text(&case["name"]);
        let def = definition(case).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            def.as_bytes().unwrap(),
            sclang.as_bytes().unwrap(),
            "{name}"
        );
    }
}

#[test]
fn every_refused_case_of_the_bodies_fails_in_the_crate() {
    let refusals: Vec<Value> = cases().into_iter().filter(refused).collect();
    assert_ne!(refusals.len(), 0, "bodies.json holds no refused case");
    for case in refusals {
        let name = text(&case["name"]);
        assert!(definition(&case).is_err(), "{name} builds");
    }
}

#[test]
fn every_method_of_ugen_of_sc3_plugins_has_a_case_that_builds() {
    for method in sc3_plugins::ugen_methods() {
        let selector = method.method.selector;
        let has_case = cases().iter().any(|case| {
            !refused(case)
                && case
                    .get("send")
                    .is_some_and(|sent| text(&sent[1]) == selector)
        });
        assert!(has_case, "UGen:{selector} has no case");
    }
}

#[test]
fn every_pseudo_method_of_sc3_plugins_has_a_case_that_builds() {
    let built: Vec<(String, String)> = cases()
        .iter()
        .filter(|case| !refused(case) && case.get("call").is_some())
        .map(|case| {
            let call = &case["call"];
            (text(&call[0]).to_owned(), text(&call[1]).to_owned())
        })
        .collect();
    let classes = sc3_plugins::all()
        .iter()
        .map(|class| (class.name, class.methods))
        .chain(
            sc3_plugins::pseudo_all()
                .iter()
                .map(|class| (class.name, class.methods)),
        );
    for (class, methods) in classes {
        let pseudo = methods
            .iter()
            .filter(|method| method.addable == Addable::Pseudo);
        for RateMethod { selector, .. } in pseudo {
            let has_case = built
                .iter()
                .any(|(name, method)| name == class && method == selector);
            assert!(has_case, "{class}.{selector} has no case");
        }
    }
}
