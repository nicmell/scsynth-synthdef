//! The values that a build passes from UGen to UGen.

use crate::rate::Rate;

/// An input of a UGen during a build: a number, an output of a UGen, or an
/// array of them. A builder also gives its UGens back as this type.
#[derive(Clone, Debug, PartialEq)]
pub enum UGenInput {
    /// A number, as a Float of sclang. It becomes a constant.
    Number(f64),
    /// An output of a UGen of the build.
    Output(OutputProxy),
    /// Several inputs, such as the outputs of a UGen with several outputs.
    Array(Vec<UGenInput>),
}

impl UGenInput {
    /// The rate of a number, which is scalar, or of an output. An array has
    /// no rate of its own.
    #[must_use]
    pub fn rate(&self) -> Option<Rate> {
        match self {
            Self::Number(_) => Some(Rate::Scalar),
            Self::Output(output) => Some(output.rate),
            Self::Array(_) => None,
        }
    }
}

impl From<f64> for UGenInput {
    fn from(number: f64) -> Self {
        Self::Number(number)
    }
}

impl From<i32> for UGenInput {
    fn from(number: i32) -> Self {
        Self::Number(number.into())
    }
}

impl From<OutputProxy> for UGenInput {
    fn from(output: OutputProxy) -> Self {
        Self::Output(output)
    }
}

impl From<Vec<UGenInput>> for UGenInput {
    fn from(inputs: Vec<UGenInput>) -> Self {
        Self::Array(inputs)
    }
}

/// An output of a UGen of a build: the `OutputProxy` of sclang.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OutputProxy {
    pub(crate) build: u64,
    pub(crate) ugen: usize,
    pub(crate) output: u32,
    pub(crate) rate: Rate,
}

impl OutputProxy {
    /// The rate of the output.
    #[must_use]
    pub fn rate(&self) -> Rate {
        self.rate
    }

    /// The index of the output among the outputs of its UGen.
    #[must_use]
    pub fn output_index(&self) -> u32 {
        self.output
    }
}
