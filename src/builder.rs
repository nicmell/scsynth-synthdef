//! The build of a synth definition, as sclang builds a `SynthDef`.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::catalog::{
    self, Addable, ArgDefault, ArgKind, ArrayExpansion, ConvertEnv, InputSlot, Outputs,
    Passthrough, RateMethod, UGenClass, Wiring,
};
use crate::check_inputs;
use crate::env::Env;
use crate::error::BuildError;
use crate::file::Version;
use crate::limits;
use crate::operator;
use crate::optimize::{self, Rewritten};
use crate::pseudo::{self, NamedEntry, lang};
use crate::rate::{Rate, least_by_name};
use crate::synth_def::{ControlName, InputSpec, SynthDef, UGen, Variant};
use crate::topo;
use crate::ugen_input::{ArgValue, OutputProxy, UGenInput};

/// The serial of the next UGen of any build. A rollback takes none back.
static NEXT_SERIAL: AtomicU64 = AtomicU64::new(0);

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
/// `initBuild` and `finishBuild`. A call that fails leaves the build as it
/// was, and an output that the call made belongs to no build. A clone takes
/// the outputs from before it, and no output that the other build makes
/// after it.
#[derive(Clone, Debug)]
pub struct SynthDefBuilder {
    name: String,
    controls: Vec<f64>,
    control_names: Vec<ControlName>,
    /// The size of the default of each name, or none for a name of
    /// `Control.names` that has no default yet.
    sizes: Vec<Option<usize>>,
    /// The last name of the current function, which `Control.init` gives
    /// its values when it has no default.
    last_name: Option<usize>,
    /// The table of `NamedControl` for the build, in the order of the names.
    named: Vec<NamedEntry>,
    arguments: Vec<UGenInput>,
    children: Vec<Node>,
    /// The serial of each child, which its outputs carry.
    serials: Vec<u64>,
    width_first: Vec<usize>,
    variants: Vec<Variant>,
    max_local_bufs: Option<MaxLocalBufs>,
}

/// The `MaxLocalBufs` of a build: its node, and the number of `LocalBuf`s
/// that take it, which `finish` writes into its input.
#[derive(Clone, Copy, Debug)]
struct MaxLocalBufs {
    node: usize,
    count: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) rate: Rate,
    pub(crate) inputs: Vec<Source>,
    pub(crate) outputs: Vec<Rate>,
    pub(crate) special_index: i16,
    pub(crate) width_first_before: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Source {
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
        let mut builder = Self {
            name: name.into(),
            controls: Vec::new(),
            control_names: Vec::new(),
            sizes: Vec::new(),
            last_name: None,
            named: Vec::new(),
            arguments: Vec::new(),
            children: Vec::new(),
            serials: Vec::new(),
            width_first: Vec::new(),
            variants: Vec::new(),
            max_local_bufs: None,
        };
        builder.arguments = builder.add_arguments(args)?;
        Ok(builder)
    }

    /// Adds the controls of the arguments of a function, as
    /// `addControlsFromArgsOfFunc` and `buildControls` do: a control UGen
    /// for each group of rate, at the next parameters, and the names in the
    /// order of the arguments. It gives back one input per argument.
    fn add_arguments(&mut self, args: Vec<Arg>) -> Result<Vec<UGenInput>, BuildError> {
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
        let mut arguments = vec![UGenInput::Array(Vec::new()); args.len()];
        let mut indices = vec![0; args.len()];
        for group in [Group::Scalar, Group::Trigger, Group::Audio, Group::Control] {
            let members: Vec<usize> = (0..args.len())
                .filter(|&position| args[position].group() == group)
                .collect();
            if members.is_empty() {
                continue;
            }
            let mut index = self.controls.len();
            let mut values = Vec::new();
            let mut lags = Vec::new();
            for &position in &members {
                indices[position] = index;
                index += args[position].values.len();
                values.extend_from_slice(&args[position].values);
                lags.extend(args[position].lag_values());
            }
            let mut outputs = self.add_controls(group, &values, &lags)?.into_iter();
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
            self.sizes.push(Some(arg.values.len()));
            self.control_names.push(ControlName {
                name: arg.name,
                index: to_u32(indices[position]),
            });
            self.last_name = Some(self.control_names.len() - 1);
        }
        Ok(arguments)
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

    /// Names the next parameters, as `Control.names` does: the name at
    /// position `i` takes the index of the parameter `i` after those of the
    /// build so far. A name has no default until a `Control` or a
    /// `TrigControl` gives the last name its values.
    pub fn control_names(&mut self, names: &[&str]) {
        let index = self.controls.len();
        for (offset, name) in names.iter().enumerate() {
            self.control_names.push(ControlName {
                name: (*name).to_owned(),
                index: to_u32(index + offset),
            });
            self.sizes.push(None);
            self.last_name = Some(self.control_names.len() - 1);
        }
    }

    /// Adds the controls of the arguments of a function inside the graph
    /// function, and calls `body` with one input per argument, as
    /// `SynthDef.wrap` does. The controls take the next parameters, in the
    /// groups of rate of [`new`](Self::new). Around `body`, the last name is
    /// the last of `args`, as `controlNames` holds only the names of the
    /// function in sclang. For `prependArgs`, `body` takes their values from
    /// its caller, and `args` holds only the arguments that become controls.
    ///
    /// # Errors
    ///
    /// The errors of [`new`](Self::new) for `args`, and those of `body`. A
    /// call that fails leaves the build as it was.
    pub fn wrap<T>(
        &mut self,
        args: Vec<Arg>,
        body: impl FnOnce(&mut Self, &[UGenInput]) -> Result<T, BuildError>,
    ) -> Result<T, BuildError> {
        self.transaction(|builder| {
            let outer = builder.last_name.take();
            let inputs = builder.add_arguments(args)?;
            let result = body(builder, &inputs);
            builder.last_name = outer;
            result
        })
    }

    /// Makes an envelope by a class method of `Env` that makes one, such as
    /// `perc`, with the arguments by name, as sclang makes it. The catalogue
    /// gives the order of the arguments and their defaults. A constructor
    /// that computes with outputs, as `adsr` with a `sustainLevel` that is
    /// an output, adds the UGens of its math at once, as sclang does.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for a method that `Env` lacks, for an argument that
    /// the method does not take or that is of the wrong kind, and
    /// [`BuildError::BadEnv`] where sclang throws. A call that fails leaves
    /// the build as it was.
    pub fn env(&mut self, selector: &str, args: &[(&str, ArgValue)]) -> Result<Env, BuildError> {
        let method = catalog::env_method(selector).ok_or_else(|| BuildError::UnknownMethod {
            class: "Env".to_owned(),
            selector: selector.to_owned(),
        })?;
        let values = bind_optional("Env", method, args)?;
        self.transaction(|builder| pseudo::env::build(builder, method, values))
    }

    /// `env.range(lo, hi)`, `env.exprange(lo, hi)` and
    /// `env.curverange(lo, hi, curve)` of sclang: a copy of the envelope,
    /// with its levels mapped by `linlin`, `linexp` or `lincurve` from the
    /// least of them to the greatest, as [`send`](Self::send) maps an array.
    /// The catalogue gives the arguments and their defaults (see
    /// [`catalog::env_ranges`]).
    ///
    /// # Errors
    ///
    /// [`BuildError::BadMethod`] for a selector that the catalogue does not
    /// have, and [`BuildError::BadEnv`] for levels that `minItem` and
    /// `maxItem` cannot compare, such as an output among numbers, where
    /// sclang throws. Also the errors of the arguments, and of the UGens that
    /// the map adds. A call that fails leaves the build as it was.
    pub fn send_env(
        &mut self,
        env: &Env,
        selector: &str,
        args: &[(&str, ArgValue)],
    ) -> Result<Env, BuildError> {
        let method = catalog::env_range(selector).ok_or_else(|| BuildError::BadMethod {
            selector: selector.to_owned(),
            message: "it is no range of Env".to_owned(),
        })?;
        let values = bind_optional("Env", method, args)?;
        self.transaction(|builder| pseudo::env::range(builder, env, method, values))
    }

    /// `chain.pvcalc(numframes, func, frombin, tobin, zeroothers)` of
    /// sclang. It unpacks the bins of `chain` from `frombin` to `tobin`, as
    /// `UnpackFFT` does, and calls `func` with the array of their magnitudes
    /// and the array of their phases. Then it packs into `chain` what `func`
    /// gives, as `PackFFT` does. The catalogue gives the arguments and their
    /// defaults (see [`catalog::pv_method`]).
    ///
    /// As in sclang, the size of the result decides what `PackFFT` takes. An
    /// array of one value gets the phases after it, and an array of two is
    /// the magnitudes and the phases. Anything else, a UGen included, is the
    /// magnitudes, with the phases of `chain`.
    ///
    /// # Errors
    ///
    /// [`BuildError::WrongKind`] for a `chain` that is not one output of a
    /// `PV_ChainUGen`, as sclang understands the method on no other value,
    /// and for a value of `func` by name. Also the errors of the arguments,
    /// of `func`, and of the UGens that the method adds. A call that fails
    /// leaves the build as it was.
    pub fn pvcalc(
        &mut self,
        chain: impl Into<UGenInput>,
        args: &[(&str, ArgValue)],
        func: impl FnOnce(&mut Self, UGenInput, UGenInput) -> Result<UGenInput, BuildError>,
    ) -> Result<UGenInput, BuildError> {
        let (method, values) = pv_values("pvcalc", args)?;
        let chain = chain.into();
        self.transaction(|builder| pseudo::pvcalc(builder, chain, method, values, func))
    }

    /// `chain.pvcalc2(chain2, numframes, func, frombin, tobin, zeroothers)` of
    /// sclang: [`pvcalc`](Self::pvcalc) over the bins of `chain` and of
    /// `chain2`. `func` takes the magnitudes and the phases of `chain`, then
    /// those of `chain2`, and what it gives goes into `chain`.
    ///
    /// # Errors
    ///
    /// The errors of [`pvcalc`](Self::pvcalc).
    pub fn pvcalc2(
        &mut self,
        chain: impl Into<UGenInput>,
        args: &[(&str, ArgValue)],
        func: impl FnOnce(
            &mut Self,
            UGenInput,
            UGenInput,
            UGenInput,
            UGenInput,
        ) -> Result<UGenInput, BuildError>,
    ) -> Result<UGenInput, BuildError> {
        let (method, values) = pv_values("pvcalc2", args)?;
        let chain = chain.into();
        self.transaction(|builder| pseudo::pvcalc2(builder, chain, method, values, func))
    }

    /// `chain.pvcollect(numframes, func, frombin, tobin, zeroothers)` of
    /// sclang. It calls `func` for each bin of `chain` from `frombin` to
    /// `tobin`, with its magnitude, its phase, its bin and its index from 0,
    /// and packs what each call gives into `chain`. A result of one value
    /// gets the phase of its bin, and an array of more values stays as it is.
    ///
    /// # Errors
    ///
    /// The errors of [`pvcalc`](Self::pvcalc).
    pub fn pvcollect(
        &mut self,
        chain: impl Into<UGenInput>,
        args: &[(&str, ArgValue)],
        func: impl FnMut(&mut Self, UGenInput, UGenInput, f64, f64) -> Result<UGenInput, BuildError>,
    ) -> Result<UGenInput, BuildError> {
        let (method, values) = pv_values("pvcollect", args)?;
        let chain = chain.into();
        self.transaction(|builder| pseudo::pvcollect(builder, chain, method, values, func))
    }

    /// Sends the message `selector` to `receiver` with the arguments by name,
    /// as `receiver.range(200, 800)` sends `range` in sclang. The messages are
    /// the instance methods of `UGen` that build graphs (see
    /// [`catalog::ugen_methods`]), with the arguments and the defaults of
    /// `UGen`. The call passes all of them, in their order, to the method
    /// that the receiver answers, as sclang does when every argument comes by
    /// position.
    ///
    /// An output gets the method of the class of its UGen: that of `UGen`,
    /// or one of its own, as `DUGen` has for `linlin`. An output of a UGen
    /// with several outputs is an `OutputProxy`. An array gets the method of
    /// `Array`, which mostly hands the message to each element, with the
    /// arrays among the arguments spread over the elements. A number gets the
    /// method of `Float`, which computes a number where it can.
    ///
    /// # Errors
    ///
    /// [`BuildError::BadMethod`] for a selector that the catalogue does not
    /// have, and where sclang throws or writes a broken definition: a
    /// receiver that does not understand the message, a test of `if` on an
    /// output, or a rate that the method does not take. Also the errors of
    /// the arguments, and of the UGens that the method adds. A call that
    /// fails leaves the build as it was.
    pub fn send(
        &mut self,
        receiver: impl Into<UGenInput>,
        selector: &str,
        args: &[(&str, ArgValue)],
    ) -> Result<UGenInput, BuildError> {
        let method = catalog::ugen_method(selector).ok_or_else(|| BuildError::BadMethod {
            selector: selector.to_owned(),
            message: "it is no method of UGen that builds a graph".to_owned(),
        })?;
        let values = bind_optional("UGen", &method.method, args)?;
        let receiver = receiver.into();
        self.transaction(|builder| pseudo::send(builder, receiver, method, values))
    }

    /// `asArrayForInterpolation`: the envelope as `IEnvGen` reads it, an
    /// array per channel, one channel included. Its fourth value is
    /// `times.sum`, which adds UGens when a time is an output.
    ///
    /// # Errors
    ///
    /// [`BuildError::BadEnv`] for a shape that sclang does not know, and the
    /// errors of [`binary_op`](Self::binary_op).
    pub fn env_array_for_interpolation(&mut self, env: &Env) -> Result<UGenInput, BuildError> {
        self.transaction(|builder| pseudo::env::array_for_interpolation(builder, env))
    }

    /// Adds a UGen, as `UGen:new1` does. It gives back one output, or an
    /// array of the outputs when the UGen has none or several.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an input that is NaN, an array, or an output of
    /// another build or of a failed call.
    pub fn add(&mut self, ugen: NewUGen) -> Result<UGenInput, BuildError> {
        let mut inputs = Vec::with_capacity(ugen.inputs.len());
        for (input, value) in ugen.inputs.iter().enumerate() {
            inputs.push(self.source(&ugen.name, input, value)?);
        }
        let index = self.push(Node {
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

    /// Adds a `UnaryOpUGen` for the operator `op`, as `UnaryOpUGen.new` adds
    /// one: at the rate of its input, and for a number too. An array expands
    /// into one `UnaryOpUGen` for each element, as `multiNewList` expands it.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an operator that `UnaryOpUGen` does not know, for
    /// an empty array, and for an input that [`add`](Self::add) refuses.
    pub fn unary_op(&mut self, op: &str, a: impl Into<UGenInput>) -> Result<UGenInput, BuildError> {
        let special_index =
            operator::unary_index(op).ok_or_else(|| BuildError::UnknownOperator(op.to_owned()))?;
        let a = a.into();
        self.transaction(|builder| {
            builder.expand(UNARY_OP, &["a"], &[a], true, &mut |builder, values| {
                builder.unary_leaf(special_index, values[0].clone())
            })
        })
    }

    /// Adds a `BinaryOpUGen` for the operator `op`, as `BinaryOpUGen.new`
    /// adds one: `a op b`, at the highest rate of its inputs in the order
    /// demand, audio, control, scalar, and for two numbers too. Arrays expand
    /// as `multiNewList` expands them: a value spreads over an array, and the
    /// shorter of two arrays wraps.
    ///
    /// Before it adds a `BinaryOpUGen`, it folds as `BinaryOpUGen:new1` does.
    /// `* 0` on either side gives 0. `* 1` and `+ 0` on either side, `- 0`
    /// and `/ 1` give the other input. `* -1` on either side and `/ -1` give
    /// the `neg` of the other input, and `0 -` gives the `neg` of `b`. The
    /// `neg` of a number is a number.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an operator that `BinaryOpUGen` does not know, for
    /// an empty array, and for an input that [`add`](Self::add) refuses.
    pub fn binary_op(
        &mut self,
        op: &str,
        a: impl Into<UGenInput>,
        b: impl Into<UGenInput>,
    ) -> Result<UGenInput, BuildError> {
        let special_index =
            operator::binary_index(op).ok_or_else(|| BuildError::UnknownOperator(op.to_owned()))?;
        let (a, b) = (a.into(), b.into());
        self.transaction(|builder| {
            builder.expand(
                BINARY_OP,
                &["a", "b"],
                &[a, b],
                true,
                &mut |builder, values| {
                    builder.binary_leaf(op, special_index, values[0].clone(), values[1].clone())
                },
            )
        })
    }

    /// Adds a UGen by its class name and the selector of a rate method, with
    /// the arguments by name, as `SinOsc.ar(freq: 440)` adds one in sclang.
    /// The catalogue gives the order of the arguments, their defaults, the
    /// wiring and the rule of `checkInputs`. It gives back one output, or an
    /// array of the outputs when the UGen has none or several.
    ///
    /// For a method that the catalogue marks [`Addable::Pseudo`], such as
    /// `LinLin.kr`, a body of the crate adds the UGens that the method adds in
    /// sclang, in the same order, and gives back what the method gives: a
    /// number, an output, or an array of them.
    ///
    /// An array on an argument of the wire, or on the count of the outputs,
    /// expands into several UGens, as `multiNewList` expands it, and the
    /// result is the array of their results. A single input is an array of
    /// one where the method takes an array, as `asArray` makes it. If the
    /// method applies `madd`, [`madd`](Self::madd) then takes the whole
    /// result, with `mul` and `add`.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for a class, a method or an argument that the
    /// catalogue does not have, for a method that it marks as not addable,
    /// for an argument without a value or with a value of another kind, for
    /// `mul` other than 1 and `add` other than 0 on a UGen without outputs,
    /// for an empty array where an argument expands, for an array inside an
    /// array where it does not expand, for a bad count of channels, for
    /// inputs that break the rule of `checkInputs`, and for an input that
    /// [`add`](Self::add) refuses. A pseudo-UGen fails as the UGens that it
    /// adds fail.
    pub fn ugen(
        &mut self,
        class: &str,
        selector: &str,
        args: &[(&str, ArgValue)],
    ) -> Result<UGenInput, BuildError> {
        let (ugen_class, method) = match find_method(class, selector)? {
            Found::UGen(ugen_class, method) => (ugen_class, method),
            Found::Pseudo(name, method) => return self.pseudo(name, method, args),
        };
        let wiring = match method.addable {
            Addable::Yes(wiring) => wiring,
            Addable::Pseudo => return self.pseudo(ugen_class.name, method, args),
            Addable::No(reason) => {
                return Err(BuildError::NotAddable {
                    class: class.to_owned(),
                    selector: selector.to_owned(),
                    reason: reason.to_owned(),
                });
            }
        };
        let mut values = bind_arguments(class, method, args)?;
        let madd = madd_arguments(class, method, wiring, &values)?;
        if let Some(through) = wiring.passthrough {
            if let Some(input) = passed_through(through, &values) {
                return self.transaction(|builder| builder.madd_input(class, input, madd));
            }
        }
        let level = level_argument(class, method, wiring, &values)?;
        let positions = expanding(wiring);
        let mut names = Vec::with_capacity(positions.len());
        let mut expanded = Vec::with_capacity(positions.len());
        for &position in &positions {
            let ArgValue::Input(input) = &values[position] else {
                return Err(if wiring.inputs.contains(&InputSlot::Arg(position)) {
                    wrong_kind(class, method, position, "a number or an output")
                } else {
                    bad_count(class, method, position)
                });
            };
            names.push(method.args[position].name);
            expanded.push(input.clone());
        }
        self.transaction(|builder| {
            if let Some(position) = wiring.silence {
                if let ArgValue::Input(channels) = &values[position] {
                    let channels = pseudo::elements(channels.clone());
                    let channels = lang::replace_zeroes_with_silence(builder, channels)?;
                    values[position] = ArgValue::Input(UGenInput::Array(channels));
                }
            }
            builder.audio_rate_inputs(wiring, &positions, &mut expanded)?;
            let channels = match ugen_class.convert_env {
                Some(form) => builder.convert_env(form, method, &mut values)?,
                None => None,
            };
            let array = channels.or_else(|| array_part(wiring, &values));
            if let Some((position, part)) = &array {
                for value in part.values() {
                    names.push(method.args[*position].name);
                    expanded.push(value);
                }
            }
            let fixed = positions.len();
            let result = builder.expand(
                class,
                &names,
                &expanded,
                wiring.nested_expansion,
                &mut |builder, instance| {
                    for (&position, value) in positions.iter().zip(instance) {
                        values[position] = ArgValue::Input(value.clone());
                    }
                    if let Some((position, part)) = &array {
                        values[*position] = ArgValue::Input(part.array(&instance[fixed..]));
                    }
                    builder.add_instance(class, ugen_class, method, wiring, &values)
                },
            )?;
            let result = match level {
                Some(level) => lang::binary(builder, "*", result, level)?,
                None => result,
            };
            match madd {
                Some([mul, add]) => builder.madd_expanded(class, &[result, mul, add]),
                None => Ok(result),
            }
        })
    }

    /// Multiplies `input` by `mul` and adds `add`, as `MulAdd.new` does, and
    /// so as `madd` does in sclang. Arrays expand as `multiNewList` expands
    /// them. For each instance, it takes the first case that holds, as
    /// `MulAdd:new1` does:
    ///
    /// - `mul` 0 gives `add`, and `mul` 1 with `add` 0 gives `input`.
    /// - `mul` -1 with `add` 0 gives `input.neg`.
    /// - `add` 0 gives `input * mul`, `mul` -1 gives `add - input`, and
    ///   `mul` 1 gives `input + add`.
    /// - A `MulAdd` of `input`, `mul` and `add`, if `input` runs at audio
    ///   rate, or at control rate with `mul` and `add` at control or scalar
    ///   rate. Else a `MulAdd` of `mul`, `input` and `add`, if `mul` meets
    ///   the same rule.
    /// - Else `(input * mul) + add`.
    ///
    /// These operators compute two numbers, and fold as
    /// [`binary_op`](Self::binary_op) folds.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for an empty array, and for an input that
    /// [`add`](Self::add) refuses.
    pub fn madd(
        &mut self,
        input: impl Into<UGenInput>,
        mul: impl Into<UGenInput>,
        add: impl Into<UGenInput>,
    ) -> Result<UGenInput, BuildError> {
        let values = [input.into(), mul.into(), add.into()];
        self.transaction(|builder| builder.madd_expanded(MUL_ADD, &values))
    }

    /// Adds a variant: the defaults of the parameters, with the values of
    /// `values` by control name, as the `variants` of `SynthDef.new` give
    /// them. Its name is `<definition>.<name>`. A name that two controls
    /// share sets the last of them, as the map of names of sclang does.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] for a name that a variant has already, for a control
    /// that the definition does not have, and for more values than the
    /// default of the control has, or than the parameters hold from its
    /// index. A name of [`control_names`](Self::control_names) without a
    /// default takes no value.
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
                .rposition(|control_name| control_name.name == *control)
            else {
                return Err(BuildError::UnknownControl {
                    variant: full_name,
                    control: (*control).to_owned(),
                });
            };
            let start = to_usize(self.control_names[position].index);
            let room = controls.len().saturating_sub(start);
            let size = self.sizes[position].unwrap_or(0).min(room);
            if control_values.len() > size {
                return Err(BuildError::VariantSize {
                    variant: full_name,
                    control: (*control).to_owned(),
                    values: control_values.len(),
                    size,
                });
            }
            if let Some(slots) = controls.get_mut(start..start + control_values.len()) {
                slots.copy_from_slice(control_values);
            }
        }
        self.variants.push(Variant {
            name: full_name,
            values: controls.into_iter().map(to_float32).collect(),
        });
        Ok(())
    }

    /// Ends the build, as `finishBuild` does. It copies a PV chain for its
    /// readers, as `addCopiesIfNeeded` does, and rewrites the graph, as
    /// `optimizeGraph` does. Then it collects the constants, sorts the UGens
    /// and indexes them.
    ///
    /// # Errors
    ///
    /// A [`BuildError`] when the definition breaks a limit of scsynth, and
    /// [`BuildError::FinishBuild`] where `finishBuild` of sclang 3.14.1
    /// throws or writes a broken definition.
    pub fn finish(self) -> Result<SynthDef, BuildError> {
        let mut graph = optimize::Graph::new(self.children, &self.width_first);
        let copies = graph.add_copies(self.max_local_bufs.map(|max| max.node))?;
        if let Some(max) = self.max_local_bufs {
            let count = Source::Number(f64::from(max.count + copies));
            graph.set_input(max.node, 0, count);
        }
        graph.optimize_graph()?;
        let Rewritten { nodes, width_first } = graph.into_rewritten();
        let mut constants = Vec::new();
        let mut constant_index = BTreeMap::new();
        for node in &nodes {
            for source in &node.inputs {
                if let Source::Number(number) = *source {
                    constant_index.entry(number.to_bits()).or_insert_with(|| {
                        constants.push(to_float32(number));
                        to_u32(constants.len() - 1)
                    });
                }
            }
        }
        let antecedents: Vec<Vec<usize>> = nodes
            .iter()
            .zip(&width_first)
            .map(|(node, width_first)| {
                node.inputs
                    .iter()
                    .filter_map(|source| match *source {
                        Source::Output { ugen, .. } => Some(ugen),
                        Source::Number(_) => None,
                    })
                    .chain(width_first.iter().copied())
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
                let node = &nodes[ugen];
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

    /// The inputs that go to audio rate before the expansion. The input of a
    /// delay goes through `asAudioRateInput`. When one argument of
    /// `audio_together` is at audio rate, `K2A.ar` takes each other one, in
    /// order, as `DemandEnvGen.ar` does.
    fn audio_rate_inputs(
        &mut self,
        wiring: Wiring,
        positions: &[usize],
        expanded: &mut [UGenInput],
    ) -> Result<(), BuildError> {
        let slot_of = |argument: usize| positions.iter().position(|&position| position == argument);
        if let Some(slot) = wiring.audio_input.and_then(slot_of) {
            expanded[slot] = lang::as_audio_rate_input(self, expanded[slot].clone())?;
        }
        let together: Vec<usize> = wiring
            .audio_together
            .unwrap_or_default()
            .iter()
            .filter_map(|&argument| slot_of(argument))
            .collect();
        let audio = |input: &UGenInput| lang::rate(input) == Some(Rate::Audio);
        if together.iter().any(|&slot| audio(&expanded[slot])) {
            for slot in together {
                if !audio(&expanded[slot]) {
                    let input = ArgValue::Input(expanded[slot].clone());
                    expanded[slot] = self.ugen("K2A", "ar", &[("in", input)])?;
                }
            }
        }
        Ok(())
    }

    /// The `envelope` of `values` as `convertEnv` takes an `Env`: the array
    /// of `EnvGen`, which the `Refs` expansion takes, or the channels of
    /// `IEnvGen`. Several channels of `IEnvGen` expand, as instances, with
    /// the index.
    fn convert_env(
        &mut self,
        form: ConvertEnv,
        method: &RateMethod,
        values: &mut [ArgValue],
    ) -> Result<Option<(usize, ArrayPart)>, BuildError> {
        let Some(position) = method.args.iter().position(|arg| arg.name == "envelope") else {
            return Ok(None);
        };
        let ArgValue::Env(env) = &values[position] else {
            return Ok(None);
        };
        if form == ConvertEnv::Array {
            values[position] = ArgValue::Input(env.as_array()?);
            return Ok(None);
        }
        let env = env.clone();
        let channels = pseudo::elements(pseudo::env::array_for_interpolation(self, &env)?);
        if let [channel] = channels.as_slice() {
            values[position] = ArgValue::Input(channel.clone());
            return Ok(None);
        }
        values[position] = ArgValue::Input(UGenInput::Array(channels.clone()));
        Ok(Some((position, ArrayPart::Instances(channels))))
    }

    /// Runs the body of a pseudo-UGen. A body that fails adds no UGen.
    fn pseudo(
        &mut self,
        class: &'static str,
        method: &'static RateMethod,
        args: &[(&str, ArgValue)],
    ) -> Result<UGenInput, BuildError> {
        let values = bind_optional(class, method, args)?;
        self.transaction(|builder| pseudo::build(builder, class, method, values))
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
                    let lags: Vec<UGenInput> = lags.iter().map(|&lag| lag.into()).collect();
                    outputs.extend(self.add_control("LagControl", Rate::Control, values, &lags)?);
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
        lags: &[UGenInput],
    ) -> Result<Vec<OutputProxy>, BuildError> {
        let special_index = i16::try_from(self.controls.len())
            .map_err(|_| BuildError::TooManyParameters(self.controls.len()))?;
        let mut inputs = Vec::with_capacity(lags.len());
        for (input, lag) in lags.iter().enumerate() {
            inputs.push(self.source(name, input, lag)?);
        }
        self.controls.extend_from_slice(values);
        let index = self.push(Node {
            name: name.to_owned(),
            rate,
            inputs,
            outputs: vec![rate; values.len()],
            special_index,
            width_first_before: self.width_first.len(),
        });
        Ok((0..values.len())
            .map(|output| OutputProxy {
                serial: self.serials[index],
                ugen: index,
                output: to_u32(output),
                rate,
            })
            .collect())
    }

    /// Names the next parameter, with a default of `size` values, as
    /// `addIr`, `addKr`, `addTr` and `addAr` do for a `NamedControl`.
    pub(crate) fn add_control_name(&mut self, name: &str, size: usize) {
        self.control_names.push(ControlName {
            name: name.to_owned(),
            index: to_u32(self.controls.len()),
        });
        self.sizes.push(Some(size));
        self.last_name = Some(self.control_names.len() - 1);
    }

    /// The table of `NamedControl` for the build.
    pub(crate) fn named_controls(&self) -> &[NamedEntry] {
        &self.named
    }

    /// Adds a name to the table of `NamedControl`.
    pub(crate) fn push_named_control(&mut self, entry: NamedEntry) {
        self.named.push(entry);
    }

    /// Adds a control UGen of `values` at the next parameter, as the `init`
    /// of `Control`, `AudioControl` and `LagControl` does. A `Control` or a
    /// `TrigControl` gives its values to the last name, if that name has no
    /// default.
    pub(crate) fn control_ugen(
        &mut self,
        class: &str,
        rate: Rate,
        values: &[f64],
        lags: &[UGenInput],
    ) -> Result<UGenInput, BuildError> {
        self.add_control(class, rate, values, lags)?;
        if matches!(class, "Control" | "TrigControl") {
            if let Some(size @ None) = self.last_name.and_then(|name| self.sizes.get_mut(name)) {
                *size = Some(values.len());
            }
        }
        Ok(self.outputs_of(self.children.len() - 1))
    }

    /// Runs `build`, and gives back the build as it was if it fails: its
    /// UGens, its parameters and its names.
    fn transaction<T>(
        &mut self,
        build: impl FnOnce(&mut Self) -> Result<T, BuildError>,
    ) -> Result<T, BuildError> {
        let (children, width_first) = (self.children.len(), self.width_first.len());
        let (controls, names, entries) = (
            self.controls.len(),
            self.control_names.len(),
            self.named.len(),
        );
        let (max_local_bufs, last_name) = (self.max_local_bufs, self.last_name);
        let last_size = last_name.and_then(|name| self.sizes.get(name).copied());
        let result = build(self);
        if result.is_err() {
            self.children.truncate(children);
            self.serials.truncate(children);
            self.width_first.truncate(width_first);
            self.controls.truncate(controls);
            self.control_names.truncate(names);
            self.sizes.truncate(names);
            self.named.truncate(entries);
            self.max_local_bufs = max_local_bufs;
            self.last_name = last_name;
            if let (Some(slot), Some(size)) = (
                last_name.and_then(|name| self.sizes.get_mut(name)),
                last_size,
            ) {
                *slot = size;
            }
        }
        result
    }

    /// The output of the `MaxLocalBufs` of the build, which counts one more
    /// `LocalBuf`. The first call adds it, as `LocalBuf:new1` does.
    pub(crate) fn max_local_bufs(&mut self) -> Result<UGenInput, BuildError> {
        let (node, count) = if let Some(max) = self.max_local_bufs {
            (max.node, max.count)
        } else {
            let node = self.children.len();
            self.add(NewUGen::new("MaxLocalBufs", Rate::Scalar).inputs([0.0.into()]))?;
            (node, 0)
        };
        self.max_local_bufs = Some(MaxLocalBufs {
            node,
            count: count + 1,
        });
        Ok(self.outputs_of(node))
    }

    /// The input `index` of the UGen of `input`, for an output of this
    /// build, as `inputs[index]` gives it in sclang.
    pub(crate) fn input_of(&self, input: &UGenInput, index: usize) -> Option<UGenInput> {
        let UGenInput::Output(proxy) = input else {
            return None;
        };
        if !self.owns(proxy) {
            return None;
        }
        Some(match *self.children.get(proxy.ugen)?.inputs.get(index)? {
            Source::Number(number) => UGenInput::Number(number),
            Source::Output { ugen, output } => UGenInput::Output(OutputProxy {
                serial: self.serials[ugen],
                ugen,
                output,
                rate: self.children[ugen].outputs[to_usize(output)],
            }),
        })
    }

    /// The class of the UGen of `input`, for an output of this build.
    pub(crate) fn class_of(&self, input: &UGenInput) -> Option<&str> {
        match input {
            UGenInput::Output(output) if self.owns(output) => self
                .children
                .get(output.ugen)
                .map(|node| node.name.as_str()),
            _ => None,
        }
    }

    /// Whether `proxy` is an output of a UGen of this build.
    fn owns(&self, proxy: &OutputProxy) -> bool {
        self.serials.get(proxy.ugen) == Some(&proxy.serial)
    }

    /// Adds `node` as the next child, with a new serial, and gives its index.
    fn push(&mut self, node: Node) -> usize {
        self.children.push(node);
        self.serials
            .push(NEXT_SERIAL.fetch_add(1, Ordering::Relaxed));
        self.children.len() - 1
    }

    /// The expansion of `multiNewList` over `values`, the arguments that
    /// expand, named by `names` for the errors. `leaf` adds what one
    /// instance adds. Below the top, an array expands again only if
    /// `nested`.
    pub(crate) fn expand<F>(
        &mut self,
        ugen: &str,
        names: &[&str],
        values: &[UGenInput],
        nested: bool,
        leaf: &mut F,
    ) -> Result<UGenInput, BuildError>
    where
        F: FnMut(&mut Self, &[UGenInput]) -> Result<UGenInput, BuildError>,
    {
        self.expand_at(ugen, names, values, nested, 0, leaf)
    }

    fn expand_at<F>(
        &mut self,
        ugen: &str,
        names: &[&str],
        values: &[UGenInput],
        nested: bool,
        depth: usize,
        leaf: &mut F,
    ) -> Result<UGenInput, BuildError>
    where
        F: FnMut(&mut Self, &[UGenInput]) -> Result<UGenInput, BuildError>,
    {
        let mut size = 0;
        for (name, value) in names.iter().zip(values) {
            if let UGenInput::Array(items) = value {
                let (ugen, argument) = (ugen.to_owned(), (*name).to_owned());
                if items.is_empty() {
                    return Err(BuildError::EmptyInput { ugen, argument });
                }
                if depth > 0 && !nested {
                    return Err(BuildError::NestedArray { ugen, argument });
                }
                size = size.max(items.len());
            }
        }
        if size == 0 {
            return leaf(self, values);
        }
        let mut instances = Vec::with_capacity(size);
        for index in 0..size {
            let instance: Vec<UGenInput> = values
                .iter()
                .map(|value| match value {
                    UGenInput::Array(items) => items[index % items.len()].clone(),
                    other => other.clone(),
                })
                .collect();
            instances.push(self.expand_at(ugen, names, &instance, nested, depth + 1, leaf)?);
        }
        Ok(UGenInput::Array(instances))
    }

    /// One instance of a method of the catalogue, from values without the
    /// arrays that expand.
    fn add_instance(
        &mut self,
        class: &str,
        ugen_class: &UGenClass,
        method: &RateMethod,
        wiring: Wiring,
        values: &[ArgValue],
    ) -> Result<UGenInput, BuildError> {
        let inputs = wire_inputs(class, method, wiring, values)?;
        let outputs = wire_outputs(class, method, wiring, values)?;
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

    fn unary_leaf(&mut self, special_index: i16, a: UGenInput) -> Result<UGenInput, BuildError> {
        let rate = rate_of(&a);
        self.add(
            NewUGen::new(UNARY_OP, rate)
                .inputs([a])
                .special_index(special_index),
        )
    }

    /// One instance of `BinaryOpUGen.new`: the folds of `new1`, in their
    /// order, or a `BinaryOpUGen`.
    fn binary_leaf(
        &mut self,
        op: &str,
        special_index: i16,
        a: UGenInput,
        b: UGenInput,
    ) -> Result<UGenInput, BuildError> {
        match op {
            "*" if is_number(&a, 0.0) || is_number(&b, 0.0) => return Ok(UGenInput::Number(0.0)),
            "*" if is_number(&a, 1.0) => return Ok(b),
            "*" if is_number(&a, -1.0) => return self.neg(b),
            "*" | "/" if is_number(&b, 1.0) => return Ok(a),
            "*" | "/" if is_number(&b, -1.0) => return self.neg(a),
            "+" if is_number(&a, 0.0) => return Ok(b),
            "-" if is_number(&a, 0.0) => return self.neg(b),
            "+" | "-" if is_number(&b, 0.0) => return Ok(a),
            _ => {}
        }
        let rate = rate_of(&a).max(rate_of(&b));
        self.add(
            NewUGen::new(BINARY_OP, rate)
                .inputs([a, b])
                .special_index(special_index),
        )
    }

    /// `neg` in sclang: a number for a number, and a `UnaryOpUGen` for an
    /// output.
    fn neg(&mut self, input: UGenInput) -> Result<UGenInput, BuildError> {
        if let UGenInput::Number(number) = input {
            return Ok(UGenInput::Number(-number));
        }
        let special_index = operator::unary_index("neg")
            .ok_or_else(|| BuildError::UnknownOperator("neg".to_owned()))?;
        self.unary_leaf(special_index, input)
    }

    /// `input.madd(mul, add)` in sclang: `(input * mul) + add` for a number,
    /// as `SimpleNumber:madd` computes it, and else `MulAdd.new`.
    fn madd_input(
        &mut self,
        class: &str,
        input: UGenInput,
        madd: Option<[UGenInput; 2]>,
    ) -> Result<UGenInput, BuildError> {
        let [mul, add] = madd.unwrap_or([UGenInput::Number(1.0), UGenInput::Number(0.0)]);
        if let UGenInput::Number(_) = input {
            let product = lang::binary(self, "*", input, mul)?;
            return lang::binary(self, "+", product, add);
        }
        self.madd_expanded(class, &[input, mul, add])
    }

    /// `MulAdd.new` over `values`: the input, `mul` and `add`, with the class
    /// that the errors name.
    fn madd_expanded(&mut self, ugen: &str, values: &[UGenInput]) -> Result<UGenInput, BuildError> {
        self.expand(
            ugen,
            &["in", "mul", "add"],
            values,
            true,
            &mut |builder, instance| {
                builder.madd_leaf(
                    instance[0].clone(),
                    instance[1].clone(),
                    instance[2].clone(),
                )
            },
        )
    }

    /// One instance of `MulAdd.new`: the cases of `MulAdd:new1`.
    fn madd_leaf(
        &mut self,
        input: UGenInput,
        mul: UGenInput,
        add: UGenInput,
    ) -> Result<UGenInput, BuildError> {
        if is_number(&mul, 0.0) {
            return Ok(add);
        }
        let (minus, no_mul, no_add) = (
            is_number(&mul, -1.0),
            is_number(&mul, 1.0),
            is_number(&add, 0.0),
        );
        if no_mul && no_add {
            return Ok(input);
        }
        if minus && no_add {
            return self.neg(input);
        }
        if no_add {
            return lang::binary(self, "*", input, mul);
        }
        if minus {
            return lang::binary(self, "-", add, input);
        }
        if no_mul {
            return lang::binary(self, "+", input, add);
        }
        let rates = [rate_of(&input), rate_of(&mul), rate_of(&add)];
        if can_be_mul_add(rates[0], rates[1], rates[2]) {
            return self.mul_add(input, mul, add);
        }
        if can_be_mul_add(rates[1], rates[0], rates[2]) {
            return self.mul_add(mul, input, add);
        }
        let product = lang::binary(self, "*", input, mul)?;
        lang::binary(self, "+", product, add)
    }

    fn mul_add(
        &mut self,
        a: UGenInput,
        b: UGenInput,
        c: UGenInput,
    ) -> Result<UGenInput, BuildError> {
        let rate = least_by_name([rate_of(&a), rate_of(&b), rate_of(&c)]).unwrap_or(Rate::Scalar);
        self.add(NewUGen::new(MUL_ADD, rate).inputs([a, b, c]))
    }

    fn source(&self, ugen: &str, input: usize, value: &UGenInput) -> Result<Source, BuildError> {
        match value {
            UGenInput::Number(number) if number.is_nan() => Err(BuildError::NotANumber {
                ugen: ugen.to_owned(),
                input,
            }),
            UGenInput::Number(number) => Ok(Source::Number(*number)),
            UGenInput::Output(output) if !self.owns(output) => Err(BuildError::ForeignOutput {
                ugen: ugen.to_owned(),
                input,
            }),
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
                    serial: self.serials[index],
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

const UNARY_OP: &str = "UnaryOpUGen";
const BINARY_OP: &str = "BinaryOpUGen";
const MUL_ADD: &str = "MulAdd";

/// The rate of a number or an output, which is all that an expansion leaves.
fn rate_of(value: &UGenInput) -> Rate {
    value.rate().unwrap_or(Rate::Scalar)
}

/// Whether `input` is the number `number`, as `==` compares them in sclang.
#[allow(clippy::float_cmp)]
fn is_number(input: &UGenInput, number: f64) -> bool {
    matches!(input, UGenInput::Number(value) if *value == number)
}

/// `MulAdd:canBeMulAdd`: whether a `MulAdd` takes inputs at these rates in
/// this order.
pub(crate) fn can_be_mul_add(input: Rate, mul: Rate, add: Rate) -> bool {
    let slow = |rate: Rate| matches!(rate, Rate::Control | Rate::Scalar);
    match input {
        Rate::Audio => true,
        Rate::Control => slow(mul) && slow(add),
        Rate::Scalar | Rate::Demand => false,
    }
}

/// The input that a method of `through` gives back in place of an instance:
/// an input at scalar rate, or any input with the number 0 in the zero
/// argument.
fn passed_through(through: Passthrough, values: &[ArgValue]) -> Option<UGenInput> {
    let ArgValue::Input(input) = &values[through.input] else {
        return None;
    };
    let zero = through.zero.is_some_and(|position| {
        matches!(values[position], ArgValue::Input(UGenInput::Number(number)) if number == 0.0)
    });
    (zero || lang::rate(input) == Some(Rate::Scalar)).then(|| input.clone())
}

/// The positions of the arguments that expand: those of the wire, and the
/// count of the outputs, as the method gives them `multiNew`.
fn expanding(wiring: Wiring) -> Vec<usize> {
    let mut positions: Vec<usize> = wiring
        .inputs
        .iter()
        .filter_map(|slot| match *slot {
            InputSlot::Arg(position) => Some(position),
            _ => None,
        })
        .collect();
    if let Outputs::Arg { arg, .. } = wiring.outputs {
        positions.push(arg);
    }
    positions.sort_unstable();
    positions.dedup();
    positions
}

/// What the array argument of a method adds to the expansion of `ugen`.
enum ArrayPart {
    /// Its elements, which expand as the arguments do.
    Elements(Vec<UGenInput>),
    /// The array of each instance, which the expansion takes by its index.
    Instances(Vec<UGenInput>),
}

impl ArrayPart {
    /// The values that the part adds to the expansion.
    fn values(&self) -> Vec<UGenInput> {
        match self {
            Self::Elements(elements) => elements.clone(),
            Self::Instances(arrays) => vec![UGenInput::Array(
                (0..arrays.len()).map(pseudo::number).collect(),
            )],
        }
    }

    /// The array argument of one instance, from what the expansion gives the
    /// part.
    fn array(&self, values: &[UGenInput]) -> UGenInput {
        match (self, values) {
            (Self::Instances(arrays), &[UGenInput::Number(index)]) => {
                arrays[count_to_usize(index)].clone()
            }
            _ => UGenInput::Array(values.to_vec()),
        }
    }
}

/// The position of the array argument of `wiring`, and what it adds to the
/// expansion, as its `ArrayExpansion` takes an array among its elements.
/// `None` where the array goes whole to each instance.
fn array_part(wiring: Wiring, values: &[ArgValue]) -> Option<(usize, ArrayPart)> {
    let position = wiring.inputs.iter().find_map(|slot| match *slot {
        InputSlot::ArgArray(position) => Some(position),
        _ => None,
    })?;
    let ArgValue::Input(array) = &values[position] else {
        return None;
    };
    let is_array = |item: &UGenInput| matches!(item, UGenInput::Array(_));
    let part = match (wiring.array_expansion?, array) {
        (ArrayExpansion::Elements, _) => ArrayPart::Elements(pseudo::elements(array.clone())),
        (ArrayExpansion::Flop, UGenInput::Array(items)) if items.iter().any(is_array) => {
            ArrayPart::Instances(items.clone())
        }
        (ArrayExpansion::Refs, UGenInput::Array(items)) if items.first().is_some_and(is_array) => {
            ArrayPart::Instances(items.clone())
        }
        _ => return None,
    };
    Some((position, part))
}

/// sclang keeps its Floats in 64 bits and writes them with `putFloat`, which
/// rounds them to float32.
#[allow(clippy::cast_possible_truncation)]
fn to_float32(value: f64) -> f32 {
    value as f32
}

/// A rate method that `ugen` finds by the name of its class.
enum Found {
    /// A method of a UGen class.
    UGen(&'static UGenClass, &'static RateMethod),
    /// A method of a pseudo-UGen class outside `UGen`, with the name of the
    /// class.
    Pseudo(&'static str, &'static RateMethod),
}

/// The method `selector` of the UGen class or the pseudo class `class`.
fn find_method(class: &str, selector: &str) -> Result<Found, BuildError> {
    let unknown_method = || BuildError::UnknownMethod {
        class: class.to_owned(),
        selector: selector.to_owned(),
    };
    if let Some(ugen_class) = catalog::lookup(class) {
        let method = ugen_class.method(selector).ok_or_else(unknown_method)?;
        return Ok(Found::UGen(ugen_class, method));
    }
    let pseudo_class =
        catalog::pseudo_lookup(class).ok_or_else(|| BuildError::UnknownClass(class.to_owned()))?;
    let method = pseudo_class.method(selector).ok_or_else(unknown_method)?;
    Ok(Found::Pseudo(pseudo_class.name, method))
}

/// The value of each argument of `method`: the one that `args` gives by
/// name, or the default.
fn bind_arguments(
    class: &str,
    method: &RateMethod,
    args: &[(&str, ArgValue)],
) -> Result<Vec<ArgValue>, BuildError> {
    bind_optional(class, method, args)?
        .into_iter()
        .zip(method.args)
        .map(|(value, arg)| {
            value.ok_or_else(|| BuildError::MissingArgument {
                class: class.to_owned(),
                argument: arg.name.to_owned(),
            })
        })
        .collect()
}

/// The value of each argument of `method`: the one that `args` gives by
/// name, or the default, or none for a nil default.
/// The instance method of `PV_ChainUGen` with this selector, and its
/// arguments bound by name. `func` takes no value by name: the closure of
/// the call takes its place.
fn pv_values(
    selector: &str,
    args: &[(&str, ArgValue)],
) -> Result<(&'static RateMethod, Vec<Option<ArgValue>>), BuildError> {
    let method = catalog::pv_method(selector).ok_or_else(|| BuildError::UnknownMethod {
        class: "PV_ChainUGen".to_owned(),
        selector: selector.to_owned(),
    })?;
    if args.iter().any(|(name, _)| *name == "func") {
        return Err(BuildError::WrongKind {
            class: "PV_ChainUGen".to_owned(),
            argument: "func".to_owned(),
            expected: "the closure of the call",
        });
    }
    Ok((method, bind_optional("PV_ChainUGen", method, args)?))
}

pub(crate) fn bind_optional(
    class: &str,
    method: &RateMethod,
    args: &[(&str, ArgValue)],
) -> Result<Vec<Option<ArgValue>>, BuildError> {
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
    Ok(method
        .args
        .iter()
        .zip(values)
        .map(|(arg, value)| match (value, arg.default) {
            (Some(value), _) => Some(value),
            (None, Some(ArgDefault::Float(number))) => Some(ArgValue::from(number)),
            (None, Some(ArgDefault::Symbol(text)))
                if matches!(arg.kind, ArgKind::Symbol | ArgKind::Curve) =>
            {
                Some(ArgValue::from(text))
            }
            (None, Some(ArgDefault::Floats(numbers))) => Some(ArgValue::Input(UGenInput::Array(
                numbers.iter().map(|&number| number.into()).collect(),
            ))),
            (None, Some(ArgDefault::Bool(value))) => Some(ArgValue::Bool(value)),
            _ => None,
        })
        .collect())
}

/// The `mul` and `add` that `ugen` hands `madd`. `None` if the method
/// applies no `madd`, or applies it to a UGen without outputs, which takes
/// only the defaults, 1 and 0.
fn madd_arguments(
    class: &str,
    method: &RateMethod,
    wiring: Wiring,
    values: &[ArgValue],
) -> Result<Option<[UGenInput; 2]>, BuildError> {
    if !wiring.madd {
        return Ok(None);
    }
    let no_outputs = matches!(wiring.outputs, Outputs::Fixed(rates) if rates.is_empty());
    let mut madd = [UGenInput::Number(1.0), UGenInput::Number(0.0)];
    for (position, (arg, value)) in method.args.iter().zip(values).enumerate() {
        let slot = match arg.name {
            "mul" => 0,
            "add" => 1,
            _ => continue,
        };
        let ArgValue::Input(input) = value else {
            return Err(wrong_kind(class, method, position, "a number or an output"));
        };
        if no_outputs && *input != madd[slot] {
            return Err(BuildError::MaddNotSupported(arg.name.to_owned()));
        }
        madd[slot] = input.clone();
    }
    Ok(if no_outputs { None } else { Some(madd) })
}

/// The `level` that the method multiplies the result of its expansion by, as
/// `LinXFade2` does, or `None`.
fn level_argument(
    class: &str,
    method: &RateMethod,
    wiring: Wiring,
    values: &[ArgValue],
) -> Result<Option<UGenInput>, BuildError> {
    let position = method.args.iter().position(|arg| arg.name == "level");
    match position
        .filter(|_| wiring.level)
        .map(|position| (position, &values[position]))
    {
        None => Ok(None),
        Some((_, ArgValue::Input(level))) => Ok(Some(level.clone())),
        Some((position, _)) => Err(wrong_kind(class, method, position, "a number or an output")),
    }
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

fn nested_array(class: &str, method: &RateMethod, position: usize) -> BuildError {
    BuildError::NestedArray {
        ugen: class.to_owned(),
        argument: method.args[position].name.to_owned(),
    }
}

fn bad_count(class: &str, method: &RateMethod, position: usize) -> BuildError {
    BuildError::BadCount {
        class: class.to_owned(),
        argument: method.args[position].name.to_owned(),
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
                    return Err(nested_array(class, method, position));
                }
                ArgValue::Input(input) => inputs.push(input.clone()),
                ArgValue::Text(_) | ArgValue::Bool(_) | ArgValue::Env(_) | ArgValue::Curve(_) => {
                    return Err(wrong_kind(class, method, position, "a number or an output"));
                }
            },
            InputSlot::ArgArray(position) => match &values[position] {
                ArgValue::Input(UGenInput::Array(items)) => {
                    if items.iter().any(|item| matches!(item, UGenInput::Array(_))) {
                        return Err(nested_array(class, method, position));
                    }
                    inputs.extend(items.iter().cloned());
                }
                ArgValue::Input(input) => inputs.push(input.clone()),
                ArgValue::Text(_) | ArgValue::Bool(_) | ArgValue::Env(_) | ArgValue::Curve(_) => {
                    return Err(wrong_kind(class, method, position, "an array"));
                }
            },
            InputSlot::ArgString(position) => match &values[position] {
                ArgValue::Text(text) => inputs.extend(symbol_codes(text)),
                ArgValue::Input(_) | ArgValue::Bool(_) | ArgValue::Env(_) | ArgValue::Curve(_) => {
                    return Err(wrong_kind(class, method, position, "a symbol"));
                }
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
    match wiring.outputs {
        Outputs::Fixed(rates) => Ok(rates.to_vec()),
        Outputs::Arg { arg, rate } => match values[arg] {
            ArgValue::Input(UGenInput::Number(count))
                if count >= 1.0 && count.fract() == 0.0 && count <= f64::from(u16::MAX) =>
            {
                Ok(vec![rate; count_to_usize(count)])
            }
            _ => Err(bad_count(class, method, arg)),
        },
        Outputs::ArraySize { arg, rate } => match &values[arg] {
            ArgValue::Input(UGenInput::Array(items)) if !items.is_empty() => {
                Ok(vec![rate; items.len()])
            }
            ArgValue::Input(UGenInput::Array(_))
            | ArgValue::Text(_)
            | ArgValue::Bool(_)
            | ArgValue::Env(_)
            | ArgValue::Curve(_) => Err(bad_count(class, method, arg)),
            ArgValue::Input(_) => Ok(vec![rate]),
        },
    }
}

/// The inputs of a symbol, as sclang writes them: its size, then each byte
/// as `Char:ascii` gives it, from -128 to 127.
pub(crate) fn symbol_codes(text: &str) -> Vec<UGenInput> {
    let mut codes = vec![UGenInput::from(f64::from(to_u32(text.len())))];
    codes.extend(
        text.bytes()
            .map(|byte| UGenInput::from(f64::from(i8::from_ne_bytes([byte])))),
    );
    codes
}

/// A whole number from 1 to `u16::MAX`, as a count of outputs.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn count_to_usize(count: f64) -> usize {
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
    use super::{Arg, ArgRate, NewUGen, Source, SynthDefBuilder};
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
        // { |freq = 440, a_in = 0, i_n = 1, t_g = 0, amps = #[1, 2]| }, fact 20 of
        // the plan of 0.1.
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

    /// A `DC.kr` of `input`.
    fn dc(input: UGenInput) -> NewUGen {
        NewUGen::new("DC", Rate::Control).inputs([input])
    }

    fn foreign() -> BuildError {
        BuildError::ForeignOutput {
            ugen: "DC".to_owned(),
            input: 0,
        }
    }

    #[test]
    fn a_clone_takes_the_outputs_from_before_it_and_none_from_after() {
        let mut builder = SynthDefBuilder::new("fork", vec![]).unwrap();
        let before = builder.add(dc(1.0.into())).unwrap();
        let mut fork = builder.clone();
        let after = fork.add(dc(2.0.into())).unwrap();
        // The index of `after` holds a UGen of `builder` too.
        builder.add(dc(3.0.into())).unwrap();
        assert!(fork.add(dc(before.clone())).is_ok());
        assert!(builder.add(dc(before)).is_ok());
        assert_eq!(builder.add(dc(after)), Err(foreign()));
    }

    #[test]
    fn an_output_of_a_call_that_failed_is_foreign() {
        let mut builder = SynthDefBuilder::new("rollback", vec![]).unwrap();
        let mut kept = None;
        let failed: Result<(), BuildError> = builder.wrap(vec![], |builder, _| {
            kept = Some(builder.add(dc(1.0.into()))?);
            Err(BuildError::UnknownClass("Nothing".to_owned()))
        });
        assert!(failed.is_err());
        // The index of `kept` holds a new UGen.
        builder.add(dc(2.0.into())).unwrap();
        assert_eq!(builder.add(dc(kept.unwrap())), Err(foreign()));
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
        let sum = builder.binary_op("*", 2.0, 3.0).unwrap();
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
    fn an_operator_refuses_an_unknown_name() {
        let mut builder = SynthDefBuilder::new("ops", vec![]).unwrap();
        assert_eq!(
            builder.unary_op("+", 1.0),
            Err(BuildError::UnknownOperator("+".to_owned()))
        );
        assert_eq!(
            builder.binary_op("neg", 1.0, 2.0),
            Err(BuildError::UnknownOperator("neg".to_owned()))
        );
    }

    fn numbers(numbers: &[f64]) -> UGenInput {
        UGenInput::Array(numbers.iter().copied().map(UGenInput::from).collect())
    }

    /// The creation index of each output, in the shape of the input.
    fn shape(input: &UGenInput) -> String {
        match input {
            UGenInput::Number(number) => number.to_string(),
            UGenInput::Output(output) => format!("u{}.{}", output.ugen, output.output),
            UGenInput::Array(items) => {
                let items: Vec<String> = items.iter().map(shape).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }

    #[test]
    fn an_operator_expands_and_the_shorter_array_wraps() {
        let mut builder = SynthDefBuilder::new("ops", vec![]).unwrap();
        let sines = builder
            .ugen(
                "SinOsc",
                "ar",
                &[("freq", numbers(&[440.0, 442.0, 444.0]).into())],
            )
            .unwrap();
        assert_eq!(shape(&sines), "[u0.0, u1.0, u2.0]");
        let products = builder.binary_op("*", sines, numbers(&[0.1, 0.2])).unwrap();
        assert_eq!(shape(&products), "[u3.0, u4.0, u5.0]");
        let negated = builder.unary_op("neg", products).unwrap();
        assert_eq!(shape(&negated), "[u6.0, u7.0, u8.0]");
        let wrapped: Vec<Source> = builder
            .children
            .iter()
            .filter(|node| node.name == "BinaryOpUGen")
            .map(|node| node.inputs[1])
            .collect();
        assert_eq!(
            wrapped,
            [
                Source::Number(0.1),
                Source::Number(0.2),
                Source::Number(0.1)
            ]
        );
    }

    #[test]
    fn a_nested_array_expands_depth_first() {
        let mut builder = SynthDefBuilder::new("nested", vec![]).unwrap();
        let freq = UGenInput::Array(vec![numbers(&[1.0, 2.0]), 3.0.into()]);
        let sines = builder
            .ugen("SinOsc", "ar", &[("freq", freq.into())])
            .unwrap();
        assert_eq!(shape(&sines), "[[u0.0, u1.0], u2.0]");
    }

    #[test]
    fn a_count_expands_with_a_count_for_each_instance() {
        // Fact 6 of the plan of 0.2: In.ar(0, [1, 2]).
        let mut builder = SynthDefBuilder::new("counts", vec![]).unwrap();
        let inputs = builder
            .ugen("In", "ar", &[("numChannels", numbers(&[1.0, 2.0]).into())])
            .unwrap();
        assert_eq!(shape(&inputs), "[u0.0, [u1.0, u1.1]]");
        assert!(matches!(
            builder.ugen("In", "ar", &[("numChannels", numbers(&[1.0, 1.5]).into())]),
            Err(BuildError::BadCount { .. })
        ));
    }

    #[test]
    fn an_empty_array_is_refused_at_any_depth() {
        let mut builder = SynthDefBuilder::new("empty", vec![]).unwrap();
        let empty = || UGenInput::Array(Vec::new());
        let error = |ugen: &str, argument: &str| BuildError::EmptyInput {
            ugen: ugen.to_owned(),
            argument: argument.to_owned(),
        };
        assert_eq!(
            builder.ugen("SinOsc", "ar", &[("freq", empty().into())]),
            Err(error("SinOsc", "freq"))
        );
        let deeper = UGenInput::Array(vec![empty(), 440.0.into()]);
        assert_eq!(
            builder.ugen("SinOsc", "ar", &[("freq", deeper.into())]),
            Err(error("SinOsc", "freq"))
        );
        assert_eq!(
            builder.ugen("In", "ar", &[("numChannels", empty().into())]),
            Err(error("In", "numChannels"))
        );
        assert_eq!(
            builder.unary_op("neg", empty()),
            Err(error("UnaryOpUGen", "a"))
        );
        assert_eq!(
            builder.binary_op("+", numbers(&[1.0, 2.0]), empty()),
            Err(error("BinaryOpUGen", "b"))
        );
    }

    /// The inputs of each UGen of `class` in the build, as `shape` writes
    /// them.
    fn wires(builder: &SynthDefBuilder, class: &str) -> Vec<String> {
        builder
            .children
            .iter()
            .filter(|node| node.name == class)
            .map(|node| {
                let inputs: Vec<String> = node
                    .inputs
                    .iter()
                    .map(|source| match *source {
                        Source::Number(number) => number.to_string(),
                        Source::Output { ugen, output } => format!("u{ugen}.{output}"),
                    })
                    .collect();
                format!("[{}]", inputs.join(", "))
            })
            .collect()
    }

    #[test]
    fn an_array_among_the_channels_expands_out() {
        // Fact 2 of the plan of 0.6: Out.ar(0, [[a, b], c]) makes Out(0, a, c)
        // and Out(0, b, c).
        let mut builder = SynthDefBuilder::new("nested", vec![]).unwrap();
        let sines = builder
            .ugen(
                "SinOsc",
                "ar",
                &[("freq", numbers(&[1.0, 2.0, 3.0]).into())],
            )
            .unwrap();
        let UGenInput::Array(sines) = sines else {
            panic!("three sines give an array");
        };
        let channels = UGenInput::Array(vec![
            UGenInput::Array(vec![sines[0].clone(), sines[1].clone()]),
            sines[2].clone(),
        ]);
        let outs = builder
            .ugen(
                "Out",
                "ar",
                &[("bus", 0.0.into()), ("channelsArray", channels.into())],
            )
            .unwrap();
        assert_eq!(shape(&outs), "[[], []]");
        assert_eq!(
            wires(&builder, "Out"),
            ["[0, u0.0, u2.0]", "[0, u1.0, u2.0]"]
        );
    }

    #[test]
    fn a_zero_inside_the_channels_stays_without_a_zero_at_the_top() {
        // Fact 6 of the plan of 0.6: sclang fails on Out.ar(0, [[0, a], b]).
        let mut builder = SynthDefBuilder::new("zeros", vec![]).unwrap();
        let sine = builder.ugen("SinOsc", "ar", &[]).unwrap();
        let channels =
            UGenInput::Array(vec![UGenInput::Array(vec![0.0.into(), sine.clone()]), sine]);
        assert!(matches!(
            builder.ugen(
                "Out",
                "ar",
                &[("bus", 0.0.into()), ("channelsArray", channels.into())]
            ),
            Err(BuildError::CheckInputs { .. })
        ));
    }

    #[test]
    fn send_reply_flops_an_array_among_its_values() {
        // Fact 3 of the plan of 0.6.
        let mut builder = SynthDefBuilder::new("flop", vec![]).unwrap();
        let trig = builder.ugen("Impulse", "kr", &[]).unwrap();
        let mut reply = |values: UGenInput| {
            builder
                .ugen(
                    "SendReply",
                    "kr",
                    &[
                        ("trig", trig.clone().into()),
                        ("cmdName", "/a".into()),
                        ("values", values.into()),
                    ],
                )
                .map(|replies| shape(&replies))
        };
        let pair_first = UGenInput::Array(vec![numbers(&[2.0, 3.0]), 4.0.into()]);
        assert_eq!(reply(pair_first), Ok("[[], []]".to_owned()));
        let pair_last = UGenInput::Array(vec![4.0.into(), numbers(&[2.0, 3.0])]);
        assert_eq!(reply(pair_last), Ok("[[], []]".to_owned()));
        assert_eq!(
            wires(&builder, "SendReply"),
            [
                "[u0.0, -1, 2, 47, 97, 2, 3]",
                "[u0.0, -1, 2, 47, 97, 4]",
                "[u0.0, -1, 2, 47, 97, 4]",
                "[u0.0, -1, 2, 47, 97, 2, 3]"
            ]
        );
    }

    #[test]
    fn env_gen_takes_each_array_of_its_envelope_whole() {
        // Fact 4 of the plan of 0.6: three gates wrap two envelopes.
        let mut builder = SynthDefBuilder::new("refs", vec![]).unwrap();
        let envelopes = UGenInput::Array(vec![
            numbers(&[0.0, 1.0, -99.0, -99.0, 1.0, 0.5, 1.0, 0.0]),
            numbers(&[0.0, 1.0, -99.0, -99.0, 1.0, 0.25, 1.0, 0.0]),
        ]);
        let gens = builder
            .ugen(
                "EnvGen",
                "kr",
                &[
                    ("envelope", envelopes.into()),
                    ("gate", numbers(&[1.0, 0.5, 0.25]).into()),
                ],
            )
            .unwrap();
        assert_eq!(shape(&gens), "[u0.0, u1.0, u2.0]");
        assert_eq!(
            wires(&builder, "EnvGen")[2],
            "[0.25, 1, 0, 1, 0, 0, 1, -99, -99, 1, 0.5, 1, 0]"
        );
    }

    #[test]
    fn an_array_that_stays_among_the_inputs_is_refused() {
        // Fact 5 of the plan of 0.6: sclang writes a file that scsynth
        // misreads.
        let mut builder = SynthDefBuilder::new("kept", vec![]).unwrap();
        let refused = |ugen: &str, argument: &str| {
            Err(BuildError::NestedArray {
                ugen: ugen.to_owned(),
                argument: argument.to_owned(),
            })
        };
        let late = UGenInput::Array(vec![
            0.0.into(),
            numbers(&[1.0, 2.0]),
            (-99.0).into(),
            (-99.0).into(),
        ]);
        assert_eq!(
            builder.ugen("EnvGen", "kr", &[("envelope", late.into())]),
            refused("EnvGen", "envelope")
        );
        let nested = UGenInput::Array(vec![numbers(&[0.0, 1.0]), 2.0.into()]);
        assert_eq!(
            builder.ugen(
                "IEnvGen",
                "kr",
                &[("envelope", nested.into()), ("index", 0.5.into())]
            ),
            refused("IEnvGen", "envelope")
        );
        let trig = builder.ugen("Impulse", "kr", &[]).unwrap();
        let deep = UGenInput::Array(vec![UGenInput::Array(vec![
            numbers(&[1.0, 2.0]),
            3.0.into(),
        ])]);
        assert_eq!(
            builder.ugen(
                "SendReply",
                "kr",
                &[("trig", trig.into()), ("values", deep.into())]
            ),
            refused("SendReply", "values")
        );
    }

    #[test]
    fn send_reply_expands_one_level() {
        // Fact 18 of the plan of 0.2: SendReply expands with flop.
        let mut builder = SynthDefBuilder::new("reply", vec![]).unwrap();
        let reply = |builder: &mut SynthDefBuilder, trig: UGenInput| {
            builder.ugen(
                "SendReply",
                "kr",
                &[
                    ("trig", trig.into()),
                    ("cmdName", "/tick".into()),
                    ("values", numbers(&[1.0, 2.0]).into()),
                ],
            )
        };
        let impulses = builder
            .ugen(
                "Impulse",
                "kr",
                &[("freq", numbers(&[1.0, 2.0, 3.0]).into())],
            )
            .unwrap();
        let trig = |index: usize| {
            let UGenInput::Array(items) = &impulses else {
                panic!("three impulses give an array");
            };
            items[index].clone()
        };
        let flat = reply(&mut builder, UGenInput::Array(vec![trig(0), trig(1)])).unwrap();
        assert_eq!(shape(&flat), "[[], []]");
        let nested = UGenInput::Array(vec![UGenInput::Array(vec![trig(0), trig(1)]), trig(2)]);
        assert_eq!(
            reply(&mut builder, nested),
            Err(BuildError::NestedArray {
                ugen: "SendReply".to_owned(),
                argument: "trig".to_owned()
            })
        );
    }

    /// The names of the UGens that a build added, with the operators, in
    /// the order of their creation.
    fn names(builder: &SynthDefBuilder) -> Vec<String> {
        builder
            .children
            .iter()
            .map(|node| match node.name.as_str() {
                "BinaryOpUGen" => crate::operator::binary_name(node.special_index)
                    .unwrap_or("?")
                    .to_owned(),
                "UnaryOpUGen" => crate::operator::unary_name(node.special_index)
                    .unwrap_or("?")
                    .to_owned(),
                name => name.to_owned(),
            })
            .collect()
    }

    #[test]
    fn binary_op_folds_as_binary_op_ugen_new1() {
        // Fact 12 of the plan of 0.2.
        let mut builder = SynthDefBuilder::new("folds", vec![]).unwrap();
        let sine = builder.ugen("SinOsc", "ar", &[]).unwrap();
        let mut fold = |op: &str, a: UGenInput, b: UGenInput| builder.binary_op(op, a, b).unwrap();
        let x = || sine.clone();
        let n = UGenInput::from;
        for (op, a, b) in [
            ("*", x(), n(1.0)),
            ("*", n(1.0), x()),
            ("+", x(), n(0.0)),
            ("+", n(0.0), x()),
            ("-", x(), n(0.0)),
            ("/", x(), n(1.0)),
        ] {
            assert_eq!(fold(op, a, b), sine, "{op}");
        }
        assert_eq!(fold("*", x(), n(0.0)), n(0.0));
        assert_eq!(fold("*", n(-0.0), x()), n(0.0));
        assert_eq!(fold("-", n(0.0), n(3.0)), n(-3.0));
        assert_eq!(fold("*", n(1.0), n(2.0)), n(2.0));
        assert_eq!(fold("/", n(3.0), n(-1.0)), n(-3.0));
        assert!(matches!(
            fold("-", n(0.0), n(0.0)),
            UGenInput::Number(zero) if zero.is_sign_negative()
        ));
        let negs = [
            fold("*", x(), n(-1.0)),
            fold("*", n(-1.0), x()),
            fold("-", n(0.0), x()),
            fold("/", x(), n(-1.0)),
        ];
        assert!(negs.iter().all(|neg| neg.rate() == Some(Rate::Audio)));
        let product = fold("*", n(2.0), n(3.0));
        assert_eq!(product.rate(), Some(Rate::Scalar));
        assert_eq!(names(&builder), ["SinOsc", "neg", "neg", "neg", "neg", "*"]);
    }

    #[test]
    fn madd_takes_the_first_case_of_mul_add_new1() {
        // Facts 9 and 10 of the plan of 0.2.
        let mut builder = SynthDefBuilder::new("madd", vec![Arg::new("k", 1.0)]).unwrap();
        let control = builder.arguments()[0].clone();
        let audio = builder.ugen("SinOsc", "ar", &[]).unwrap();
        let n = UGenInput::from;
        assert_eq!(builder.madd(audio.clone(), 0.0, 0.5).unwrap(), n(0.5));
        assert_eq!(builder.madd(audio.clone(), 1.0, 0.0).unwrap(), audio);
        assert_eq!(builder.madd(2.0, 3.0, 4.0).unwrap(), n(10.0));
        assert_eq!(builder.madd(2.0, -1.0, 0.0).unwrap(), n(-2.0));
        builder.madd(audio.clone(), -1.0, 0.0).unwrap();
        builder.madd(audio.clone(), 0.5, 0.0).unwrap();
        builder.madd(audio.clone(), -1.0, 0.5).unwrap();
        builder.madd(audio.clone(), 1.0, 0.5).unwrap();
        let fused = builder.madd(audio.clone(), 0.5, 0.25).unwrap();
        assert_eq!(fused.rate(), Some(Rate::Audio));
        let swapped = builder.madd(control.clone(), audio.clone(), 0.25).unwrap();
        assert_eq!(swapped.rate(), Some(Rate::Audio));
        let slow = builder
            .madd(control.clone(), control.clone(), 0.25)
            .unwrap();
        assert_eq!(slow.rate(), Some(Rate::Control));
        builder.madd(control, 0.5, audio.clone()).unwrap();
        builder.madd(2.0, 3.0, audio).unwrap();
        assert_eq!(
            names(&builder),
            [
                "Control", "SinOsc", "neg", "*", "-", "+", "MulAdd", "MulAdd", "MulAdd", "*", "+",
                "+"
            ]
        );
        let UGenInput::Output(swapped) = swapped else {
            panic!("{swapped:?} is no output");
        };
        assert_eq!(
            builder.children[swapped.ugen].inputs[..2],
            [
                Source::Output { ugen: 1, output: 0 },
                Source::Output { ugen: 0, output: 0 }
            ]
        );
    }

    #[test]
    fn a_mul_add_with_a_demand_input_runs_at_audio_rate() {
        // Fact 11 of the plan of 0.2.
        let mut builder = SynthDefBuilder::new("demand", vec![]).unwrap();
        let audio = builder.ugen("SinOsc", "ar", &[]).unwrap();
        let demand = builder.add(NewUGen::new("Dseq", Rate::Demand)).unwrap();
        let fused = builder.madd(audio, demand.clone(), 0.5).unwrap();
        assert_eq!(fused.rate(), Some(Rate::Audio));
        assert_eq!(
            builder.binary_op("*", demand, 2.0).unwrap().rate(),
            Some(Rate::Demand)
        );
    }

    #[test]
    fn ugen_applies_madd_once_to_the_whole_expansion() {
        // Fact 8 of the plan of 0.2.
        let mut builder = SynthDefBuilder::new("madd", vec![]).unwrap();
        let sines = builder
            .ugen(
                "SinOsc",
                "ar",
                &[
                    ("freq", numbers(&[440.0, 442.0]).into()),
                    ("mul", numbers(&[0.1, 0.2, 0.3]).into()),
                ],
            )
            .unwrap();
        assert_eq!(shape(&sines), "[u2.0, u3.0, u4.0]");
        assert_eq!(
            builder.ugen(
                "SinOsc",
                "ar",
                &[("mul", UGenInput::Array(Vec::new()).into())]
            ),
            Err(BuildError::EmptyInput {
                ugen: "SinOsc".to_owned(),
                argument: "mul".to_owned()
            })
        );
        assert_eq!(
            builder.madd(1.0, UGenInput::Array(Vec::new()), 0.0),
            Err(BuildError::EmptyInput {
                ugen: "MulAdd".to_owned(),
                argument: "mul".to_owned()
            })
        );
    }

    #[test]
    fn a_ugen_without_outputs_takes_only_the_default_mul_and_add() {
        // Fact 20 of the plan of 0.2.
        let mut builder = SynthDefBuilder::new("bela", vec![]).unwrap();
        let sine = builder.ugen("SinOsc", "ar", &[]).unwrap();
        let out = |builder: &mut SynthDefBuilder, mul: f64, add: f64| {
            builder.ugen(
                "AnalogOut",
                "ar",
                &[
                    ("output", sine.clone().into()),
                    ("mul", mul.into()),
                    ("add", add.into()),
                ],
            )
        };
        assert_eq!(
            out(&mut builder, 1.0, 0.0),
            Ok(UGenInput::Array(Vec::new()))
        );
        assert_eq!(
            out(&mut builder, 1.0, 0.5),
            Err(BuildError::MaddNotSupported("add".to_owned()))
        );
    }

    #[test]
    fn a_call_that_fails_adds_no_ugen() {
        let mut builder = SynthDefBuilder::new("atomic", vec![]).unwrap();
        let freq = numbers(&[440.0, f64::NAN]);
        assert_eq!(
            builder.ugen("SinOsc", "ar", &[("freq", freq.clone().into())]),
            Err(BuildError::NotANumber {
                ugen: "SinOsc".to_owned(),
                input: 0
            })
        );
        assert!(builder.binary_op("+", freq, 1.0).is_err());
        out(&mut builder, Rate::Control, vec![0.0.into(), 1.0.into()]);
        let def = builder.finish().unwrap();
        assert_eq!(def.children.len(), 1);
        assert_eq!(def.constants, [0.0, 1.0]);
    }

    #[test]
    fn a_message_that_fails_adds_no_ugen() {
        // lincurve adds the UGens of its arithmetic before it finds that
        // Select has no method at scalar rate.
        let mut builder = SynthDefBuilder::new("atomic", vec![]).unwrap();
        let value = builder.ugen("Rand", "new", &[]).unwrap();
        let curve = builder.ugen("DC", "kr", &[]).unwrap();
        let children = builder.children.len();
        let args = [("curve", ArgValue::from(curve))];
        assert!(builder.send(value, "lincurve", &args).is_err());
        assert_eq!(builder.children.len(), children);
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
            refuse("Dunique", "new", &[]),
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
            refuse("AnalogOut", "ar", &[("mul", 2.0.into())]),
            BuildError::MaddNotSupported("mul".to_owned())
        );
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
        // Out.ar(0, 0.5), which sclang refuses, fact 34 of the plan of 0.1.
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
