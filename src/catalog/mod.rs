//! The catalogue of the UGens of the core class library of SuperCollider
//! 3.14.1, and of its pseudo-UGens outside `UGen`: what each class takes and
//! gives, as `generator/catalog.scd` measured it in sclang.

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

/// A pseudo-UGen class outside `UGen`, such as `LinLin`: its rate methods
/// build other UGens, and scsynth has no unit of its name.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PseudoClass {
    /// The name of the class.
    pub name: &'static str,
    /// The categories of the class in the help of SuperCollider.
    pub categories: &'static [&'static str],
    /// The methods `ar`, `kr`, `ir` and `new` that the class responds to,
    /// but the `new` of `Object`.
    pub methods: &'static [RateMethod],
}

impl PseudoClass {
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
    /// A boolean, such as `wrap` of `SelectXFocus`.
    Bool,
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
    /// It can, with a body of the crate that adds what the method adds in
    /// sclang: the method of a pseudo-UGen.
    Pseudo,
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
    /// Whether the expansion goes on into an array inside an array, as
    /// `multiNewList` does.
    pub nested_expansion: bool,
    /// The position of the argument that goes through `asAudioRateInput`
    /// before the expansion, as the input of a delay at audio rate.
    pub audio_input: Option<usize>,
    /// Whether the method multiplies the result of the expansion by its
    /// argument `level`, as `LinXFade2` does.
    pub level: bool,
    /// When the method gives back its input in place of an instance, as the
    /// Lag family does for a number or a lag of 0.
    pub passthrough: Option<Passthrough>,
}

/// When a method gives back `in.madd(mul, add)` in place of an instance: for
/// an input at scalar rate, or for the number 0 in its zero argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Passthrough {
    /// The position of the input.
    pub input: usize,
    /// The position of the argument whose 0 also gives back the input, as
    /// `lagTime` of `Lag.kr`.
    pub zero: Option<usize>,
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

/// The pseudo-UGen class outside `UGen` with this name.
#[must_use]
pub fn pseudo_lookup(name: &str) -> Option<&'static PseudoClass> {
    let classes = pseudo_all();
    let position = classes
        .binary_search_by(|class| class.name.cmp(name))
        .ok()?;
    classes.get(position)
}

/// All the pseudo-UGen classes outside `UGen`, sorted by name.
#[must_use]
pub fn pseudo_all() -> &'static [PseudoClass] {
    &generated::PSEUDO_CLASSES
}

#[cfg(test)]
mod tests {
    use super::{
        Addable, CheckInputs, InputSlot, Outputs, Wiring, all, lookup, pseudo_all, pseudo_lookup,
    };
    use crate::rate::Rate;

    fn wiring(class: &str, selector: &str) -> Wiring {
        match lookup(class).unwrap().method(selector).unwrap().addable {
            Addable::Yes(wiring) => wiring,
            Addable::No(reason) => panic!("{class}.{selector}: {reason}"),
            Addable::Pseudo => panic!("{class}.{selector} is a pseudo-UGen"),
        }
    }

    fn reason(class: &str, selector: &str) -> &'static str {
        match lookup(class).unwrap().method(selector).unwrap().addable {
            Addable::No(reason) => reason,
            Addable::Yes(_) | Addable::Pseudo => panic!("{class}.{selector} is addable"),
        }
    }

    #[test]
    fn the_catalogue_holds_the_core_classes_sorted_by_name() {
        // Fact 29 of the plan of 0.1: 419 classes, without sc3-plugins.
        assert_eq!(all().len(), 419);
        assert!(all().windows(2).all(|pair| pair[0].name < pair[1].name));
        assert!(lookup("DFM1").is_none());
    }

    #[test]
    fn the_pseudo_classes_are_apart_and_sorted_by_name() {
        let classes = pseudo_all();
        assert!(classes.windows(2).all(|pair| pair[0].name < pair[1].name));
        assert!(classes.iter().all(|class| lookup(class.name).is_none()));
        let lin_lin = pseudo_lookup("LinLin").unwrap();
        assert_eq!(lin_lin.method("kr").unwrap().addable, Addable::Pseudo);
        assert!(lin_lin.method("new").is_none());
        assert!(pseudo_lookup("SinOsc").is_none());
        assert_eq!(
            lookup("Changed").unwrap().method("kr").unwrap().addable,
            Addable::Pseudo
        );
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
        // Fact 27 of the plan of 0.1.
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
    fn send_reply_expands_one_level_and_sin_osc_all_the_way() {
        // Fact 18 of the plan of 0.2: SendReply expands with flop.
        assert!(!wiring("SendReply", "kr").nested_expansion);
        assert!(!wiring("SendReply", "ar").nested_expansion);
        assert!(wiring("SinOsc", "ar").nested_expansion);
        let one_level = all()
            .iter()
            .flat_map(|class| class.methods)
            .filter(
                |method| matches!(method.addable, Addable::Yes(wiring) if !wiring.nested_expansion),
            )
            .count();
        assert_eq!(one_level, 2);
    }

    #[test]
    fn a_method_that_does_more_than_new1_is_not_addable() {
        // Fact 33 of the plan of 0.1.
        assert!(reason("VarLag", "kr").contains("no instance"));
        assert!(reason("LocalBuf", "new").contains("MaxLocalBufs"));
        assert!(reason("Poll", "kr").contains("Impulse"));
        assert_eq!(wiring("DelayN", "kr").rate, Rate::Control);
        assert!(reason("Duty", "kr").contains("rule of its own"));
    }

    #[test]
    fn the_delays_convert_their_input_and_lin_x_fade2_multiplies_its_level() {
        // Facts 24 and 25 of the plan of 0.3.
        assert_eq!(wiring("DelayN", "ar").audio_input, Some(0));
        assert_eq!(wiring("BufCombC", "ar").audio_input, Some(1));
        assert_eq!(wiring("DelTapWr", "ar").audio_input, Some(1));
        assert_eq!(wiring("DelayN", "kr").audio_input, None);
        assert!(wiring("LinXFade2", "kr").level);
        assert!(!wiring("XFade2", "kr").level);
        let converted = all()
            .iter()
            .flat_map(|class| class.methods)
            .filter(|method| matches!(method.addable, Addable::Yes(wiring) if wiring.audio_input.is_some()))
            .count();
        assert_eq!(converted, 19);
    }

    #[test]
    fn the_lag_family_gives_back_its_input_for_a_scalar_or_a_lag_of_0() {
        // Facts 1 and 2 of the plan of 0.4.
        let lag = wiring("Lag", "kr").passthrough.unwrap();
        assert_eq!((lag.input, lag.zero), (0, Some(1)));
        assert_eq!(wiring("Ramp", "ar").passthrough, Some(lag));
        assert_eq!(wiring("Lag3UD", "kr").passthrough.unwrap().zero, None);
        assert_eq!(wiring("Slew", "ar").passthrough.unwrap().zero, None);
        assert_eq!(wiring("LPF", "ar").passthrough, None);
        let passthroughs = all()
            .iter()
            .flat_map(|class| class.methods)
            .filter(|method| matches!(method.addable, Addable::Yes(wiring) if wiring.passthrough.is_some()))
            .count();
        assert_eq!(passthroughs, 16);
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
