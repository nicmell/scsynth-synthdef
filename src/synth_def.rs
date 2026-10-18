//! A synth definition after the build, as a definition file holds it.

use crate::error::LimitError;
use crate::file::write_def_file;
use crate::rate::Rate;

/// A synth definition after the build: what sclang writes after
/// `finishBuild`, and what scsynth reads.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SynthDef {
    /// The name that `/s_new` takes.
    pub name: String,
    /// The constants that the UGens read, by index.
    pub constants: Vec<f32>,
    /// The initial values of the parameters.
    pub controls: Vec<f32>,
    /// The names of the parameters, in file order. Each names the index of
    /// its first value.
    pub control_names: Vec<ControlName>,
    /// The UGens, in the order in which scsynth runs them. The position of a
    /// UGen is its index in an [`InputSpec`].
    pub children: Vec<UGen>,
    /// Other initial values of the parameters, each under a name of its own.
    pub variants: Vec<Variant>,
}

impl SynthDef {
    /// The bytes of a definition file that holds this definition alone, in
    /// version 2, as `SynthDef:asBytes` writes them.
    ///
    /// # Errors
    ///
    /// A [`LimitError`] when the definition breaks a limit of scsynth.
    pub fn as_bytes(&self) -> Result<Vec<u8>, LimitError> {
        write_def_file(std::slice::from_ref(self))
    }
}

/// An entry of the names table: a parameter name and the index of its first
/// value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlName {
    /// The name, as `/n_set` takes it.
    pub name: String,
    /// The index of its first value among the parameters.
    pub index: u32,
}

/// A UGen of a definition: a unit that scsynth creates in each synth.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UGen {
    /// The name of the unit: the class name in sclang.
    pub name: String,
    /// The rate at which the unit runs.
    pub rate: Rate,
    /// Where each input comes from.
    pub inputs: Vec<InputSpec>,
    /// The rate of each output.
    pub outputs: Vec<Rate>,
    /// A number that some units read, such as the operator of a
    /// `BinaryOpUGen`. It is 0 for most units.
    pub special_index: i16,
}

/// Where an input of a UGen comes from: the input-spec of the format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputSpec {
    /// A constant of the definition, by its index.
    Constant(u32),
    /// An output of an earlier UGen.
    UGen {
        /// The position of the UGen among the children.
        index: u32,
        /// The output of that UGen.
        output: u32,
    },
}

/// Other initial values of the parameters, under a name of their own.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variant {
    /// The full name, `<definition>.<variant>`, as `/s_new` takes it.
    pub name: String,
    /// One value per parameter.
    pub values: Vec<f32>,
}
