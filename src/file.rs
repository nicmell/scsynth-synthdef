//! Definition files: their versions, how they read, and how they write.

use crate::error::{LimitError, ReadError, ReadErrorKind};
use crate::limits;
use crate::rate::Rate;
use crate::synth_def::{ControlName, InputSpec, SynthDef, UGen, Variant};

/// The version of a definition file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Version {
    /// Version 0: the layout of version 1, without variants.
    V0,
    /// Version 1: counts and indices as int16. `SynthDefOld` writes it.
    V1,
    /// Version 2: counts and indices as int32. `SynthDef` writes it.
    V2,
}

impl Version {
    /// The number that the file stores for the version.
    #[must_use]
    pub const fn number(self) -> i32 {
        match self {
            Self::V0 => 0,
            Self::V1 => 1,
            Self::V2 => 2,
        }
    }

    fn wide(self) -> bool {
        self == Self::V2
    }
}

/// The content of a definition file.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefFile {
    /// The version of the file.
    pub version: Version,
    /// The definitions, in file order.
    pub synth_defs: Vec<SynthDef>,
}

const MAGIC: &[u8; 4] = b"SCgf";

/// Reads a definition file of version 0, 1 or 2, with all its definitions.
///
/// # Errors
///
/// A [`ReadError`] when the bytes are not a definition file, or when one of
/// its definitions breaks a limit of scsynth.
pub fn read_def_file(bytes: &[u8]) -> Result<DefFile, ReadError> {
    let mut reader = Reader { bytes, offset: 0 };
    if reader.take(MAGIC.len(), "the magic SCgf")? != MAGIC {
        return Err(ReadError::new(0, ReadErrorKind::BadMagic));
    }
    let at = reader.offset;
    let version = match reader.i32("the version")? {
        0 => Version::V0,
        1 => Version::V1,
        2 => Version::V2,
        number => {
            return Err(ReadError::new(
                at,
                ReadErrorKind::UnsupportedVersion(number),
            ));
        }
    };
    let count = reader.count(false, "the count of definitions")?;
    let mut synth_defs = Vec::with_capacity(reader.capacity(count, 1));
    for _ in 0..count {
        let at = reader.offset;
        let def = read_definition(&mut reader, version)?;
        limits::check(&def, version)
            .map_err(|error| ReadError::new(at, ReadErrorKind::Limit(error)))?;
        synth_defs.push(def);
    }
    if reader.offset != bytes.len() {
        return Err(ReadError::new(reader.offset, ReadErrorKind::TrailingBytes));
    }
    Ok(DefFile {
        version,
        synth_defs,
    })
}

/// Writes definitions as a file of version 2, as `Collection:writeDef` does.
///
/// # Errors
///
/// A [`LimitError`] when a definition breaks a limit of scsynth, or when the
/// definitions are too many for one file.
pub fn write_def_file(defs: &[SynthDef]) -> Result<Vec<u8>, LimitError> {
    write(defs, Version::V2)
}

/// Writes definitions as a file of version 1, as `SynthDefOld` does.
///
/// # Errors
///
/// A [`LimitError`] when a definition breaks a limit of scsynth, or when the
/// definitions are too many for one file.
pub fn write_def_file_old(defs: &[SynthDef]) -> Result<Vec<u8>, LimitError> {
    write(defs, Version::V1)
}

fn read_definition(reader: &mut Reader<'_>, version: Version) -> Result<SynthDef, ReadError> {
    let wide = version.wide();
    let name = reader.pstring("the name of a definition")?;
    let count = reader.count(wide, "the count of constants")?;
    let constants = reader.floats(count, "a constant")?;
    let count = reader.count(wide, "the count of parameters")?;
    let controls = reader.floats(count, "the value of a parameter")?;
    let count = reader.count(wide, "the count of parameter names")?;
    let mut control_names = Vec::with_capacity(reader.capacity(count, 2));
    for _ in 0..count {
        let name = reader.pstring("a parameter name")?;
        let index = reader.index(wide, "the index of a parameter name")?;
        control_names.push(ControlName { name, index });
    }
    let count = reader.count(wide, "the count of UGens")?;
    let mut children = Vec::with_capacity(reader.capacity(count, 8));
    for _ in 0..count {
        children.push(read_ugen(reader, wide)?);
    }
    let mut variants = Vec::new();
    if version != Version::V0 {
        let count = reader.count(false, "the count of variants")?;
        variants.reserve(reader.capacity(count, 1));
        for _ in 0..count {
            let name = reader.pstring("a variant name")?;
            let values = reader.floats(controls.len(), "the value of a variant")?;
            variants.push(Variant { name, values });
        }
    }
    Ok(SynthDef {
        name,
        constants,
        controls,
        control_names,
        children,
        variants,
    })
}

fn read_ugen(reader: &mut Reader<'_>, wide: bool) -> Result<UGen, ReadError> {
    let name = reader.pstring("the name of a UGen")?;
    let rate = reader.rate("the rate of a UGen")?;
    let inputs = reader.count(wide, "the count of inputs")?;
    let outputs = reader.count(wide, "the count of outputs")?;
    let special_index = reader.i16("the special index")?;
    let mut input_specs = Vec::with_capacity(reader.capacity(inputs, 4));
    for _ in 0..inputs {
        input_specs.push(read_input(reader, wide)?);
    }
    let mut output_rates = Vec::with_capacity(reader.capacity(outputs, 1));
    for _ in 0..outputs {
        output_rates.push(reader.rate("the rate of an output")?);
    }
    Ok(UGen {
        name,
        rate,
        inputs: input_specs,
        outputs: output_rates,
        special_index,
    })
}

fn read_input(reader: &mut Reader<'_>, wide: bool) -> Result<InputSpec, ReadError> {
    let at = reader.offset;
    let source = reader.int(wide, "the source of an input")?;
    if source == -1 {
        let constant = reader.index(wide, "the index of a constant")?;
        return Ok(InputSpec::Constant(constant));
    }
    let index = u32::try_from(source)
        .map_err(|_| ReadError::new(at, ReadErrorKind::NegativeIndex(source)))?;
    let output = reader.index(wide, "the output of an input")?;
    Ok(InputSpec::UGen { index, output })
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize, expected: &'static str) -> Result<&'a [u8], ReadError> {
        let rest = &self.bytes[self.offset..];
        if rest.len() < len {
            return Err(ReadError::new(
                self.offset,
                ReadErrorKind::UnexpectedEnd { expected },
            ));
        }
        self.offset += len;
        Ok(&rest[..len])
    }

    fn array<const N: usize>(&mut self, expected: &'static str) -> Result<[u8; N], ReadError> {
        let mut array = [0; N];
        array.copy_from_slice(self.take(N, expected)?);
        Ok(array)
    }

    /// How many elements of at least `size` bytes the rest of the bytes can
    /// hold, at most `count`. A count from the file does not size an
    /// allocation by itself.
    fn capacity(&self, count: usize, size: usize) -> usize {
        count.min((self.bytes.len() - self.offset) / size)
    }

    fn i8(&mut self, expected: &'static str) -> Result<i8, ReadError> {
        self.array(expected).map(i8::from_be_bytes)
    }

    fn i16(&mut self, expected: &'static str) -> Result<i16, ReadError> {
        self.array(expected).map(i16::from_be_bytes)
    }

    fn i32(&mut self, expected: &'static str) -> Result<i32, ReadError> {
        self.array(expected).map(i32::from_be_bytes)
    }

    fn int(&mut self, wide: bool, expected: &'static str) -> Result<i32, ReadError> {
        if wide {
            self.i32(expected)
        } else {
            self.i16(expected).map(i32::from)
        }
    }

    fn count(&mut self, wide: bool, expected: &'static str) -> Result<usize, ReadError> {
        let at = self.offset;
        let count = self.int(wide, expected)?;
        usize::try_from(count).map_err(|_| ReadError::new(at, ReadErrorKind::NegativeCount(count)))
    }

    fn index(&mut self, wide: bool, expected: &'static str) -> Result<u32, ReadError> {
        let at = self.offset;
        let index = self.int(wide, expected)?;
        u32::try_from(index).map_err(|_| ReadError::new(at, ReadErrorKind::NegativeIndex(index)))
    }

    fn floats(&mut self, count: usize, expected: &'static str) -> Result<Vec<f32>, ReadError> {
        let mut floats = Vec::with_capacity(self.capacity(count, 4));
        for _ in 0..count {
            floats.push(self.array(expected).map(f32::from_be_bytes)?);
        }
        Ok(floats)
    }

    fn pstring(&mut self, expected: &'static str) -> Result<String, ReadError> {
        let at = self.offset;
        let [len] = self.array(expected)?;
        let bytes = self.take(usize::from(len), expected)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ReadError::new(at, ReadErrorKind::NotUtf8))
    }

    fn rate(&mut self, expected: &'static str) -> Result<Rate, ReadError> {
        let at = self.offset;
        let number = self.i8(expected)?;
        Rate::from_rate_number(number)
            .ok_or_else(|| ReadError::new(at, ReadErrorKind::UnknownRate(number)))
    }
}

fn write(defs: &[SynthDef], version: Version) -> Result<Vec<u8>, LimitError> {
    limits::check_definitions(defs.len())?;
    for def in defs {
        limits::check(def, version)?;
    }
    let mut writer = Writer {
        bytes: Vec::new(),
        wide: version.wide(),
    };
    writer.bytes.extend_from_slice(MAGIC);
    writer
        .bytes
        .extend_from_slice(&version.number().to_be_bytes());
    writer.short(defs.len());
    for def in defs {
        writer.definition(def);
    }
    Ok(writer.bytes)
}

/// Writes definitions that meet the limits, so every count and index fits
/// its integer.
struct Writer {
    bytes: Vec<u8>,
    wide: bool,
}

impl Writer {
    fn definition(&mut self, def: &SynthDef) {
        self.pstring(&def.name);
        self.int(def.constants.len());
        self.floats(&def.constants);
        self.int(def.controls.len());
        self.floats(&def.controls);
        self.int(def.control_names.len());
        for control_name in &def.control_names {
            self.pstring(&control_name.name);
            self.index(control_name.index);
        }
        self.int(def.children.len());
        for ugen in &def.children {
            self.ugen(ugen);
        }
        self.short(def.variants.len());
        for variant in &def.variants {
            self.pstring(&variant.name);
            self.floats(&variant.values);
        }
    }

    fn ugen(&mut self, ugen: &UGen) {
        self.pstring(&ugen.name);
        self.bytes
            .extend_from_slice(&ugen.rate.rate_number().to_be_bytes());
        self.int(ugen.inputs.len());
        self.int(ugen.outputs.len());
        self.bytes
            .extend_from_slice(&ugen.special_index.to_be_bytes());
        for input in &ugen.inputs {
            match *input {
                InputSpec::Constant(constant) => {
                    self.signed(-1);
                    self.index(constant);
                }
                InputSpec::UGen { index, output } => {
                    self.index(index);
                    self.index(output);
                }
            }
        }
        for rate in &ugen.outputs {
            self.bytes
                .extend_from_slice(&rate.rate_number().to_be_bytes());
        }
    }

    fn signed(&mut self, value: i32) {
        if self.wide {
            self.bytes.extend_from_slice(&value.to_be_bytes());
        } else {
            let value = i16::try_from(value).expect("the limits keep the value in an int16");
            self.bytes.extend_from_slice(&value.to_be_bytes());
        }
    }

    fn int(&mut self, value: usize) {
        self.signed(i32::try_from(value).expect("the limits keep the count in an int32"));
    }

    fn index(&mut self, value: u32) {
        self.signed(i32::try_from(value).expect("the limits keep the index in an int32"));
    }

    fn short(&mut self, value: usize) {
        let value = i16::try_from(value).expect("the limits keep the count in an int16");
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn floats(&mut self, floats: &[f32]) {
        for float in floats {
            self.bytes.extend_from_slice(&float.to_be_bytes());
        }
    }

    fn pstring(&mut self, string: &str) {
        let len = u8::try_from(string.len()).expect("the limits keep a name in 255 bytes");
        self.bytes.push(len);
        self.bytes.extend_from_slice(string.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::{Version, read_def_file, write_def_file, write_def_file_old};
    use crate::error::{LimitErrorKind, ReadErrorKind};
    use crate::rate::Rate;
    use crate::synth_def::{ControlName, InputSpec, SynthDef, UGen, Variant};

    /// `Control.kr` into `Out.kr(0, ...)`, with the parameter `x` and the
    /// variant `a.v`.
    fn definition() -> SynthDef {
        SynthDef {
            name: "a".to_owned(),
            constants: vec![0.0, 1.0],
            controls: vec![0.5],
            control_names: vec![ControlName {
                name: "x".to_owned(),
                index: 0,
            }],
            children: vec![
                UGen {
                    name: "Control".to_owned(),
                    rate: Rate::Control,
                    inputs: vec![],
                    outputs: vec![Rate::Control],
                    special_index: 0,
                },
                UGen {
                    name: "Out".to_owned(),
                    rate: Rate::Control,
                    inputs: vec![
                        InputSpec::Constant(0),
                        InputSpec::UGen {
                            index: 0,
                            output: 0,
                        },
                    ],
                    outputs: vec![],
                    special_index: 0,
                },
            ],
            variants: vec![Variant {
                name: "a.v".to_owned(),
                values: vec![0.25],
            }],
        }
    }

    /// The bytes of [`definition`] in version 2, field by field.
    const VERSION_2: &[u8] = &[
        b'S', b'C', b'g', b'f', 0, 0, 0, 2, 0, 1, // magic, version, 1 definition
        1, b'a', // name
        0, 0, 0, 2, 0, 0, 0, 0, 0x3f, 0x80, 0, 0, // constants 0.0 and 1.0
        0, 0, 0, 1, 0x3f, 0, 0, 0, // parameter 0.5
        0, 0, 0, 1, 1, b'x', 0, 0, 0, 0, // name x at 0
        0, 0, 0, 2, // 2 UGens
        7, b'C', b'o', b'n', b't', b'r', b'o', b'l', // Control
        1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, // control rate, no input, 1 output
        3, b'O', b'u', b't', 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, // Out, 2 inputs
        0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, // constant 0
        0, 0, 0, 0, 0, 0, 0, 0, // UGen 0, output 0
        0, 1, 3, b'a', b'.', b'v', 0x3e, 0x80, 0, 0, // variant a.v, 0.25
    ];

    /// The bytes of [`definition`] in version 1: int16 counts and indices.
    const VERSION_1: &[u8] = &[
        b'S', b'C', b'g', b'f', 0, 0, 0, 1, 0, 1, // magic, version, 1 definition
        1, b'a', // name
        0, 2, 0, 0, 0, 0, 0x3f, 0x80, 0, 0, // constants 0.0 and 1.0
        0, 1, 0x3f, 0, 0, 0, // parameter 0.5
        0, 1, 1, b'x', 0, 0, // name x at 0
        0, 2, // 2 UGens
        7, b'C', b'o', b'n', b't', b'r', b'o', b'l', // Control
        1, 0, 0, 0, 1, 0, 0, 1, // control rate, no input, 1 output
        3, b'O', b'u', b't', 1, 0, 2, 0, 0, 0, 0, // Out, 2 inputs
        0xff, 0xff, 0, 0, // constant 0
        0, 0, 0, 0, // UGen 0, output 0
        0, 1, 3, b'a', b'.', b'v', 0x3e, 0x80, 0, 0, // variant a.v, 0.25
    ];

    fn error(bytes: &[u8]) -> (usize, ReadErrorKind) {
        let error = read_def_file(bytes).expect_err("the bytes do not read");
        (error.offset(), error.kind().clone())
    }

    #[test]
    fn version_2_writes_int32_counts_and_indices() {
        assert_eq!(write_def_file(&[definition()]).unwrap(), VERSION_2);
        assert_eq!(definition().as_bytes().unwrap(), VERSION_2);
        let file = read_def_file(VERSION_2).unwrap();
        assert_eq!(file.version, Version::V2);
        assert_eq!(file.synth_defs, [definition()]);
    }

    #[test]
    fn version_1_writes_int16_counts_and_indices() {
        assert_eq!(write_def_file_old(&[definition()]).unwrap(), VERSION_1);
        let file = read_def_file(VERSION_1).unwrap();
        assert_eq!(file.version, Version::V1);
        assert_eq!(file.synth_defs, [definition()]);
    }

    #[test]
    fn version_0_is_version_1_without_variants() {
        let mut bytes = VERSION_1[..VERSION_1.len() - 10].to_vec();
        bytes[7] = 0;
        let file = read_def_file(&bytes).unwrap();
        assert_eq!(file.version, Version::V0);
        let mut def = definition();
        def.variants.clear();
        assert_eq!(file.synth_defs, [def]);

        bytes[7] = 1;
        assert_eq!(
            error(&bytes),
            (
                bytes.len(),
                ReadErrorKind::UnexpectedEnd {
                    expected: "the count of variants"
                }
            )
        );
        let mut bytes = VERSION_1.to_vec();
        bytes[7] = 0;
        assert_eq!(
            error(&bytes),
            (VERSION_1.len() - 10, ReadErrorKind::TrailingBytes)
        );
    }

    #[test]
    fn several_definitions_share_a_file() {
        let mut second = definition();
        second.name = "b".to_owned();
        second.variants[0].name = "b.v".to_owned();
        let defs = [definition(), second];
        for (bytes, version) in [
            (write_def_file(&defs).unwrap(), Version::V2),
            (write_def_file_old(&defs).unwrap(), Version::V1),
        ] {
            let file = read_def_file(&bytes).unwrap();
            assert_eq!(file.version, version);
            assert_eq!(file.synth_defs, defs);
        }
    }

    #[test]
    fn every_truncation_is_refused() {
        for len in 0..VERSION_2.len() {
            let (offset, kind) = error(&VERSION_2[..len]);
            assert!(
                matches!(kind, ReadErrorKind::UnexpectedEnd { .. }),
                "{len}: {kind:?}"
            );
            assert!(offset <= len, "{len}: {offset}");
        }
    }

    #[test]
    fn trailing_bytes_are_refused() {
        let mut bytes = VERSION_2.to_vec();
        bytes.push(0);
        assert_eq!(
            error(&bytes),
            (VERSION_2.len(), ReadErrorKind::TrailingBytes)
        );
    }

    #[test]
    fn a_wrong_magic_is_refused() {
        let mut bytes = VERSION_2.to_vec();
        bytes[3] = b'g';
        assert_eq!(error(&bytes), (0, ReadErrorKind::BadMagic));
    }

    #[test]
    fn another_version_is_refused() {
        for version in [3, -1] {
            let mut bytes = VERSION_2.to_vec();
            bytes[4..8].copy_from_slice(&i32::to_be_bytes(version));
            assert_eq!(
                error(&bytes),
                (4, ReadErrorKind::UnsupportedVersion(version))
            );
        }
    }

    #[test]
    fn a_negative_count_is_refused() {
        let mut bytes = VERSION_2.to_vec();
        bytes[12..16].copy_from_slice(&i32::to_be_bytes(-2));
        assert_eq!(error(&bytes), (12, ReadErrorKind::NegativeCount(-2)));
        let mut bytes = VERSION_2.to_vec();
        bytes[8..10].copy_from_slice(&i16::to_be_bytes(-1));
        assert_eq!(error(&bytes), (8, ReadErrorKind::NegativeCount(-1)));
    }

    #[test]
    fn a_negative_index_other_than_the_constant_mark_is_refused() {
        let input = VERSION_2.len() - 26;
        let mut bytes = VERSION_2.to_vec();
        bytes[input..input + 4].copy_from_slice(&i32::to_be_bytes(-2));
        assert_eq!(error(&bytes), (input, ReadErrorKind::NegativeIndex(-2)));
        let mut bytes = VERSION_2.to_vec();
        bytes[input + 4..input + 8].copy_from_slice(&i32::to_be_bytes(-1));
        assert_eq!(error(&bytes), (input + 4, ReadErrorKind::NegativeIndex(-1)));
        let mut bytes = VERSION_2.to_vec();
        bytes[input + 12..input + 16].copy_from_slice(&i32::to_be_bytes(-1));
        assert_eq!(
            error(&bytes),
            (input + 12, ReadErrorKind::NegativeIndex(-1))
        );
        let name_index = 38;
        let mut bytes = VERSION_2.to_vec();
        bytes[name_index..name_index + 4].copy_from_slice(&i32::to_be_bytes(-1));
        assert_eq!(
            error(&bytes),
            (name_index, ReadErrorKind::NegativeIndex(-1))
        );
    }

    #[test]
    fn an_unknown_rate_is_refused() {
        let ugen_rate = 54;
        let mut bytes = VERSION_2.to_vec();
        bytes[ugen_rate] = 4;
        assert_eq!(error(&bytes), (ugen_rate, ReadErrorKind::UnknownRate(4)));
        let output_rate = 65;
        let mut bytes = VERSION_2.to_vec();
        bytes[output_rate] = 0xff;
        assert_eq!(error(&bytes), (output_rate, ReadErrorKind::UnknownRate(-1)));
    }

    #[test]
    fn a_name_that_is_not_utf_8_is_refused() {
        let mut bytes = VERSION_2.to_vec();
        bytes[11] = 0xff;
        assert_eq!(error(&bytes), (10, ReadErrorKind::NotUtf8));
    }

    #[test]
    fn reading_applies_the_limits() {
        let mut bytes = definition().as_bytes().unwrap();
        let output = bytes.len() - 14;
        bytes[output..output + 4].copy_from_slice(&i32::to_be_bytes(1));
        let (offset, kind) = error(&bytes);
        assert_eq!(offset, 10);
        let ReadErrorKind::Limit(limit) = kind else {
            panic!("{kind:?}");
        };
        assert_eq!(limit.definition(), "a");
        assert!(matches!(limit.kind(), LimitErrorKind::NoSuchOutput { .. }));
    }

    #[test]
    fn writing_applies_the_limits() {
        let mut def = definition();
        def.children[0].name = "u".repeat(32);
        assert!(def.as_bytes().is_err());
        assert!(write_def_file_old(&[def]).is_err());
    }
}
