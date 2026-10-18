//! The build of a synth definition, as sclang builds a `SynthDef`.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::catalog::{
    self, Addable, ArgDefault, ArgKind, InputSlot, Outputs, RateMethod, UGenClass, Wiring,
};
use crate::check_inputs;
use crate::error::BuildError;
use crate::file::Version;
use crate::limits;
use crate::operator;
use crate::rate::Rate;
use crate::synth_def::{ControlName, InputSpec, SynthDef, UGen, Variant};
use crate::topo;
use crate::ugen_input::{ArgValue, OutputProxy, UGenInput};

static NEXT_BUILD: AtomicU64 = AtomicU64::new(0);

/// The rate entry of an argument, as in the `rates` of `SynthDef.new`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArgRate {
    /// `\ir`: a scalar control, set when the synth starts.
    Ir,
    /// `\tr`: a trigger control.
    Tr,
    /// `\ar`: an audio-rate control.
    Ar,
    /// `\kr`: a control-rate control, the rate of an argument without an
    /// entry.
    Kr,
}

/// An argument of the graph function: a parameter of the definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    name: String,
    values: Vec<f64>,
    array: bool,
    rate: Option<ArgRate>,
    lags: Vec<f64>,
}

impl Arg {
    /// An argument with one default value.
    pub fn new(name: impl Into<String>, default: f64) -> Self {
        Self {
            name: name.into(),
            values: vec![default],
            array: false,
            rate: None,
            lags: Vec::new(),
        }
    }

    /// An argument with an array of default values, such as `amps = #[1, 2]`.
    pub fn array(name: impl Into<String>, defaults: impl Into<Vec<f64>>) -> Self {
        Self {
            values: defaults.into(),
            array: true,
            ..Self::new(name, 0.0)
        }
    }

    /// The rate entry of the argument. The prefixes `i_`, `t_` and `a_` of
    /// its name count too, as in sclang.
    #[must_use]
    pub fn rate(mut self, rate: ArgRate) -> Self {
        self.rate = Some(rate);
        self
    }

    /// The lag of each value of a control-rate argument.
    #[must_use]
    pub fn lag(self, lag: f64) -> Self {
        self.lags(vec![lag])
    }

    /// One lag per value of a control-rate argument. If the lags are fewer
    /// than the values, they repeat from the start, as `wrapExtend` repeats
    /// them.
    #[must_use]
    pub fn lags(mut self, lags: impl Into<Vec<f64>>) -> Self {
        self.lags = lags.into();
        self
    }

    fn group(&self) -> Group {
        let prefix = |prefix: &[u8]| self.name.as_bytes().starts_with(prefix);
        if self.rate == Some(ArgRate::Ir) || prefix(b"i_") {
            Group::Scalar
        } else if self.rate == Some(ArgRate::Tr) || prefix(b"t_") {
            Group::Trigger
        } else if self.rate == Some(ArgRate::Ar) || prefix(b"a_") {
            Group::Audio
        } else {
            Group::Control
        }
    }

    fn lag_values(&self) -> Vec<f64> {
        if self.lags.is_empty() {
            return vec![0.0; self.values.len()];
        }
        (0..self.values.len())
            .map(|i| self.lags[i % self.lags.len()])
            .collect()
    }
}

/// The groups of `buildControls`, in the order in which it builds them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Group {
    Scalar,
    Trigger,
    Audio,
    Control,
}

/// A UGen to add with [`SynthDefBuilder::add`], at the level of `UGen:new1`
/// after `init`: its wire inputs, its outputs and its special index.
#[derive(Clone, Debug, PartialEq)]
pub struct NewUGen {
    name: String,
    rate: Rate,
    inputs: Vec<UGenInput>,
    outputs: Vec<Rate>,
    special_index: i16,
    width_first: bool,
}

impl NewUGen {
    /// A UGen of the class `name` at `rate`, with no input, one output at
    /// `rate` and the special index 0.
    pub fn new(name: impl Into<String>, rate: Rate) -> Self {
        Self {
            name: name.into(),
            rate,
            inputs: Vec::new(),
            outputs: vec![rate],
            special_index: 0,
            width_first: false,
        }
    }

    /// The inputs, in the order of the wire.
    #[must_use]
    pub fn inputs(mut self, inputs: impl Into<Vec<UGenInput>>) -> Self {
        self.inputs = inputs.into();
        self
    }

    /// The rate of each output. An `Out` has none.
    #[must_use]
    pub fn outputs(mut self, outputs: impl Into<Vec<Rate>>) -> Self {
        self.outputs = outputs.into();
        self
    }

    /// The special index.
    #[must_use]
    pub fn special_index(mut self, special_index: i16) -> Self {
        self.special_index = special_index;
        self
    }

    /// Whether the class is a `WidthFirstUGen`. Every UGen added after it
    /// runs after it.
    #[must_use]
    pub fn width_first(mut self, width_first: bool) -> Self {
        self.width_first = width_first;
        self
    }
}

/// A synth definition during its build: the `SynthDef` of sclang between
/// `initBuild` and `finishBuild`.
#[derive(Clone, Debug)]
pub struct SynthDefBuilder {
    build: u64,
    name: String,
    controls: Vec<f64>,
    control_names: Vec<ControlName>,
    sizes: Vec<usize>,
    arguments: Vec<UGenInput>,
    children: Vec<Node>,
    width_first: Vec<usize>,
    variants: Vec<Variant>,
}

#[derive(Clone, Debug)]
struct Node {
    name: String,
    rate: Rate,
    inputs: Vec<Source>,
    outputs: Vec<Rate>,
    special_index: i16,
    width_first_before: usize,
}

#[derive(Clone, Copy, Debug)]
enum Source {
    Number(f64),
    Output { ugen: usize, output: u32 },
}

impl SynthDefBuilder {
    /// A build with the arguments of the graph function. It builds their
    /// control UGens at once, as `buildUgenGraph` does before it calls the
    /// function.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for two arguments of one name, for an empty array,
    /// for a lag on a scalar, trigger or audio argument, and for a NaN lag.
    pub fn new(name: impl Into<String>, args: Vec<Arg>) -> Result<Self, BuildError> {
        for (position, arg) in args.iter().enumerate() {
            if args[..position].iter().any(|other| other.name == arg.name) {
                return Err(BuildError::DuplicateArgument(arg.name.clone()));
            }
            if arg.values.is_empty() {
                return Err(BuildError::EmptyArray(arg.name.clone()));
            }
            if arg.lags.iter().any(|lag| lag.is_nan()) {
                return Err(BuildError::NanLag(arg.name.clone()));
            }
            if arg.group() != Group::Control && arg.lags.iter().any(|&lag| lag != 0.0) {
                return Err(BuildError::LagOnNonControl(arg.name.clone()));
            }
        }
        let mut builder = Self {
            build: NEXT_BUILD.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            controls: Vec::new(),
            control_names: Vec::new(),
            sizes: Vec::new(),
            arguments: Vec::new(),
            children: Vec::new(),
            width_first: Vec::new(),
            variants: Vec::new(),
        };
        let mut arguments = vec![UGenInput::Array(Vec::new()); args.len()];
        let mut indices = vec![0; args.len()];
        for group in [Group::Scalar, Group::Trigger, Group::Audio, Group::Control] {
            let members: Vec<usize> = (0..args.len())
                .filter(|&position| args[position].group() == group)
                .collect();
            if members.is_empty() {
                continue;
            }
            let mut index = builder.controls.len();
            let mut values = Vec::new();
            let mut lags = Vec::new();
            for &position in &members {
                indices[position] = index;
                index += args[position].values.len();
                values.extend_from_slice(&args[position].values);
                lags.extend(args[position].lag_values());
            }
            let mut outputs = builder.add_controls(group, &values, &lags)?.into_iter();
            for &position in &members {
                let arg = &args[position];
                let proxies: Vec<UGenInput> = outputs
                    .by_ref()
                    .take(arg.values.len())
                    .map(UGenInput::Output)
                    .collect();
                arguments[position] = if arg.array {
                    UGenInput::Array(proxies)
                } else {
                    proxies
                        .into_iter()
                        .next()
                        .unwrap_or(UGenInput::Array(Vec::new()))
                };
            }
        }
        for (position, arg) in args.into_iter().enumerate() {
            builder.sizes.push(arg.values.len());
            builder.control_names.push(ControlName {
                name: arg.name,
                index: to_u32(indices[position]),
            });
        }
        builder.arguments = arguments;
        Ok(builder)
    }

    /// One input per argument, in the order of the arguments: an output, or
    /// an array of outputs for an array.
    #[must_use]
    pub fn arguments(&self) -> &[UGenInput] {
        &self.arguments
    }

    /// The input of the argument `name`.
    #[must_use]
    pub fn argument(&self, name: &str) -> Option<&UGenInput> {
        let position = self
            .control_names
            .iter()
            .position(|control| control.name == name)?;
        self.arguments.get(position)
    }

    /// Adds a UGen, as `UGen:new1` does. It gives back one output, or an
    /// array of the outputs when the UGen has none or several.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an input that is NaN, an array, or an output of
    /// another build.
    pub fn add(&mut self, ugen: NewUGen) -> Result<UGenInput, BuildError> {
        let mut inputs = Vec::with_capacity(ugen.inputs.len());
        for (input, value) in ugen.inputs.iter().enumerate() {
            inputs.push(self.source(&ugen.name, input, value)?);
        }
        let index = self.children.len();
        self.children.push(Node {
            name: ugen.name,
            rate: ugen.rate,
            inputs,
            outputs: ugen.outputs,
            special_index: ugen.special_index,
            width_first_before: self.width_first.len(),
        });
        if ugen.width_first {
            self.width_first.push(index);
        }
        Ok(self.outputs_of(index))
    }

    /// Adds a `UnaryOpUGen`: the operator `op` on `a`, at the rate of `a`. A
    /// number is at scalar rate. Unlike sclang, it does not compute the
    /// operator of a number.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an operator that `UnaryOpUGen` does not know, and
    /// for an input that [`add`](Self::add) refuses.
    pub fn unary_op(&mut self, op: &str, a: impl Into<UGenInput>) -> Result<UGenInput, BuildError> {
        const NAME: &str = "UnaryOpUGen";
        let a = a.into();
        let special_index =
            operator::unary_index(op).ok_or_else(|| BuildError::UnknownOperator(op.to_owned()))?;
        let rate = rate_of(NAME, 0, &a)?;
        self.add(
            NewUGen::new(NAME, rate)
                .inputs([a])
                .special_index(special_index),
        )
    }

    /// Adds a `BinaryOpUGen`: `a op b`, at the highest rate of its inputs in
    /// the order demand, audio, control, scalar. A number is at scalar rate.
    /// Unlike sclang, it does not fold `* 1`, `+ 0` and the like, and it does
    /// not compute the operator of two numbers.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an operator that `BinaryOpUGen` does not know, and
    /// for an input that [`add`](Self::add) refuses.
    pub fn binary_op(
        &mut self,
        op: &str,
        a: impl Into<UGenInput>,
        b: impl Into<UGenInput>,
    ) -> Result<UGenInput, BuildError> {
        const NAME: &str = "BinaryOpUGen";
        let (a, b) = (a.into(), b.into());
        let special_index =
            operator::binary_index(op).ok_or_else(|| BuildError::UnknownOperator(op.to_owned()))?;
        let rate = rate_of(NAME, 0, &a)?.max(rate_of(NAME, 1, &b)?);
        self.add(
            NewUGen::new(NAME, rate)
                .inputs([a, b])
                .special_index(special_index),
        )
    }

    /// Adds a UGen by its class name and the selector of a rate method, with
    /// the arguments by name, as `SinOsc.ar(freq: 440)` adds one in sclang.
    /// The catalogue gives the order of the arguments, their defaults, the
    /// wiring and the rule of `checkInputs`. It gives back one output, or an
    /// array of the outputs when the UGen has none or several.
    ///
    /// A single input is an array of one where the method takes an array, as
    /// `asArray` makes it.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for a class, a method or an argument that the
    /// catalogue does not have, for a method that it marks as not addable,
    /// for an argument without a value or with a value of another kind, for
    /// `mul` other than 1 and `add` other than 0, for an array where the
    /// method takes one input, for a bad count of channels, for inputs that
    /// break the rule of `checkInputs`, and for an input that
    /// [`add`](Self::add) refuses.
    pub fn ugen(
        &mut self,
        class: &str,
        selector: &str,
        args: &[(&str, ArgValue)],
    ) -> Result<UGenInput, BuildError> {
        let (ugen_class, method, wiring) = addable_method(class, selector)?;
        let values = bind_arguments(class, method, args)?;
        check_madd(method, wiring, &values)?;
        let inputs = wire_inputs(class, method, wiring, &values)?;
        let outputs = wire_outputs(class, method, wiring, &values)?;
        check_inputs::check(ugen_class.check_inputs, wiring.rate, &inputs).map_err(|message| {
            BuildError::CheckInputs {
                class: class.to_owned(),
                message,
            }
        })?;
        self.add(
            NewUGen::new(class, wiring.rate)
                .inputs(inputs)
                .outputs(outputs)
                .special_index(wiring.special_index)
                .width_first(ugen_class.is_width_first),
        )
    }

    /// Adds a variant: the defaults of the parameters, with the values of
    /// `values` by control name, as the `variants` of `SynthDef.new` give
    /// them. Its name is `<definition>.<name>`.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for a name that a variant has already, for a control
    /// that the definition does not have, and for more values than the
    /// control has.
    pub fn variant(&mut self, name: &str, values: &[(&str, Vec<f64>)]) -> Result<(), BuildError> {
        let full_name = format!("{}.{name}", self.name);
        if self
            .variants
            .iter()
            .any(|variant| variant.name == full_name)
        {
            return Err(BuildError::DuplicateVariant(full_name));
        }
        let mut controls = self.controls.clone();
        for (control, control_values) in values {
            let Some(position) = self
                .control_names
                .iter()
                .position(|control_name| control_name.name == *control)
            else {
                return Err(BuildError::UnknownControl {
                    variant: full_name,
                    control: (*control).to_owned(),
                });
            };
            if control_values.len() > self.sizes[position] {
                return Err(BuildError::VariantSize {
                    variant: full_name,
                    control: (*control).to_owned(),
                    values: control_values.len(),
                    size: self.sizes[position],
                });
            }
            let start = to_usize(self.control_names[position].index);
            controls[start..start + control_values.len()].copy_from_slice(control_values);
        }
        self.variants.push(Variant {
            name: full_name,
            values: controls.into_iter().map(to_float32).collect(),
        });
        Ok(())
    }

    /// Ends the build, as `finishBuild` does without `optimizeGraph`: it
    /// collects the constants, sorts the UGens and indexes them.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] when the definition breaks a limit of scsynth.
    pub fn finish(self) -> Result<SynthDef, BuildError> {
        let mut constants = Vec::new();
        let mut constant_index = BTreeMap::new();
        for node in &self.children {
            for source in &node.inputs {
                if let Source::Number(number) = *source {
                    constant_index.entry(number.to_bits()).or_insert_with(|| {
                        constants.push(to_float32(number));
                        to_u32(constants.len() - 1)
                    });
                }
            }
        }
        let antecedents: Vec<Vec<usize>> = self
            .children
            .iter()
            .map(|node| {
                node.inputs
                    .iter()
                    .filter_map(|source| match *source {
                        Source::Output { ugen, .. } => Some(ugen),
                        Source::Number(_) => None,
                    })
                    .chain(self.width_first[..node.width_first_before].iter().copied())
                    .collect()
            })
            .collect();
        let order = topo::sort(&antecedents);
        let mut synth_index = vec![0; order.len()];
        for (position, &ugen) in order.iter().enumerate() {
            synth_index[ugen] = to_u32(position);
        }
        let children = order
            .iter()
            .map(|&ugen| {
                let node = &self.children[ugen];
                UGen {
                    name: node.name.clone(),
                    rate: node.rate,
                    inputs: node
                        .inputs
                        .iter()
                        .map(|source| match *source {
                            Source::Number(number) => {
                                InputSpec::Constant(constant_index[&number.to_bits()])
                            }
                            Source::Output { ugen, output } => InputSpec::UGen {
                                index: synth_index[ugen],
                                output,
                            },
                        })
                        .collect(),
                    outputs: node.outputs.clone(),
                    special_index: node.special_index,
                }
            })
            .collect();
        let def = SynthDef {
            name: self.name,
            constants,
            controls: self.controls.into_iter().map(to_float32).collect(),
            control_names: self.control_names,
            children,
            variants: self.variants,
        };
        limits::check(&def, Version::V2).map_err(BuildError::Limit)?;
        Ok(def)
    }

    fn add_controls(
        &mut self,
        group: Group,
        values: &[f64],
        lags: &[f64],
    ) -> Result<Vec<OutputProxy>, BuildError> {
        match group {
            Group::Scalar => self.add_control("Control", Rate::Scalar, values, &[]),
            Group::Trigger => self.add_control("TrigControl", Rate::Control, values, &[]),
            Group::Audio => self.add_control("AudioControl", Rate::Audio, values, &[]),
            Group::Control if lags.iter().any(|&lag| lag != 0.0) => {
                let mut outputs = Vec::with_capacity(values.len());
                for (values, lags) in values.chunks(16).zip(lags.chunks(16)) {
                    outputs.extend(self.add_control("LagControl", Rate::Control, values, lags)?);
                }
                Ok(outputs)
            }
            Group::Control => self.add_control("Control", Rate::Control, values, &[]),
        }
    }

    fn add_control(
        &mut self,
        name: &str,
        rate: Rate,
        values: &[f64],
        lags: &[f64],
    ) -> Result<Vec<OutputProxy>, BuildError> {
        let special_index = i16::try_from(self.controls.len())
            .map_err(|_| BuildError::TooManyParameters(self.controls.len()))?;
        self.controls.extend_from_slice(values);
        let index = self.children.len();
        self.children.push(Node {
            name: name.to_owned(),
            rate,
            inputs: lags.iter().map(|&lag| Source::Number(lag)).collect(),
            outputs: vec![rate; values.len()],
            special_index,
            width_first_before: self.width_first.len(),
        });
        Ok((0..values.len())
            .map(|output| OutputProxy {
                build: self.build,
                ugen: index,
                output: to_u32(output),
                rate,
            })
            .collect())
    }

    fn source(&self, ugen: &str, input: usize, value: &UGenInput) -> Result<Source, BuildError> {
        match value {
            UGenInput::Number(number) if number.is_nan() => Err(BuildError::NotANumber {
                ugen: ugen.to_owned(),
                input,
            }),
            UGenInput::Number(number) => Ok(Source::Number(*number)),
            UGenInput::Output(output) if output.build != self.build => {
                Err(BuildError::ForeignOutput {
                    ugen: ugen.to_owned(),
                    input,
                })
            }
            UGenInput::Output(output) => Ok(Source::Output {
                ugen: output.ugen,
                output: output.output,
            }),
            UGenInput::Array(_) => Err(BuildError::MultichannelExpansion {
                ugen: ugen.to_owned(),
                input,
            }),
        }
    }

    fn outputs_of(&self, index: usize) -> UGenInput {
        let mut outputs: Vec<UGenInput> = self.children[index]
            .outputs
            .iter()
            .enumerate()
            .map(|(output, &rate)| {
                UGenInput::Output(OutputProxy {
                    build: self.build,
                    ugen: index,
                    output: to_u32(output),
                    rate,
                })
            })
            .collect();
        if outputs.len() == 1 {
            outputs.remove(0)
        } else {
            UGenInput::Array(outputs)
        }
    }
}

fn rate_of(ugen: &str, input: usize, value: &UGenInput) -> Result<Rate, BuildError> {
    value
        .rate()
        .ok_or_else(|| BuildError::MultichannelExpansion {
            ugen: ugen.to_owned(),
            input,
        })
}

/// sclang keeps its Floats in 64 bits and writes them with `putFloat`, which
/// rounds them to float32.
#[allow(clippy::cast_possible_truncation)]
fn to_float32(value: f64) -> f32 {
    value as f32
}

/// The class, the method and its wiring, if `ugen` can add the method.
fn addable_method(
    class: &str,
    selector: &str,
) -> Result<(&'static UGenClass, &'static RateMethod, Wiring), BuildError> {
    let ugen_class =
        catalog::lookup(class).ok_or_else(|| BuildError::UnknownClass(class.to_owned()))?;
    let method = ugen_class
        .method(selector)
        .ok_or_else(|| BuildError::UnknownMethod {
            class: class.to_owned(),
            selector: selector.to_owned(),
        })?;
    let wiring = match method.addable {
        Addable::Yes(wiring) => wiring,
        Addable::No(reason) => {
            return Err(BuildError::NotAddable {
                class: class.to_owned(),
                selector: selector.to_owned(),
                reason: reason.to_owned(),
            });
        }
    };
    Ok((ugen_class, method, wiring))
}

/// The value of each argument of `method`: the one that `args` gives by
/// name, or the default.
fn bind_arguments(
    class: &str,
    method: &RateMethod,
    args: &[(&str, ArgValue)],
) -> Result<Vec<ArgValue>, BuildError> {
    let mut values: Vec<Option<ArgValue>> = vec![None; method.args.len()];
    for (name, value) in args {
        let position = method
            .args
            .iter()
            .position(|arg| arg.name == *name)
            .ok_or_else(|| BuildError::UnknownArgument {
                class: class.to_owned(),
                argument: (*name).to_owned(),
            })?;
        if values[position].is_some() {
            return Err(BuildError::ArgumentTwice((*name).to_owned()));
        }
        values[position] = Some(value.clone());
    }
    method
        .args
        .iter()
        .zip(values)
        .map(|(arg, value)| match (value, arg.default) {
            (Some(value), _) => Ok(value),
            (None, Some(ArgDefault::Float(number))) => Ok(ArgValue::from(number)),
            (None, Some(ArgDefault::Symbol(text))) if arg.kind == ArgKind::Symbol => {
                Ok(ArgValue::from(text))
            }
            _ => Err(BuildError::MissingArgument {
                class: class.to_owned(),
                argument: arg.name.to_owned(),
            }),
        })
        .collect()
}

/// `mul` and `add` keep their defaults, 1 and 0, since the builder does not
/// apply `madd`.
fn check_madd(method: &RateMethod, wiring: Wiring, values: &[ArgValue]) -> Result<(), BuildError> {
    if !wiring.madd {
        return Ok(());
    }
    for (arg, value) in method.args.iter().zip(values) {
        let default = match arg.name {
            "mul" => 1.0,
            "add" => 0.0,
            _ => continue,
        };
        if *value != ArgValue::from(default) {
            return Err(BuildError::MaddNotSupported(arg.name.to_owned()));
        }
    }
    Ok(())
}

fn wrong_kind(
    class: &str,
    method: &RateMethod,
    position: usize,
    expected: &'static str,
) -> BuildError {
    BuildError::WrongKind {
        class: class.to_owned(),
        argument: method.args[position].name.to_owned(),
        expected,
    }
}

/// The inputs of the wire, slot by slot.
fn wire_inputs(
    class: &str,
    method: &RateMethod,
    wiring: Wiring,
    values: &[ArgValue],
) -> Result<Vec<UGenInput>, BuildError> {
    let mut inputs = Vec::new();
    for &slot in wiring.inputs {
        match slot {
            InputSlot::Arg(position) => match &values[position] {
                ArgValue::Input(UGenInput::Array(_)) => {
                    return Err(BuildError::MultichannelExpansion {
                        ugen: class.to_owned(),
                        input: inputs.len(),
                    });
                }
                ArgValue::Input(input) => inputs.push(input.clone()),
                ArgValue::Text(_) => {
                    return Err(wrong_kind(class, method, position, "a number or an output"));
                }
            },
            InputSlot::ArgArray(position) => match &values[position] {
                ArgValue::Input(UGenInput::Array(items)) => inputs.extend(items.iter().cloned()),
                ArgValue::Input(input) => inputs.push(input.clone()),
                ArgValue::Text(_) => return Err(wrong_kind(class, method, position, "an array")),
            },
            InputSlot::ArgString(position) => match &values[position] {
                ArgValue::Text(text) => {
                    inputs.push(UGenInput::from(f64::from(to_u32(text.len()))));
                    inputs.extend(text.bytes().map(|byte| UGenInput::from(f64::from(byte))));
                }
                ArgValue::Input(_) => return Err(wrong_kind(class, method, position, "a symbol")),
            },
        }
    }
    Ok(inputs)
}

/// The rate of each output.
fn wire_outputs(
    class: &str,
    method: &RateMethod,
    wiring: Wiring,
    values: &[ArgValue],
) -> Result<Vec<Rate>, BuildError> {
    let bad_count = |position: usize| BuildError::BadCount {
        class: class.to_owned(),
        argument: method.args[position].name.to_owned(),
    };
    match wiring.outputs {
        Outputs::Fixed(rates) => Ok(rates.to_vec()),
        Outputs::Arg { arg, rate } => match values[arg] {
            ArgValue::Input(UGenInput::Number(count))
                if count >= 1.0 && count.fract() == 0.0 && count <= f64::from(u16::MAX) =>
            {
                Ok(vec![rate; count_to_usize(count)])
            }
            _ => Err(bad_count(arg)),
        },
        Outputs::ArraySize { arg, rate } => match &values[arg] {
            ArgValue::Input(UGenInput::Array(items)) if !items.is_empty() => {
                Ok(vec![rate; items.len()])
            }
            ArgValue::Input(UGenInput::Array(_)) | ArgValue::Text(_) => Err(bad_count(arg)),
            ArgValue::Input(_) => Ok(vec![rate]),
        },
    }
}

/// A whole number from 1 to `u16::MAX`, as a count of outputs.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn count_to_usize(count: f64) -> usize {
    count as usize
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::{Arg, ArgRate, NewUGen, SynthDefBuilder};
    use crate::error::BuildError;
    use crate::rate::Rate;
    use crate::synth_def::{ControlName, InputSpec};
    use crate::ugen_input::{ArgValue, UGenInput};

    fn out(builder: &mut SynthDefBuilder, rate: Rate, inputs: Vec<UGenInput>) {
        builder
            .add(NewUGen::new("Out", rate).inputs(inputs).outputs([]))
            .unwrap();
    }

    #[test]
    fn the_controls_follow_the_groups_of_sclang() {
        // { |freq = 440, a_in = 0, i_n = 1, t_g = 0, amps = #[1, 2]| }, fact 20.
        let builder = SynthDefBuilder::new(
            "groups",
            vec![
                Arg::new("freq", 440.0),
                Arg::new("a_in", 0.0),
                Arg::new("i_n", 1.0),
                Arg::new("t_g", 0.0),
                Arg::array("amps", [1.0, 2.0]),
            ],
        )
        .unwrap();
        let def = builder.finish().unwrap();
        let names: Vec<(&str, &str, i16, usize)> = def
            .children
            .iter()
            .map(|ugen| {
                (
                    ugen.name.as_str(),
                    match ugen.rate {
                        Rate::Scalar => "ir",
                        Rate::Control => "kr",
                        Rate::Audio => "ar",
                        Rate::Demand => "dr",
                    },
                    ugen.special_index,
                    ugen.outputs.len(),
                )
            })
            .collect();
        assert_eq!(
            names,
            [
                ("Control", "ir", 0, 1),
                ("TrigControl", "kr", 1, 1),
                ("AudioControl", "ar", 2, 1),
                ("Control", "kr", 3, 3),
            ]
        );
        assert_eq!(def.controls, [1.0, 0.0, 0.0, 440.0, 1.0, 2.0]);
        let indices: Vec<(&str, u32)> = def
            .control_names
            .iter()
            .map(|ControlName { name, index }| (name.as_str(), *index))
            .collect();
        assert_eq!(
            indices,
            [
                ("freq", 3),
                ("a_in", 2),
                ("i_n", 0),
                ("t_g", 1),
                ("amps", 4)
            ]
        );
    }

    #[test]
    fn the_prefix_wins_over_the_entry_kr() {
        let builder =
            SynthDefBuilder::new("prefix", vec![Arg::new("i_k", 3.0).rate(ArgRate::Kr)]).unwrap();
        assert_eq!(builder.arguments()[0].rate(), Some(Rate::Scalar));
    }

    #[test]
    fn the_entry_ir_wins_over_the_prefix_t() {
        let builder =
            SynthDefBuilder::new("entry", vec![Arg::new("t_x", 3.0).rate(ArgRate::Ir)]).unwrap();
        assert_eq!(builder.arguments()[0].rate(), Some(Rate::Scalar));
    }

    #[test]
    fn a_lag_makes_lag_controls_of_16_values() {
        let builder = SynthDefBuilder::new(
            "lags",
            vec![Arg::array("v", vec![0.0; 17]).lags([0.1, 0.2])],
        )
        .unwrap();
        let UGenInput::Array(outputs) = &builder.arguments()[0] else {
            panic!("an array argument gives an array");
        };
        assert_eq!(outputs.len(), 17);
        let def = builder.finish().unwrap();
        assert_eq!(def.children.len(), 2);
        assert_eq!(def.children[0].name, "LagControl");
        assert_eq!(def.children[0].inputs.len(), 16);
        assert_eq!(def.children[1].special_index, 16);
        assert_eq!(def.children[1].inputs, [InputSpec::Constant(0)]);
        assert_eq!(def.constants, [0.1, 0.2]);
    }

    #[test]
    fn the_arguments_are_refused_as_a_function_of_sclang_refuses_them() {
        let refuse = |args| SynthDefBuilder::new("refused", args).unwrap_err();
        assert_eq!(
            refuse(vec![Arg::new("a", 0.0), Arg::new("a", 1.0)]),
            BuildError::DuplicateArgument("a".to_owned())
        );
        assert_eq!(
            refuse(vec![Arg::array("a", [])]),
            BuildError::EmptyArray("a".to_owned())
        );
        assert_eq!(
            refuse(vec![Arg::new("i_a", 0.0).lag(0.1)]),
            BuildError::LagOnNonControl("i_a".to_owned())
        );
        assert_eq!(
            refuse(vec![Arg::new("a", 0.0).lag(f64::NAN)]),
            BuildError::NanLag("a".to_owned())
        );
    }

    #[test]
    fn a_constant_is_its_f64_bits() {
        let mut builder = SynthDefBuilder::new("bits", vec![]).unwrap();
        let inputs = [0.0, -0.0, 1.0, 1.0 + 1e-12, 1.0, f64::INFINITY]
            .map(UGenInput::from)
            .to_vec();
        out(&mut builder, Rate::Control, inputs);
        let def = builder.finish().unwrap();
        let bits: Vec<u32> = def.constants.iter().map(|c| c.to_bits()).collect();
        let expected: Vec<u32> = [0.0_f32, -0.0, 1.0, 1.0, f32::INFINITY]
            .iter()
            .map(|c| c.to_bits())
            .collect();
        assert_eq!(bits, expected);
        let indices: Vec<InputSpec> = (0..5).map(InputSpec::Constant).collect();
        assert_eq!(
            def.children[0].inputs,
            [
                indices[0], indices[1], indices[2], indices[3], indices[2], indices[4]
            ]
        );
    }

    #[test]
    fn an_input_must_be_a_number_or_an_output_of_this_build() {
        let mut builder = SynthDefBuilder::new("inputs", vec![]).unwrap();
        let mut other = SynthDefBuilder::new("other", vec![]).unwrap();
        let foreign = other.add(NewUGen::new("DC", Rate::Control)).unwrap();
        let add = |builder: &mut SynthDefBuilder, input: UGenInput| {
            builder
                .add(NewUGen::new("DC", Rate::Control).inputs([input]))
                .unwrap_err()
        };
        assert_eq!(
            add(&mut builder, f64::NAN.into()),
            BuildError::NotANumber {
                ugen: "DC".to_owned(),
                input: 0
            }
        );
        assert_eq!(
            add(&mut builder, vec![1.0.into(), 2.0.into()].into()),
            BuildError::MultichannelExpansion {
                ugen: "DC".to_owned(),
                input: 0
            }
        );
        assert_eq!(
            add(&mut builder, foreign),
            BuildError::ForeignOutput {
                ugen: "DC".to_owned(),
                input: 0
            }
        );
    }

    #[test]
    fn a_ugen_gives_an_output_or_an_array_of_outputs() {
        let mut builder = SynthDefBuilder::new("outputs", vec![]).unwrap();
        let one = builder.add(NewUGen::new("DC", Rate::Audio)).unwrap();
        assert!(matches!(one, UGenInput::Output(output) if output.output_index() == 0));
        let two = builder
            .add(NewUGen::new("Pan2", Rate::Audio).outputs([Rate::Audio, Rate::Audio]))
            .unwrap();
        assert!(matches!(&two, UGenInput::Array(outputs) if outputs.len() == 2));
        let none = builder
            .add(NewUGen::new("Out", Rate::Audio).outputs([]))
            .unwrap();
        assert_eq!(none, UGenInput::Array(Vec::new()));
    }

    #[test]
    fn a_variant_starts_from_the_defaults() {
        let mut builder = SynthDefBuilder::new(
            "v",
            vec![Arg::new("freq", 440.0), Arg::array("amps", [1.0, 2.0, 3.0])],
        )
        .unwrap();
        builder
            .variant("low", &[("freq", vec![220.0]), ("amps", vec![0.5])])
            .unwrap();
        assert_eq!(
            builder.variant("low", &[]),
            Err(BuildError::DuplicateVariant("v.low".to_owned()))
        );
        assert_eq!(
            builder.variant("x", &[("gain", vec![1.0])]),
            Err(BuildError::UnknownControl {
                variant: "v.x".to_owned(),
                control: "gain".to_owned()
            })
        );
        assert_eq!(
            builder.variant("y", &[("freq", vec![1.0, 2.0])]),
            Err(BuildError::VariantSize {
                variant: "v.y".to_owned(),
                control: "freq".to_owned(),
                values: 2,
                size: 1
            })
        );
        let def = builder.finish().unwrap();
        assert_eq!(def.variants.len(), 1);
        assert_eq!(def.variants[0].name, "v.low");
        assert_eq!(def.variants[0].values, [220.0, 0.5, 2.0, 3.0]);
    }

    #[test]
    fn an_operator_runs_at_the_highest_rate_of_its_inputs() {
        let mut builder = SynthDefBuilder::new("ops", vec![Arg::new("k", 0.0)]).unwrap();
        let control = builder.arguments()[0].clone();
        let audio = builder.add(NewUGen::new("SinOsc", Rate::Audio)).unwrap();
        let demand = builder.add(NewUGen::new("Dseq", Rate::Demand)).unwrap();
        let mut rate = |a: UGenInput, b: UGenInput| builder.binary_op("+", a, b).unwrap().rate();
        assert_eq!(rate(1.0.into(), 2.0.into()), Some(Rate::Scalar));
        assert_eq!(rate(1.0.into(), control.clone()), Some(Rate::Control));
        assert_eq!(rate(control.clone(), audio.clone()), Some(Rate::Audio));
        assert_eq!(rate(audio.clone(), demand.clone()), Some(Rate::Demand));
        assert_eq!(rate(demand.clone(), control.clone()), Some(Rate::Demand));
        let negated = builder.unary_op("neg", demand).unwrap();
        assert_eq!(negated.rate(), Some(Rate::Demand));
        assert_eq!(
            builder.unary_op("neg", 1.0).unwrap().rate(),
            Some(Rate::Scalar)
        );
    }

    #[test]
    fn an_operator_has_its_special_index() {
        let mut builder = SynthDefBuilder::new("ops", vec![]).unwrap();
        let sum = builder.binary_op("*", 1.0, 2.0).unwrap();
        let midicps = builder.unary_op("midicps", sum).unwrap();
        builder
            .add(
                NewUGen::new("Out", Rate::Scalar)
                    .inputs([0.0.into(), midicps])
                    .outputs([]),
            )
            .unwrap();
        let def = builder.finish().unwrap();
        let indices: Vec<(&str, i16)> = def
            .children
            .iter()
            .map(|ugen| (ugen.name.as_str(), ugen.special_index))
            .collect();
        assert_eq!(
            indices,
            [("BinaryOpUGen", 2), ("UnaryOpUGen", 17), ("Out", 0)]
        );
    }

    #[test]
    fn an_operator_refuses_an_unknown_name_and_an_array() {
        let mut builder = SynthDefBuilder::new("ops", vec![]).unwrap();
        assert_eq!(
            builder.unary_op("+", 1.0),
            Err(BuildError::UnknownOperator("+".to_owned()))
        );
        assert_eq!(
            builder.binary_op("neg", 1.0, 2.0),
            Err(BuildError::UnknownOperator("neg".to_owned()))
        );
        let array = UGenInput::Array(vec![1.0.into(), 2.0.into()]);
        assert_eq!(
            builder.binary_op("+", 1.0, array),
            Err(BuildError::MultichannelExpansion {
                ugen: "BinaryOpUGen".to_owned(),
                input: 1
            })
        );
    }

    #[test]
    fn ugen_refuses_what_the_catalogue_cannot_wire() {
        let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
        let mut refuse = |class: &str, selector: &str, args: &[(&str, ArgValue)]| -> BuildError {
            builder.ugen(class, selector, args).unwrap_err()
        };
        assert_eq!(
            refuse("NoSuchUGen", "ar", &[]),
            BuildError::UnknownClass("NoSuchUGen".to_owned())
        );
        assert!(matches!(
            refuse("SinOsc", "ir", &[]),
            BuildError::UnknownMethod { .. }
        ));
        assert!(matches!(
            refuse("Lag", "kr", &[]),
            BuildError::NotAddable { .. }
        ));
        assert!(matches!(
            refuse("Duty", "kr", &[]),
            BuildError::NotAddable { .. }
        ));
        assert!(matches!(
            refuse("SinOsc", "ar", &[("pitch", 1.0.into())]),
            BuildError::UnknownArgument { .. }
        ));
        assert_eq!(
            refuse(
                "SinOsc",
                "ar",
                &[("freq", 1.0.into()), ("freq", 2.0.into())]
            ),
            BuildError::ArgumentTwice("freq".to_owned())
        );
        assert!(matches!(
            refuse("Out", "ar", &[]),
            BuildError::MissingArgument { .. }
        ));
        assert_eq!(
            refuse("SinOsc", "ar", &[("mul", 0.5.into())]),
            BuildError::MaddNotSupported("mul".to_owned())
        );
        assert!(matches!(
            refuse(
                "SinOsc",
                "ar",
                &[("freq", vec![1.0.into(), 2.0.into()].into())]
            ),
            BuildError::MultichannelExpansion { .. }
        ));
        assert!(matches!(
            refuse("SinOsc", "ar", &[("freq", "a".into())]),
            BuildError::WrongKind { .. }
        ));
        for count in [1.5, 0.0, -1.0] {
            assert!(matches!(
                refuse("In", "ar", &[("numChannels", count.into())]),
                BuildError::BadCount { .. }
            ));
        }
    }

    #[test]
    fn ugen_checks_the_inputs_as_sclang_does() {
        // Out.ar(0, 0.5), which sclang refuses, fact 34 of the plan.
        let mut builder = SynthDefBuilder::new("checked", vec![]).unwrap();
        let error = builder
            .ugen(
                "Out",
                "ar",
                &[("bus", 0.0.into()), ("channelsArray", 0.5.into())],
            )
            .unwrap_err();
        assert!(matches!(error, BuildError::CheckInputs { .. }));
        let sine = builder.ugen("SinOsc", "ar", &[]).unwrap();
        builder
            .ugen(
                "Out",
                "ar",
                &[("bus", 0.0.into()), ("channelsArray", sine.into())],
            )
            .unwrap();
    }

    #[test]
    fn finish_applies_the_limits() {
        let builder = SynthDefBuilder::new("n".repeat(256), vec![]).unwrap();
        assert!(matches!(builder.finish(), Err(BuildError::Limit(_))));
    }
}
