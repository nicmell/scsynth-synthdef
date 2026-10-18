//! The catalogue of the UGens of the core class library of SuperCollider
//! 3.14.1: what each class takes and gives, as `generator/catalog.scd`
//! measured it in sclang.

#[rustfmt::skip]
mod generated;

use crate::rate::Rate;

/// A UGen class of the core class library.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct UGenClass {
    /// The name of the class: the name of the unit in scsynth.
    pub name: &'static str,
    /// The superclasses, from the parent up to `UGen`.
    pub superclasses: &'static [&'static str],
    /// Whether the class is a `PureUGen` or a `PureMultiOutUGen`. The
    /// `optimizeGraph` of sclang removes such a UGen when nothing reads it.
    pub is_pure: bool,
    /// Whether the class is a `WidthFirstUGen`.
    pub is_width_first: bool,
    /// The categories of the class in the help of SuperCollider, such as
    /// `UGens>Generators>Deterministic`.
    pub categories: &'static [&'static str],
    /// The rule of its `checkInputs`.
    pub check_inputs: CheckInputs,
    /// The methods `ar`, `kr`, `ir` and `new` that the class responds to and
    /// that a UGen class defines.
    pub methods: &'static [RateMethod],
}

impl UGenClass {
    /// The method with this selector.
    #[must_use]
    pub fn method(&self, selector: &str) -> Option<&'static RateMethod> {
        self.methods
            .iter()
            .find(|method| method.selector == selector)
    }

    /// The page of the class in the help of SuperCollider.
    #[must_use]
    pub fn doc_url(&self) -> String {
        format!("https://doc.sccode.org/Classes/{}.html", self.name)
    }
}

/// A rate method of a class: `ar`, `kr`, `ir` or `new`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct RateMethod {
    /// The selector.
    pub selector: &'static str,
    /// The arguments, in order.
    pub args: &'static [ArgSpec],
    /// Whether `SynthDefBuilder::ugen` can add the method by name, and how.
    pub addable: Addable,
}

/// An argument of a rate method.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ArgSpec {
    /// The name.
    pub name: &'static str,
    /// What the argument takes.
    pub kind: ArgKind,
    /// The default, from the `prototypeFrame` of the method. `None` is the
    /// nil of sclang: no default, or a default that sclang computes when the
    /// method runs.
    pub default: Option<ArgDefault>,
}

/// What an argument takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ArgKind {
    /// A signal: a number or an output.
    Signal,
    /// A positive integer number of channels.
    Count,
    /// An array of signals.
    Array,
    /// A symbol, such as the command name of `SendReply`.
    Symbol,
}

/// The default value of an argument.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ArgDefault {
    /// A number.
    Float(f64),
    /// A symbol.
    Symbol(&'static str),
    /// A boolean.
    Bool(bool),
}

/// Whether `SynthDefBuilder::ugen` can add a method by name.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Addable {
    /// It can, with this wiring.
    Yes(Wiring),
    /// It cannot, for this reason: the probe of the generator found a
    /// behavior that the wiring does not describe.
    No(&'static str),
}

/// What a rate method gives `UGen:new1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Wiring {
    /// The rate of the UGen.
    pub rate: Rate,
    /// The inputs of the wire, in order.
    pub inputs: &'static [InputSlot],
    /// The outputs.
    pub outputs: Outputs,
    /// The special index.
    pub special_index: i16,
    /// Whether the method applies `madd` with its arguments `mul` and `add`.
    pub madd: bool,
}

/// Where inputs of the wire come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InputSlot {
    /// One input: the argument with this position.
    Arg(usize),
    /// One input per element of the array argument with this position.
    ArgArray(usize),
    /// The symbol argument with this position, as its size and then its
    /// ASCII codes.
    ArgString(usize),
}

/// The outputs of a UGen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outputs {
    /// These outputs, by rate.
    Fixed(&'static [Rate]),
    /// As many outputs as the count argument with this position gives, at
    /// one rate.
    Arg {
        /// The position of the count argument.
        arg: usize,
        /// The rate of each output.
        rate: Rate,
    },
    /// One output per element of the array argument with this position, at
    /// one rate.
    ArraySize {
        /// The position of the array argument.
        arg: usize,
        /// The rate of each output.
        rate: Rate,
    },
}

/// The rule of `checkInputs` of a class, as sclang checks it at the end of
/// the build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CheckInputs {
    /// `checkValidInputs`: no input is NaN.
    Valid,
    /// The first input has the rate of the UGen.
    SameRateAsFirstInput,
    /// At audio rate, the first input is at audio rate.
    SameRateAsFirstInputIfAudio,
    /// At audio rate, the first n inputs are at audio rate.
    NInputs(usize),
    /// At audio rate, the input with this position is at audio rate.
    AudioAt(usize),
    /// At audio rate, the inputs from this position on are at audio rate.
    AudioFrom(usize),
    /// The rule of `AbstractOut`. At audio rate, the inputs after the fixed
    /// ones are at audio rate. At another rate, at least one input follows
    /// them.
    AbstractOut {
        /// The number of fixed inputs, such as the bus.
        fixed: usize,
    },
    /// A rule of its own, defined in the class with this name.
    Custom(&'static str),
}

/// The class with this name.
#[must_use]
pub fn lookup(name: &str) -> Option<&'static UGenClass> {
    let classes = all();
    let position = classes
        .binary_search_by(|class| class.name.cmp(name))
        .ok()?;
    classes.get(position)
}

/// All the classes, sorted by name.
#[must_use]
pub fn all() -> &'static [UGenClass] {
    &generated::CLASSES
}

#[cfg(test)]
mod tests {
    use super::{Addable, CheckInputs, InputSlot, Outputs, Wiring, all, lookup};
    use crate::rate::Rate;

    fn wiring(class: &str, selector: &str) -> Wiring {
        match lookup(class).unwrap().method(selector).unwrap().addable {
            Addable::Yes(wiring) => wiring,
            Addable::No(reason) => panic!("{class}.{selector}: {reason}"),
        }
    }

    fn reason(class: &str, selector: &str) -> &'static str {
        match lookup(class).unwrap().method(selector).unwrap().addable {
            Addable::No(reason) => reason,
            Addable::Yes(_) => panic!("{class}.{selector} is addable"),
        }
    }

    #[test]
    fn the_catalogue_holds_the_core_classes_sorted_by_name() {
        // Fact 29 of the plan: 419 classes, without sc3-plugins.
        assert_eq!(all().len(), 419);
        assert!(all().windows(2).all(|pair| pair[0].name < pair[1].name));
        assert!(lookup("DFM1").is_none());
    }

    #[test]
    fn sin_osc_takes_freq_and_phase_and_applies_madd() {
        let class = lookup("SinOsc").unwrap();
        assert_eq!(class.categories, ["UGens>Generators>Deterministic"]);
        assert_eq!(
            class.doc_url(),
            "https://doc.sccode.org/Classes/SinOsc.html"
        );
        assert!(class.is_pure);
        let ar = wiring("SinOsc", "ar");
        assert_eq!(ar.rate, Rate::Audio);
        assert_eq!(ar.inputs, [InputSlot::Arg(0), InputSlot::Arg(1)]);
        assert_eq!(ar.outputs, Outputs::Fixed(&[Rate::Audio]));
        assert!(ar.madd);
    }

    #[test]
    fn the_outputs_follow_a_count_or_an_array() {
        assert_eq!(
            wiring("In", "ar").outputs,
            Outputs::Arg {
                arg: 1,
                rate: Rate::Audio
            }
        );
        assert_eq!(
            wiring("Demand", "kr").outputs,
            Outputs::ArraySize {
                arg: 2,
                rate: Rate::Control
            }
        );
    }

    #[test]
    fn the_slots_follow_the_wire_of_sclang() {
        // Fact 27 of the plan.
        assert_eq!(
            wiring("SendReply", "kr").inputs,
            [
                InputSlot::Arg(0),
                InputSlot::Arg(3),
                InputSlot::ArgString(1),
                InputSlot::ArgArray(2)
            ]
        );
        let envelope = wiring("EnvGen", "kr").inputs;
        assert_eq!(envelope.last(), Some(&InputSlot::ArgArray(0)));
    }

    #[test]
    fn a_method_that_does_more_than_new1_is_not_addable() {
        // Fact 33 of the plan.
        assert!(reason("Lag", "kr").contains("no instance"));
        assert!(reason("DelayN", "ar").contains("DC"));
        assert!(reason("LocalBuf", "new").contains("MaxLocalBufs"));
        assert!(reason("Poll", "kr").contains("Impulse"));
        assert_eq!(wiring("DelayN", "kr").rate, Rate::Control);
        assert!(reason("Duty", "kr").contains("rule of its own"));
    }

    #[test]
    fn each_class_has_the_rule_of_its_check_inputs() {
        assert_eq!(
            lookup("Out").unwrap().check_inputs,
            CheckInputs::AbstractOut { fixed: 1 }
        );
        assert_eq!(
            lookup("XOut").unwrap().check_inputs,
            CheckInputs::AbstractOut { fixed: 2 }
        );
        assert_eq!(
            lookup("LPF").unwrap().check_inputs,
            CheckInputs::SameRateAsFirstInput
        );
        assert_eq!(
            lookup("Pan2").unwrap().check_inputs,
            CheckInputs::NInputs(1)
        );
        assert_eq!(
            lookup("Duty").unwrap().check_inputs,
            CheckInputs::Custom("Duty")
        );
    }
}
