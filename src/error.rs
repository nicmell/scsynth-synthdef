//! Why bytes are not a definition file, why a definition breaks a limit of
//! scsynth, and why a build fails.

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

/// Why a build fails.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildError {
    /// Two arguments have this name.
    DuplicateArgument(String),
    /// The argument has an empty array as its default.
    EmptyArray(String),
    /// The scalar, trigger or audio argument has a lag. sclang ignores the
    /// lag, with a warning.
    LagOnNonControl(String),
    /// A lag of the argument is NaN.
    NanLag(String),
    /// The parameters before a control UGen do not fit its special index,
    /// an int16.
    TooManyParameters(usize),
    /// An input is NaN, which `checkValidInputs` refuses.
    #[non_exhaustive]
    NotANumber {
        /// The class of the UGen.
        ugen: String,
        /// The position of the input.
        input: usize,
    },
    /// An input is an array, which needs multichannel expansion.
    #[non_exhaustive]
    MultichannelExpansion {
        /// The class of the UGen.
        ugen: String,
        /// The position of the input.
        input: usize,
    },
    /// An input is an output of another build.
    #[non_exhaustive]
    ForeignOutput {
        /// The class of the UGen.
        ugen: String,
        /// The position of the input.
        input: usize,
    },
    /// The name is not an operator of the kind that the call adds: unary
    /// for `unary_op`, binary for `binary_op`.
    UnknownOperator(String),
    /// Two variants have this full name.
    DuplicateVariant(String),
    /// A variant sets a control that the definition does not have.
    #[non_exhaustive]
    UnknownControl {
        /// The full name of the variant.
        variant: String,
        /// The name of the control.
        control: String,
    },
    /// A variant gives a control more values than the control has.
    #[non_exhaustive]
    VariantSize {
        /// The full name of the variant.
        variant: String,
        /// The name of the control.
        control: String,
        /// The number of values that the variant gives.
        values: usize,
        /// The number of values of the control.
        size: usize,
    },
    /// The catalogue has no class of this name.
    UnknownClass(String),
    /// The class has no rate method of this selector.
    #[non_exhaustive]
    UnknownMethod {
        /// The class.
        class: String,
        /// The selector.
        selector: String,
    },
    /// The catalogue marks the method as not addable by name.
    #[non_exhaustive]
    NotAddable {
        /// The class.
        class: String,
        /// The selector.
        selector: String,
        /// Why not.
        reason: String,
    },
    /// The method has no argument of this name.
    #[non_exhaustive]
    UnknownArgument {
        /// The class.
        class: String,
        /// The name of the argument.
        argument: String,
    },
    /// The call gives this argument twice.
    ArgumentTwice(String),
    /// The argument has no value: no default, or a default that sclang
    /// computes when the method runs.
    #[non_exhaustive]
    MissingArgument {
        /// The class.
        class: String,
        /// The name of the argument.
        argument: String,
    },
    /// The argument has a value of another kind than it takes.
    #[non_exhaustive]
    WrongKind {
        /// The class.
        class: String,
        /// The name of the argument.
        argument: String,
        /// What the argument takes.
        expected: &'static str,
    },
    /// `mul` is not 1, or `add` is not 0. The builder does not apply `madd`.
    MaddNotSupported(String),
    /// A count of channels is not a positive integer, or an array that sets
    /// the outputs is empty.
    #[non_exhaustive]
    BadCount {
        /// The class.
        class: String,
        /// The name of the argument.
        argument: String,
    },
    /// The inputs break the rule of `checkInputs` of the class.
    #[non_exhaustive]
    CheckInputs {
        /// The class.
        class: String,
        /// What is wrong.
        message: String,
    },
    /// The definition breaks a limit of scsynth.
    Limit(LimitError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateArgument(name) => write!(f, "two arguments are named '{name}'"),
            Self::EmptyArray(name) => write!(f, "the argument '{name}' has no value"),
            Self::LagOnNonControl(name) => write!(
                f,
                "the argument '{name}' has a lag, but it is not at control rate"
            ),
            Self::NanLag(name) => write!(f, "a lag of the argument '{name}' is NaN"),
            Self::TooManyParameters(count) => write!(
                f,
                "{count} parameters come before a control UGen, more than its special index holds"
            ),
            Self::NotANumber { ugen, input } => write!(f, "input {input} of {ugen} is NaN"),
            Self::MultichannelExpansion { ugen, input } => write!(
                f,
                "input {input} of {ugen} is an array, which needs multichannel expansion"
            ),
            Self::ForeignOutput { ugen, input } => {
                write!(f, "input {input} of {ugen} is an output of another build")
            }
            Self::UnknownOperator(name) => write!(f, "'{name}' is no operator of scsynth"),
            Self::DuplicateVariant(name) => write!(f, "two variants are named '{name}'"),
            Self::UnknownControl { variant, control } => write!(
                f,
                "the variant '{variant}' sets '{control}', which is not a control"
            ),
            Self::VariantSize {
                variant,
                control,
                values,
                size,
            } => write!(
                f,
                "the variant '{variant}' gives '{control}' {values} values, but it has {size}"
            ),
            Self::UnknownClass(class) => write!(f, "the catalogue has no class {class}"),
            Self::UnknownMethod { class, selector } => {
                write!(f, "{class} has no rate method {selector}")
            }
            Self::NotAddable {
                class,
                selector,
                reason,
            } => write!(f, "{class}.{selector} is not addable by name: {reason}"),
            Self::UnknownArgument { class, argument } => {
                write!(f, "{class} has no argument {argument}")
            }
            Self::ArgumentTwice(argument) => write!(f, "the argument {argument} comes twice"),
            Self::MissingArgument { class, argument } => {
                write!(f, "the argument {argument} of {class} needs a value")
            }
            Self::WrongKind {
                class,
                argument,
                expected,
            } => write!(f, "the argument {argument} of {class} takes {expected}"),
            Self::MaddNotSupported(argument) => write!(
                f,
                "the builder does not apply madd, so {argument} must keep its default"
            ),
            Self::BadCount { class, argument } => write!(
                f,
                "the argument {argument} of {class} must give at least one channel"
            ),
            Self::CheckInputs { class, message } => write!(f, "{class}: {message}"),
            Self::Limit(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for BuildError {}
