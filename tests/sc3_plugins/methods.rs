//! One definition for each addable method of the catalogue of sc3-plugins,
//! in `methods.scsyndef`. The crate builds each one from the catalogue alone,
//! with the values that `generator/sc3_plugins/fixtures.scd` gives.

use scsynth_synthdef::catalog::{Addable, ArgKind, Outputs, RateMethod, UGenClass, Wiring};
use scsynth_synthdef::sc3_plugins;
use scsynth_synthdef::{Arg, ArgValue, Rate, SynthDef, SynthDefBuilder, read_def_file};

use super::common::{arg, by_name};
use super::golden;

/// Each addable method of the catalogue, with its class and its wiring, in
/// the order of the catalogue.
fn addable() -> impl Iterator<Item = (&'static UGenClass, &'static RateMethod, Wiring)> {
    sc3_plugins::all().iter().flat_map(|class| {
        class
            .methods
            .iter()
            .filter_map(move |method| match method.addable {
                Addable::Yes(wiring) => Some((class, method, wiring)),
                _ => None,
            })
    })
}

/// The control of the argument at `position`: `a_` for `ar`, `i_` for `ir`.
fn control(method: &RateMethod, position: usize) -> String {
    let prefix = match method.selector {
        "ar" => "a_",
        "ir" => "i_",
        _ => "",
    };
    format!("{prefix}s{position}")
}

/// The sentinel of the argument at `position`, as the probes give it.
fn signal(position: usize) -> f64 {
    1000.25 + 17.0 * f64::from(u32::try_from(position).expect("a small position"))
}

/// The definition `m<index>`: each signal is a control at the rate of the
/// method, each count 2, each array two such controls, and each symbol
/// `/abcd`. A UGen with outputs gets `mul` 0.5 and `add` 0.25, where the
/// method applies them, and an `Out` of its result.
fn definition(index: usize, class: &UGenClass, method: &RateMethod, wiring: Wiring) -> SynthDef {
    let outputs = !matches!(wiring.outputs, Outputs::Fixed(rates) if rates.is_empty());
    let madd = |name: &str| wiring.madd && (name == "mul" || name == "add");
    let mut controls = Vec::new();
    for (position, spec) in method.args.iter().enumerate() {
        let name = control(method, position);
        match spec.kind {
            ArgKind::Signal if !madd(spec.name) => controls.push(Arg::new(&name, signal(position))),
            ArgKind::Array => controls.push(Arg::array(
                &name,
                [signal(position) + 1000.0, signal(position) + 1000.5],
            )),
            _ => {}
        }
    }
    let mut builder = SynthDefBuilder::new(format!("m{index}"), controls).unwrap();
    let values: Vec<(&str, ArgValue)> = method
        .args
        .iter()
        .enumerate()
        .map(|(position, spec)| {
            let value = match (spec.kind, spec.name) {
                (ArgKind::Signal, "mul") if madd("mul") => if outputs { 0.5 } else { 1.0 }.into(),
                (ArgKind::Signal, "add") if madd("add") => if outputs { 0.25 } else { 0.0 }.into(),
                (ArgKind::Signal | ArgKind::Array, _) => {
                    arg(&builder, &control(method, position)).into()
                }
                (ArgKind::Count, _) => 2.0.into(),
                (ArgKind::Symbol, _) => "/abcd".into(),
                (kind, name) => panic!(
                    "{}.{} {name} has the kind {kind:?}",
                    class.name, method.selector
                ),
            };
            (spec.name, value)
        })
        .collect();
    let result = by_name(&mut builder, class.name, method.selector, &values);
    if outputs {
        let selector = if wiring.rate == Rate::Audio {
            "ar"
        } else {
            "kr"
        };
        by_name(
            &mut builder,
            "Out",
            selector,
            &[("bus", 0.0.into()), ("channelsArray", result.into())],
        );
    }
    builder
        .finish()
        .unwrap_or_else(|error| panic!("{}.{}: {error}", class.name, method.selector))
}

#[test]
fn every_addable_method_writes_the_bytes_of_sclang() {
    let file = read_def_file(&golden("methods")).unwrap();
    let methods: Vec<_> = addable().collect();
    assert_eq!(methods.len(), file.synth_defs.len());
    for (index, ((class, method, wiring), sclang)) in
        methods.into_iter().zip(&file.synth_defs).enumerate()
    {
        let built = definition(index, class, method, wiring);
        assert_eq!(
            built.as_bytes().unwrap(),
            sclang.as_bytes().unwrap(),
            "{}.{}",
            class.name,
            method.selector
        );
    }
}
