//! Why bytes are not a definition file, and why a definition breaks a limit
//! of scsynth.

use std::fmt;

/// Why bytes do not read as a definition file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadError {
    offset: usize,
    kind: ReadErrorKind,
}

/// What is wrong with the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReadErrorKind {
    /// The bytes end inside a field.
    #[non_exhaustive]
    UnexpectedEnd {
        /// What the field holds.
        expected: &'static str,
    },
    /// The bytes go on after the last definition.
    TrailingBytes,
    /// The bytes do not start with `SCgf`.
    BadMagic,
    /// The version is not 0, 1 or 2.
    UnsupportedVersion(i32),
    /// A count is negative.
    NegativeCount(i32),
    /// An index is negative, and it is not the -1 that marks a constant.
    NegativeIndex(i32),
    /// A rate is not 0 to 3.
    UnknownRate(i8),
    /// A name is not UTF-8.
    NotUtf8,
    /// A definition breaks a limit of scsynth.
    Limit(LimitError),
}

impl ReadError {
    pub(crate) fn new(offset: usize, kind: ReadErrorKind) -> Self {
        Self { offset, kind }
    }

    /// The offset of the field in the bytes. For a broken limit, it is the
    /// offset of the definition.
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// What is wrong.
    #[must_use]
    pub fn kind(&self) -> &ReadErrorKind {
        &self.kind
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at byte {}: ", self.offset)?;
        match &self.kind {
            ReadErrorKind::UnexpectedEnd { expected } => {
                write!(f, "the bytes end before {expected}")
            }
            ReadErrorKind::TrailingBytes => {
                write!(f, "the bytes go on after the last definition")
            }
            ReadErrorKind::BadMagic => write!(f, "the bytes do not start with SCgf"),
            ReadErrorKind::UnsupportedVersion(version) => {
                write!(f, "version {version} is not 0, 1 or 2")
            }
            ReadErrorKind::NegativeCount(count) => write!(f, "the count {count} is negative"),
            ReadErrorKind::NegativeIndex(index) => write!(f, "the index {index} is negative"),
            ReadErrorKind::UnknownRate(rate) => write!(f, "the rate {rate} is not 0 to 3"),
            ReadErrorKind::NotUtf8 => write!(f, "a name is not UTF-8"),
            ReadErrorKind::Limit(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ReadError {}

/// Why a definition breaks a limit of scsynth or an invariant of the format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LimitError {
    definition: String,
    kind: LimitErrorKind,
}

/// The limit that a definition breaks.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LimitErrorKind {
    /// A name is longer than scsynth reads.
    #[non_exhaustive]
    NameTooLong {
        /// What the name names.
        what: NameKind,
        /// The name.
        name: String,
        /// The most bytes that the name can have.
        max: usize,
    },
    /// A count does not fit the integer that the version stores it in.
    #[non_exhaustive]
    TooMany {
        /// What the count counts.
        what: CountKind,
        /// The count.
        count: usize,
        /// The highest count that the version stores.
        max: usize,
    },
    /// An input reads a UGen at or after the UGen that reads it.
    #[non_exhaustive]
    LaterInput {
        /// The position of the reading UGen.
        ugen: usize,
        /// The position of the input.
        input: usize,
        /// The position of the UGen that it reads.
        source: u32,
    },
    /// An input reads an output that its UGen does not have.
    #[non_exhaustive]
    NoSuchOutput {
        /// The position of the reading UGen.
        ugen: usize,
        /// The position of the input.
        input: usize,
        /// The position of the UGen that it reads.
        source: u32,
        /// The output that it reads.
        output: u32,
    },
    /// An input reads a constant that the definition does not have.
    #[non_exhaustive]
    NoSuchConstant {
        /// The position of the reading UGen.
        ugen: usize,
        /// The position of the input.
        input: usize,
        /// The index of the constant.
        constant: u32,
    },
    /// A parameter name points past the parameters.
    #[non_exhaustive]
    NoSuchParameter {
        /// The parameter name.
        name: String,
        /// The index that it points at.
        index: u32,
    },
    /// A variant holds another number of values than the definition has
    /// parameters.
    #[non_exhaustive]
    VariantValues {
        /// The name of the variant.
        variant: String,
        /// The number of its values.
        values: usize,
    },
}

/// What a name names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NameKind {
    /// A synth definition.
    Definition,
    /// A UGen.
    UGen,
    /// A parameter.
    Parameter,
    /// A variant.
    Variant,
}

/// What a count counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CountKind {
    /// The definitions of a file.
    Definitions,
    /// The constants of a definition.
    Constants,
    /// The parameters of a definition.
    Parameters,
    /// The parameter names of a definition.
    ParameterNames,
    /// The UGens of a definition.
    UGens,
    /// The inputs of a UGen.
    Inputs,
    /// The outputs of a UGen.
    Outputs,
    /// The variants of a definition.
    Variants,
}

impl LimitError {
    pub(crate) fn new(definition: String, kind: LimitErrorKind) -> Self {
        Self { definition, kind }
    }

    /// The name of the definition. It is empty when the limit is one of the
    /// file.
    #[must_use]
    pub fn definition(&self) -> &str {
        &self.definition
    }

    /// The limit that the definition breaks.
    #[must_use]
    pub fn kind(&self) -> &LimitErrorKind {
        &self.kind
    }
}

impl fmt::Display for LimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.definition.is_empty() {
            write!(f, "the file: ")?;
        } else {
            write!(f, "synth definition '{}': ", self.definition)?;
        }
        match &self.kind {
            LimitErrorKind::NameTooLong { what, name, max } => write!(
                f,
                "the {} name '{name}' has {} bytes, more than {max}",
                what.noun(),
                name.len()
            ),
            LimitErrorKind::TooMany { what, count, max } => {
                write!(f, "{count} {}, more than {max}", what.noun())
            }
            LimitErrorKind::LaterInput {
                ugen,
                input,
                source,
            } => write!(
                f,
                "input {input} of UGen {ugen} reads UGen {source}, which is not before it"
            ),
            LimitErrorKind::NoSuchOutput {
                ugen,
                input,
                source,
                output,
            } => write!(
                f,
                "input {input} of UGen {ugen} reads output {output} of UGen {source}, which has no such output"
            ),
            LimitErrorKind::NoSuchConstant {
                ugen,
                input,
                constant,
            } => write!(
                f,
                "input {input} of UGen {ugen} reads constant {constant}, which does not exist"
            ),
            LimitErrorKind::NoSuchParameter { name, index } => write!(
                f,
                "the parameter name '{name}' points at {index}, past the parameters"
            ),
            LimitErrorKind::VariantValues { variant, values } => write!(
                f,
                "the variant '{variant}' has {values} values, not one per parameter"
            ),
        }
    }
}

impl std::error::Error for LimitError {}

impl NameKind {
    fn noun(self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::UGen => "UGen",
            Self::Parameter => "parameter",
            Self::Variant => "variant",
        }
    }
}

impl CountKind {
    fn noun(self) -> &'static str {
        match self {
            Self::Definitions => "definitions",
            Self::Constants => "constants",
            Self::Parameters => "parameters",
            Self::ParameterNames => "parameter names",
            Self::UGens => "UGens",
            Self::Inputs => "inputs of a UGen",
            Self::Outputs => "outputs of a UGen",
            Self::Variants => "variants",
        }
    }
}
