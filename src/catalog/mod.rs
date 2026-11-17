//! The catalogue of the UGens of the core class library of SuperCollider
//! 3.14.1, and of its pseudo-UGens outside `UGen`: what each class takes and
//! gives, as `generator/catalog.scd` measured it in sclang. With the feature
//! `sc3-plugins`, the module `sc3_plugins` holds the classes of sc3-plugins.

#[rustfmt::skip]
mod generated;

use crate::rate::Rate;
#[cfg(feature = "sc3-plugins")]
use crate::sc3_plugins::{lookup as plugin_lookup, pseudo_lookup as plugin_pseudo_lookup};

/// A UGen class of the core class library, or of sc3-plugins.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct UGenClass {
    /// The name of the class: the name of the unit in scsynth.
    pub name: &'static str,
    /// The superclasses, from the parent up to `UGen`.
    pub superclasses: &'static [&'static str],
    /// Whether the class is a `PureUGen` or a `PureMultiOutUGen`.
    pub is_pure: bool,
    /// What `optimizeGraph` of sclang does with a UGen of the class.
    pub optimize: Optimize,
    /// Whether the class is a `WidthFirstUGen`.
    pub is_width_first: bool,
    /// How `fftSize` finds the size of the FFT of a PV chain of the class.
    /// `None` for a class without `fftSize`.
    pub fft_size: Option<FftSize>,
    /// How `convertEnv` takes an `Env` as the envelope of the class. `None`
    /// for a class without `convertEnv`.
    pub convert_env: Option<ConvertEnv>,
    /// Whether the plugin of the unit sets `kUnitDef_CantAliasInputsToOutputs`,
    /// so that the unit takes buffers for its outputs before it frees those of
    /// its inputs, in the count of
    /// [`SynthDef::wire_buffers`](crate::SynthDef::wire_buffers).
    pub cant_alias: bool,
    /// The categories of the class in the help of SuperCollider, such as
    /// `UGens>Generators>Deterministic`.
    pub categories: &'static [&'static str],
    /// The rule of its `checkInputs`.
    pub check_inputs: CheckInputs,
    /// What `signalRange` gives for a UGen of the class, which `range`,
    /// `exprange` and `curverange` read.
    pub signal_range: SignalRange,
    /// The methods `ar`, `kr`, `ir` and `new` that the class responds to and
    /// that a UGen class defines.
    pub methods: &'static [RateMethod],
}

impl UGenClass {
    /// The method with this selector.
    #[must_use]
    pub fn method(&self, selector: &str) -> Option<&'static RateMethod> {
        by_selector(self.methods, selector)
    }

    /// The page of the class in the help of SuperCollider.
    #[must_use]
    pub fn doc_url(&self) -> String {
        doc_url(self.name)
    }
}

/// The range of the values of a UGen, as `signalRange` gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SignalRange {
    /// From -1 to 1, as `SinOsc` gives them.
    Bipolar,
    /// From 0 to 1, as `LFPulse` gives them.
    Unipolar,
}

/// An instance method of `UGen` that builds a graph, such as `range`, with
/// the classes whose methods a number and an array answer in its place.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct UGenMethod {
    /// The selector, and the arguments with the defaults of the method of
    /// `UGen`.
    pub method: RateMethod,
    /// The class whose method a Float answers, such as `SimpleNumber`.
    /// `None` where a Float does not understand the message.
    pub number: Option<&'static str>,
    /// The class whose method an `Array` answers, such as
    /// `SequenceableCollection`. `None` where an `Array` does not understand
    /// the message.
    pub array: Option<&'static str>,
    /// The subclasses of `UGen` with a method of their own, such as `DUGen`
    /// for `linlin`.
    pub overrides: &'static [&'static str],
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
    /// The methods `ar`, `kr`, `ir`, `tr` and `new` that the class responds
    /// to, but the `new` of `Object`.
    pub methods: &'static [RateMethod],
}

impl PseudoClass {
    /// The method with this selector.
    #[must_use]
    pub fn method(&self, selector: &str) -> Option<&'static RateMethod> {
        by_selector(self.methods, selector)
    }

    /// The page of the class in the help of SuperCollider.
    #[must_use]
    pub fn doc_url(&self) -> String {
        doc_url(self.name)
    }
}

/// A method of the catalogue: a rate method of a class, such as `ar`, `kr`,
/// `ir`, `tr` or `new`, or a method of `Env`, of `PV_ChainUGen` or of `UGen`
/// that builds a graph.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct RateMethod {
    /// The selector.
    pub selector: &'static str,
    /// The arguments, in order.
    pub args: &'static [ArgSpec],
    /// How the crate adds the method. For a rate method, whether
    /// `SynthDefBuilder::ugen` can add it by name. Every other method is
    /// `Pseudo`.
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
    /// A curve of an envelope, such as the `curve` of `Env.perc`: a shape
    /// by name, a value, or an array of them.
    Curve,
    /// A function, such as the `func` of `pvcalc`. The methods of the
    /// builder take a closure in its place.
    Function,
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
    /// An array of numbers, such as the `levels` of `Env.new`.
    Floats(&'static [f64]),
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
    /// A body of the crate adds what the method adds in sclang: the method
    /// of a pseudo-UGen, which `SynthDefBuilder::ugen` takes, or a method of
    /// `Env`, of `PV_ChainUGen` or of `UGen`, which a call of its own takes.
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
    /// The positions of the arguments that go to audio rate together before
    /// the expansion, as the `gate` and the `reset` of `DemandEnvGen.ar`:
    /// when one of them is at audio rate, `K2A.ar` takes each other one.
    pub audio_together: Option<&'static [usize]>,
    /// Whether the method multiplies the result of the expansion by its
    /// argument `level`, as `LinXFade2` does.
    pub level: bool,
    /// When the method gives back its input in place of an instance, as the
    /// Lag family does for a number or a lag of 0.
    pub passthrough: Option<Passthrough>,
    /// The position of the argument whose zeros become silence before the
    /// expansion, through `replaceZeroesWithSilence`, as the channels of
    /// `Out.ar`.
    pub silence: Option<usize>,
    /// How the array argument takes an array among its elements. `None` for
    /// a method without an array argument, and for one whose array stays
    /// among the inputs of the UGen, as in `IEnvGen`.
    pub array_expansion: Option<ArrayExpansion>,
}

/// How the array argument of a method takes an array among its elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ArrayExpansion {
    /// Each element is an argument of `multiNewList`, as `Out.ar` hands it
    /// `channelsArray`: an element that is an array expands the UGen.
    Elements,
    /// With an array among the elements, each element is the array of one
    /// instance, as the `flop` of `SendReply.kr` makes it.
    Flop,
    /// With an array as the first element, each element is the array of one
    /// instance, as the `Ref`s of `EnvGen:convertEnv` make it.
    Refs,
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
    /// The rule of `BufWr`. At audio rate, the phase, input 1, and the
    /// channels, from input 3 on, are at audio rate.
    BufWr,
    /// The rule of `Duty`. When the first input, `dur`, runs at demand rate,
    /// the second, `reset`, runs at demand rate, at scalar rate, or at the
    /// rate of the UGen.
    Duty,
    /// The rule of `Peak`. The first input has the rate of the UGen, or runs
    /// at audio rate for a UGen at control rate.
    Peak,
    /// The rule of `T2K`: the first input is at audio rate.
    T2K,
    /// Exactly this many inputs, at any rate, as `FM7` of sc3-plugins asks.
    InputCount(usize),
    /// A rule of its own, defined in the class with this name, which the
    /// crate does not have.
    Custom(&'static str),
}

/// What `optimizeGraph` of sclang does with a UGen, by the class that owns
/// the method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Optimize {
    /// Nothing, as `UGen` and `DetectSilence` do.
    Nothing,
    /// It removes the UGen when nothing reads it, as `PureUGen`,
    /// `PureMultiOutUGen` and `UnaryOpUGen` do.
    DeadCode,
    /// It removes the UGen when nothing reads it, and else fuses a `+` or a
    /// `-` with its inputs, as `BinaryOpUGen` does.
    BinaryOp,
}

/// How `fftSize` of a PV chain finds the size of its FFT, which a copy of
/// `addCopiesIfNeeded` gives its `LocalBuf`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FftSize {
    /// A new `BufFrames.ir` of input 0, the buffer of an `FFT`.
    BufFrames,
    /// Input 1, the `bufsize` of a `PackFFT`.
    Input,
    /// The size of the chain of input 0, as `PV_ChainUGen` finds it.
    Chain,
}

/// How `convertEnv` of sclang takes an `Env` as the envelope of a UGen, by
/// the class that owns the method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConvertEnv {
    /// Through `asMultichannelArray`, as `EnvGen` does.
    Array,
    /// Through `asArrayForInterpolation`, as `IEnvGen` does.
    Interpolation,
}

/// A `ControlSpec` of `Spec.specs` in sclang 3.14.1: the spec of a name,
/// as `\freq.asSpec` gives it. Its default is the default of a
/// `NamedControl` with the spec and no values, and of an argument without a
/// default when the metadata of the definition has specs.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ControlSpec {
    /// The name, such as `freq`.
    pub name: &'static str,
    /// The value at the start of the range.
    pub minval: f64,
    /// The value at the end of the range.
    pub maxval: f64,
    /// The warp, as `asSpecifier` names it, such as `linear` or `exp`.
    pub warp: &'static str,
    /// The step that the values round to, or 0.
    pub step: f64,
    /// The default value.
    pub default: f64,
    /// The units, such as `Hz`, or nothing.
    pub units: &'static str,
}

fn by_selector(methods: &'static [RateMethod], selector: &str) -> Option<&'static RateMethod> {
    methods.iter().find(|method| method.selector == selector)
}

pub(crate) fn by_name<T>(
    items: &'static [T],
    name: &str,
    key: fn(&T) -> &str,
) -> Option<&'static T> {
    let position = items.binary_search_by(|item| key(item).cmp(name)).ok()?;
    items.get(position)
}

fn doc_url(class: &str) -> String {
    format!("https://doc.sccode.org/Classes/{class}.html")
}

/// The class with this name.
#[must_use]
pub fn lookup(name: &str) -> Option<&'static UGenClass> {
    by_name(all(), name, |class| class.name)
}

/// The class with this name, of the core or, with the feature
/// `sc3-plugins`, of sc3-plugins.
pub(crate) fn find(name: &str) -> Option<&'static UGenClass> {
    lookup(name).or_else(|| plugin_lookup(name))
}

/// The pseudo-UGen class outside `UGen` with this name, of the core or,
/// with the feature `sc3-plugins`, of sc3-plugins.
pub(crate) fn find_pseudo(name: &str) -> Option<&'static PseudoClass> {
    pseudo_lookup(name).or_else(|| plugin_pseudo_lookup(name))
}

#[cfg(not(feature = "sc3-plugins"))]
fn plugin_lookup(_: &str) -> Option<&'static UGenClass> {
    None
}

#[cfg(not(feature = "sc3-plugins"))]
fn plugin_pseudo_lookup(_: &str) -> Option<&'static PseudoClass> {
    None
}

/// All the classes, sorted by name.
#[must_use]
pub fn all() -> &'static [UGenClass] {
    &generated::CLASSES
}

/// The pseudo-UGen class outside `UGen` with this name.
#[must_use]
pub fn pseudo_lookup(name: &str) -> Option<&'static PseudoClass> {
    by_name(pseudo_all(), name, |class| class.name)
}

/// All the pseudo-UGen classes outside `UGen`, sorted by name.
#[must_use]
pub fn pseudo_all() -> &'static [PseudoClass] {
    &generated::PSEUDO_CLASSES
}

/// The spec of `Spec.specs` with this name.
#[must_use]
pub fn spec(name: &str) -> Option<&'static ControlSpec> {
    by_name(specs(), name, |spec| spec.name)
}

/// All the specs of `Spec.specs`, sorted by name.
#[must_use]
pub fn specs() -> &'static [ControlSpec] {
    &generated::SPECS
}

/// The class method of `Env` with this selector that makes an envelope,
/// such as `perc`.
#[must_use]
pub fn env_method(selector: &str) -> Option<&'static RateMethod> {
    by_selector(env_methods(), selector)
}

/// The class methods of `Env` that make an envelope, with their arguments.
#[must_use]
pub fn env_methods() -> &'static [RateMethod] {
    &generated::ENV_METHODS
}

/// The instance method of `Env` with this selector that maps its levels to
/// a range, such as `exprange`.
#[must_use]
pub fn env_range(selector: &str) -> Option<&'static RateMethod> {
    by_selector(env_ranges(), selector)
}

/// The instance methods of `Env` that map its levels to a range, with their
/// arguments.
#[must_use]
pub fn env_ranges() -> &'static [RateMethod] {
    &generated::ENV_RANGES
}

/// The instance method of `PV_ChainUGen` with this selector that takes a
/// function, such as `pvcalc`.
#[must_use]
pub fn pv_method(selector: &str) -> Option<&'static RateMethod> {
    by_selector(pv_methods(), selector)
}

/// The instance methods of `PV_ChainUGen` that take a function, with their
/// arguments.
#[must_use]
pub fn pv_methods() -> &'static [RateMethod] {
    &generated::PV_METHODS
}

/// The instance method of `UGen` with this selector that builds a graph,
/// such as `range`.
#[must_use]
pub fn ugen_method(selector: &str) -> Option<&'static UGenMethod> {
    ugen_methods()
        .iter()
        .find(|method| method.method.selector == selector)
}

/// The instance methods of `UGen` that build graphs, with their arguments.
#[must_use]
pub fn ugen_methods() -> &'static [UGenMethod] {
    &generated::UGEN_METHODS
}

#[cfg(test)]
mod tests {
    use super::{
        Addable, ArgDefault, ArgKind, ArrayExpansion, CheckInputs, ConvertEnv, FftSize, InputSlot,
        Optimize, Outputs, SignalRange, UGenMethod, Wiring, all, env_method, env_methods,
        env_range, env_ranges, lookup, pseudo_all, pseudo_lookup, pv_method, pv_methods, spec,
        specs, ugen_method, ugen_methods,
    };
    use crate::rate::Rate;

    fn wiring(class: &str, selector: &str) -> Wiring {
        match lookup(class).unwrap().method(selector).unwrap().addable {
            Addable::Yes(wiring) => wiring,
            Addable::No(reason) => panic!("{class}.{selector}: {reason}"),
            Addable::Pseudo => panic!("{class}.{selector} is a pseudo-UGen"),
        }
    }

    /// Every wiring of the methods of the classes.
    fn wirings() -> impl Iterator<Item = Wiring> {
        all()
            .iter()
            .flat_map(|class| class.methods)
            .filter_map(|method| match method.addable {
                Addable::Yes(wiring) => Some(wiring),
                Addable::No(_) | Addable::Pseudo => None,
            })
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
        let mut counts = [0; 3];
        for method in all().iter().flat_map(|class| class.methods) {
            counts[match method.addable {
                Addable::Yes(_) => 0,
                Addable::Pseudo => 1,
                Addable::No(_) => 2,
            }] += 1;
        }
        assert_eq!(counts, [581, 36, 18]);
        let pseudo_methods: usize = pseudo_all().iter().map(|class| class.methods.len()).sum();
        assert_eq!(counts[1] + pseudo_methods, 67);
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
        let one_level = wirings().filter(|wiring| !wiring.nested_expansion);
        assert_eq!(one_level.count(), 2);
    }

    #[test]
    fn a_method_that_does_more_than_new1_is_not_addable() {
        // Fact 33 of the plan of 0.1.
        assert!(reason("Dunique", "new").contains("no instance"));
        assert!(reason("MaxLocalBufs", "new").contains("no argument"));
        assert_eq!(wiring("DelayN", "kr").rate, Rate::Control);
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
        let converted = wirings().filter(|wiring| wiring.audio_input.is_some());
        assert_eq!(converted.count(), 19);
    }

    #[test]
    fn the_methods_that_take_a_function_are_those_of_pv_chain_ugen() {
        // Fact 7 of the plan of 0.11.
        let selectors: Vec<&str> = pv_methods().iter().map(|method| method.selector).collect();
        assert_eq!(selectors, ["pvcalc", "pvcalc2", "pvcollect"]);
        let pvcalc = pv_method("pvcalc").unwrap();
        let args: Vec<(&str, ArgKind, Option<ArgDefault>)> = pvcalc
            .args
            .iter()
            .map(|arg| (arg.name, arg.kind, arg.default))
            .collect();
        assert_eq!(
            args,
            [
                ("numframes", ArgKind::Signal, None),
                ("func", ArgKind::Function, None),
                ("frombin", ArgKind::Signal, Some(ArgDefault::Float(0.0))),
                ("tobin", ArgKind::Signal, None),
                ("zeroothers", ArgKind::Signal, Some(ArgDefault::Float(0.0))),
            ]
        );
        assert_eq!(pv_method("pvcalc2").unwrap().args[0].name, "chain2");
        assert!(pv_method("fftSize").is_none());
    }

    #[test]
    fn the_ranges_of_env_are_range_exprange_and_curverange() {
        // Fact 12 of the plan of 0.12.
        let ranges: Vec<(&str, Vec<Option<ArgDefault>>)> = env_ranges()
            .iter()
            .map(|method| {
                let defaults = method.args.iter().map(|arg| arg.default).collect();
                (method.selector, defaults)
            })
            .collect();
        let float = |value| Some(ArgDefault::Float(value));
        assert_eq!(
            ranges,
            [
                ("range", vec![float(0.0), float(1.0)]),
                ("exprange", vec![float(0.01), float(1.0)]),
                ("curverange", vec![float(0.0), float(1.0), float(-4.0)]),
            ]
        );
        assert!(env_range("delay").is_none());
    }

    #[test]
    fn the_methods_of_ugen_that_build_graphs_are_these_42() {
        // Fact 1 of the plan of 0.12.
        let selectors: Vec<&str> = ugen_methods()
            .iter()
            .map(|method| method.method.selector)
            .collect();
        assert_eq!(
            selectors,
            [
                "range",
                "exprange",
                "curverange",
                "unipolar",
                "bipolar",
                "linlin",
                "linexp",
                "explin",
                "expexp",
                "lincurve",
                "curvelin",
                "bilin",
                "prune",
                "clip",
                "fold",
                "wrap",
                "lag",
                "lag2",
                "lag3",
                "lagud",
                "lag2ud",
                "lag3ud",
                "varlag",
                "slew",
                "blend",
                "minNyquist",
                "degrad",
                "raddeg",
                "moddif",
                "sanitize",
                "poll",
                "dpoll",
                "checkBadValues",
                "snap",
                "softRound",
                "binaryValue",
                "isPositive",
                "isNegative",
                "isStrictlyPositive",
                "|==|",
                "degreeToKey",
                "if",
            ]
        );
        let linlin = ugen_method("linlin").unwrap();
        let args: Vec<(&str, ArgKind, Option<ArgDefault>)> = linlin
            .method
            .args
            .iter()
            .map(|arg| (arg.name, arg.kind, arg.default))
            .collect();
        assert_eq!(
            args,
            [
                ("inMin", ArgKind::Signal, None),
                ("inMax", ArgKind::Signal, None),
                ("outMin", ArgKind::Signal, None),
                ("outMax", ArgKind::Signal, None),
                ("clip", ArgKind::Symbol, Some(ArgDefault::Symbol("minmax"))),
            ]
        );
        assert!(ugen_method("madd").is_none());
    }

    /// The selectors of the methods whose receiver of one kind answers the
    /// method of `owner`.
    fn answered(kind: fn(&UGenMethod) -> Option<&str>, owner: Option<&str>) -> Vec<&'static str> {
        ugen_methods()
            .iter()
            .filter(|method| kind(method) == owner)
            .map(|method| method.method.selector)
            .collect()
    }

    #[test]
    fn a_number_answers_31_of_the_methods() {
        // Fact 10 of the plan of 0.12.
        let number = |method: &UGenMethod| method.number;
        assert_eq!(
            answered(number, None),
            [
                "range",
                "exprange",
                "curverange",
                "unipolar",
                "bipolar",
                "prune",
                "minNyquist",
                "sanitize",
                "dpoll",
                "checkBadValues",
                "if",
            ]
        );
        assert_eq!(answered(number, Some("Float")), ["clip", "fold", "wrap"]);
        assert_eq!(answered(number, Some("Object")), ["blend", "|==|"]);
        assert_eq!(answered(number, Some("SimpleNumber")).len(), 26);
    }

    #[test]
    fn an_array_answers_38_of_the_methods() {
        // Fact 11 of the plan of 0.12.
        let array = |method: &UGenMethod| method.array;
        assert_eq!(
            answered(array, None),
            ["snap", "softRound", "binaryValue", "if"]
        );
        assert_eq!(answered(array, Some("Array")), ["poll", "dpoll"]);
        assert_eq!(answered(array, Some("Object")), ["|==|"]);
        assert_eq!(answered(array, Some("SequenceableCollection")).len(), 35);
    }

    #[test]
    fn dugen_and_lfgauss_have_methods_of_their_own() {
        // Fact 2 of the plan of 0.12.
        let overridden: Vec<(&str, &[&str])> = ugen_methods()
            .iter()
            .filter(|method| !method.overrides.is_empty())
            .map(|method| (method.method.selector, method.overrides))
            .collect();
        let dugen: &[&str] = &["DUGen"];
        assert_eq!(
            overridden,
            [
                ("range", &["LFGauss"][..]),
                ("linlin", dugen),
                ("linexp", dugen),
                ("explin", dugen),
                ("expexp", dugen),
            ]
        );
    }

    #[test]
    fn sixteen_classes_are_unipolar() {
        // Fact 4 of the plan of 0.12: 13 classes, and 3 that inherit.
        let unipolar: Vec<&str> = all()
            .iter()
            .filter(|class| class.signal_range == SignalRange::Unipolar)
            .map(|class| class.name)
            .collect();
        assert_eq!(
            unipolar,
            [
                "AnalogIn",
                "DigitalIO",
                "DigitalIn",
                "Dust",
                "Impulse",
                "KeyState",
                "LFPulse",
                "MouseButton",
                "MouseX",
                "MouseY",
                "MultiplexAnalogIn",
                "SetResetFF",
                "TDelay",
                "ToggleFF",
                "Trig",
                "Trig1",
            ]
        );
    }

    #[test]
    fn the_units_that_cannot_alias_are_the_18_of_the_plugins() {
        // Fact 2 of the plan of 0.10.
        let flagged: Vec<&str> = all()
            .iter()
            .filter(|class| class.cant_alias)
            .map(|class| class.name)
            .collect();
        assert_eq!(
            flagged,
            [
                "Balance2",
                "BeatTrack",
                "BeatTrack2",
                "DecodeB2",
                "Demand",
                "DemandEnvGen",
                "Duty",
                "GrainBuf",
                "GrainFM",
                "GrainIn",
                "GrainSin",
                "PanAz",
                "PanB2",
                "PartConv",
                "PlayBuf",
                "TDuty",
                "TGrains",
                "Warp1",
            ]
        );
    }

    #[test]
    fn demand_env_gen_ar_takes_its_gate_and_reset_to_audio_rate_together() {
        // Fact 6 of the plan of 0.10.
        assert_eq!(
            wiring("DemandEnvGen", "ar").audio_together,
            Some(&[4, 5][..])
        );
        assert_eq!(wiring("DemandEnvGen", "kr").audio_together, None);
        let together = wirings().filter(|wiring| wiring.audio_together.is_some());
        assert_eq!(together.count(), 1);
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
        let passthroughs = wirings().filter(|wiring| wiring.passthrough.is_some());
        assert_eq!(passthroughs.count(), 16);
    }

    #[test]
    fn the_zeros_of_the_outputs_at_audio_rate_become_silence() {
        // Fact 7 of the plan of 0.4.
        assert_eq!(wiring("Out", "ar").silence, Some(1));
        assert_eq!(wiring("OffsetOut", "ar").silence, Some(1));
        assert_eq!(wiring("LocalOut", "ar").silence, Some(0));
        assert_eq!(wiring("XOut", "ar").silence, Some(2));
        assert_eq!(wiring("Out", "kr").silence, None);
        let silenced = wirings().filter(|wiring| wiring.silence.is_some());
        assert_eq!(silenced.count(), 5);
    }

    #[test]
    fn each_array_argument_takes_an_array_as_its_class_does() {
        // Facts 2 to 5 of the plan of 0.6.
        assert_eq!(
            wiring("Out", "ar").array_expansion,
            Some(ArrayExpansion::Elements)
        );
        assert_eq!(
            wiring("SendReply", "kr").array_expansion,
            Some(ArrayExpansion::Flop)
        );
        assert_eq!(
            wiring("EnvGen", "kr").array_expansion,
            Some(ArrayExpansion::Refs)
        );
        assert_eq!(wiring("IEnvGen", "kr").array_expansion, None);
        assert_eq!(wiring("SinOsc", "ar").array_expansion, None);
        let arrays: Vec<Option<ArrayExpansion>> = wirings()
            .filter(|wiring| {
                let array = |slot: &InputSlot| matches!(slot, InputSlot::ArgArray(_));
                wiring.inputs.iter().any(array)
            })
            .map(|wiring| wiring.array_expansion)
            .collect();
        let count = |expansion| arrays.iter().filter(|&&found| found == expansion).count();
        assert_eq!(
            [
                count(Some(ArrayExpansion::Elements)),
                count(Some(ArrayExpansion::Flop)),
                count(Some(ArrayExpansion::Refs)),
                count(None)
            ],
            [34, 2, 2, 2]
        );
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
        assert_eq!(lookup("Duty").unwrap().check_inputs, CheckInputs::Duty);
        // The rules of their own, which subclasses inherit: facts 1 to 4 of
        // the plan of 0.5.
        assert_eq!(lookup("TDuty").unwrap().check_inputs, CheckInputs::Duty);
        assert_eq!(
            lookup("RunningMin").unwrap().check_inputs,
            CheckInputs::Peak
        );
        assert!(
            all()
                .iter()
                .all(|class| !matches!(class.check_inputs, CheckInputs::Custom(_)))
        );
    }

    #[test]
    fn convert_env_and_the_methods_of_env_are_those_of_sclang() {
        let converting: Vec<(&str, ConvertEnv)> = all()
            .iter()
            .filter_map(|class| class.convert_env.map(|form| (class.name, form)))
            .collect();
        assert_eq!(
            converting,
            [
                ("EnvGen", ConvertEnv::Array),
                ("IEnvGen", ConvertEnv::Interpolation)
            ]
        );
        assert_eq!(env_methods().len(), 14);
        let perc = env_method("perc").unwrap();
        let curve = perc.args.iter().find(|arg| arg.name == "curve").unwrap();
        assert_eq!(
            (curve.kind, curve.default),
            (ArgKind::Curve, Some(ArgDefault::Float(-4.0)))
        );
        let new = env_method("new").unwrap();
        assert_eq!(
            new.args[0].default,
            Some(ArgDefault::Floats(&[0.0, 1.0, 0.0]))
        );
        assert!(env_method("shapeNumber").is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn the_specs_are_the_23_of_sclang() {
        assert_eq!(specs().len(), 23);
        let freq = spec("freq").unwrap();
        assert_eq!(
            (
                freq.minval,
                freq.maxval,
                freq.warp,
                freq.default,
                freq.units
            ),
            (20.0, 20000.0, "exp", 440.0, "Hz")
        );
        let db = spec("db").unwrap();
        assert_eq!(
            (db.minval, db.default),
            (f64::NEG_INFINITY, f64::NEG_INFINITY)
        );
        assert_eq!(spec("rq").unwrap().default, 0.707);
        assert!(spec("nosuchspec").is_none());
    }

    #[test]
    fn each_class_optimizes_as_its_optimize_graph_does() {
        // Fact 6 of the plan of 0.7.
        assert_eq!(lookup("SinOsc").unwrap().optimize, Optimize::DeadCode);
        assert_eq!(lookup("DC").unwrap().optimize, Optimize::DeadCode);
        assert_eq!(lookup("UnaryOpUGen").unwrap().optimize, Optimize::DeadCode);
        assert_eq!(lookup("BinaryOpUGen").unwrap().optimize, Optimize::BinaryOp);
        assert_eq!(lookup("DetectSilence").unwrap().optimize, Optimize::Nothing);
        assert_eq!(lookup("Out").unwrap().optimize, Optimize::Nothing);
        let count = |optimize| {
            all()
                .iter()
                .filter(|class| class.optimize == optimize)
                .count()
        };
        assert_eq!(
            [
                count(Optimize::Nothing),
                count(Optimize::DeadCode),
                count(Optimize::BinaryOp)
            ],
            [319, 99, 1]
        );
    }

    #[test]
    fn each_pv_chain_finds_its_fft_size_as_its_class_does() {
        // Fact 3 of the plan of 0.7.
        assert_eq!(lookup("FFT").unwrap().fft_size, Some(FftSize::BufFrames));
        assert_eq!(lookup("PackFFT").unwrap().fft_size, Some(FftSize::Input));
        assert_eq!(
            lookup("PV_MagAbove").unwrap().fft_size,
            Some(FftSize::Chain)
        );
        assert_eq!(lookup("IFFT").unwrap().fft_size, None);
        let count = |size| all().iter().filter(|class| class.fft_size == size).count();
        assert_eq!(
            [
                count(Some(FftSize::BufFrames)),
                count(Some(FftSize::Input)),
                count(Some(FftSize::Chain))
            ],
            [1, 1, 36]
        );
    }
}
