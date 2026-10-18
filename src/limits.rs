//! The limits of scsynth and the invariants of the format, which every
//! definition that the crate reads or writes meets.

use crate::error::{CountKind, LimitError, LimitErrorKind, NameKind};
use crate::file::Version;
use crate::synth_def::{InputSpec, SynthDef};

/// The longest definition name, in bytes: the length of a pstring is a u8.
pub(crate) const DEFINITION_NAME_MAX: usize = 255;

/// The longest UGen, parameter or variant name that scsynth reads, in bytes:
/// `kSCNameByteLen` minus 1.
pub(crate) const NAME_MAX: usize = 31;

/// `count` definitions fit one file, whose count of definitions is an int16
/// in every version.
pub(crate) fn check_definitions(count: usize) -> Result<(), LimitError> {
    fits(count, int16_max(), CountKind::Definitions)
        .map_err(|kind| LimitError::new(String::new(), kind))
}

/// `def` meets every limit of scsynth for a file of `version`.
pub(crate) fn check(def: &SynthDef, version: Version) -> Result<(), LimitError> {
    check_definition(def, version).map_err(|kind| LimitError::new(def.name.clone(), kind))
}

fn check_definition(def: &SynthDef, version: Version) -> Result<(), LimitErrorKind> {
    let max = if version == Version::V2 {
        int32_max()
    } else {
        int16_max()
    };
    name(&def.name, DEFINITION_NAME_MAX, NameKind::Definition)?;
    fits(def.constants.len(), max, CountKind::Constants)?;
    fits(def.controls.len(), max, CountKind::Parameters)?;
    fits(def.control_names.len(), max, CountKind::ParameterNames)?;
    fits(def.children.len(), max, CountKind::UGens)?;
    fits(def.variants.len(), int16_max(), CountKind::Variants)?;
    for control_name in &def.control_names {
        name(&control_name.name, NAME_MAX, NameKind::Parameter)?;
        if to_usize(control_name.index) > def.controls.len() {
            return Err(LimitErrorKind::NoSuchParameter {
                name: control_name.name.clone(),
                index: control_name.index,
            });
        }
    }
    for (ugen, child) in def.children.iter().enumerate() {
        name(&child.name, NAME_MAX, NameKind::UGen)?;
        fits(child.inputs.len(), max, CountKind::Inputs)?;
        fits(child.outputs.len(), max, CountKind::Outputs)?;
        for (input, spec) in child.inputs.iter().enumerate() {
            match *spec {
                InputSpec::Constant(constant) => {
                    if to_usize(constant) >= def.constants.len() {
                        return Err(LimitErrorKind::NoSuchConstant {
                            ugen,
                            input,
                            constant,
                        });
                    }
                }
                InputSpec::UGen {
                    index: source,
                    output,
                } => {
                    if to_usize(source) >= ugen {
                        return Err(LimitErrorKind::LaterInput {
                            ugen,
                            input,
                            source,
                        });
                    }
                    if to_usize(output) >= def.children[to_usize(source)].outputs.len() {
                        return Err(LimitErrorKind::NoSuchOutput {
                            ugen,
                            input,
                            source,
                            output,
                        });
                    }
                }
            }
        }
    }
    for variant in &def.variants {
        name(&variant.name, NAME_MAX, NameKind::Variant)?;
        if variant.values.len() != def.controls.len() {
            return Err(LimitErrorKind::VariantValues {
                variant: variant.name.clone(),
                values: variant.values.len(),
            });
        }
    }
    Ok(())
}

fn name(name: &str, max: usize, what: NameKind) -> Result<(), LimitErrorKind> {
    if name.len() > max {
        return Err(LimitErrorKind::NameTooLong {
            what,
            name: name.to_owned(),
            max,
        });
    }
    Ok(())
}

fn fits(count: usize, max: usize, what: CountKind) -> Result<(), LimitErrorKind> {
    if count > max {
        return Err(LimitErrorKind::TooMany { what, count, max });
    }
    Ok(())
}

fn to_usize(index: u32) -> usize {
    usize::try_from(index).unwrap_or(usize::MAX)
}

fn int16_max() -> usize {
    usize::try_from(i16::MAX).unwrap_or(usize::MAX)
}

fn int32_max() -> usize {
    usize::try_from(i32::MAX).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::{check, check_definitions};
    use crate::error::{CountKind, LimitErrorKind, NameKind};
    use crate::file::Version;
    use crate::rate::Rate;
    use crate::synth_def::{ControlName, InputSpec, SynthDef, UGen, Variant};

    /// `DC.kr(0.5)` into `Out.kr(0, ...)`, with one parameter and a variant.
    fn definition() -> SynthDef {
        SynthDef {
            name: "limits".to_owned(),
            constants: vec![0.5, 0.0],
            controls: vec![1.0],
            control_names: vec![ControlName {
                name: "level".to_owned(),
                index: 0,
            }],
            children: vec![
                UGen {
                    name: "DC".to_owned(),
                    rate: Rate::Control,
                    inputs: vec![InputSpec::Constant(0)],
                    outputs: vec![Rate::Control],
                    special_index: 0,
                },
                UGen {
                    name: "Out".to_owned(),
                    rate: Rate::Control,
                    inputs: vec![
                        InputSpec::Constant(1),
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
                name: "limits.low".to_owned(),
                values: vec![0.5],
            }],
        }
    }

    fn kind(def: &SynthDef, version: Version) -> Option<LimitErrorKind> {
        check(def, version).err().map(|error| error.kind().clone())
    }

    #[test]
    fn a_valid_definition_meets_the_limits_of_both_versions() {
        assert_eq!(kind(&definition(), Version::V2), None);
        assert_eq!(kind(&definition(), Version::V1), None);
    }

    #[test]
    fn a_definition_name_takes_255_bytes() {
        let mut def = definition();
        def.name = "d".repeat(255);
        assert_eq!(kind(&def, Version::V2), None);
        def.name.push('d');
        assert!(matches!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NameTooLong {
                what: NameKind::Definition,
                max: 255,
                ..
            })
        ));
    }

    #[test]
    fn a_ugen_name_takes_31_bytes() {
        let mut def = definition();
        def.children[0].name = "u".repeat(31);
        assert_eq!(kind(&def, Version::V2), None);
        def.children[0].name.push('u');
        assert!(matches!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NameTooLong {
                what: NameKind::UGen,
                max: 31,
                ..
            })
        ));
    }

    #[test]
    fn a_parameter_name_takes_31_bytes() {
        let mut def = definition();
        def.control_names[0].name = "p".repeat(31);
        assert_eq!(kind(&def, Version::V2), None);
        def.control_names[0].name.push('p');
        assert!(matches!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NameTooLong {
                what: NameKind::Parameter,
                ..
            })
        ));
    }

    #[test]
    fn a_variant_name_takes_31_bytes_with_its_definition_name() {
        let mut def = definition();
        def.variants[0].name = format!("limits.{}", "v".repeat(24));
        assert_eq!(kind(&def, Version::V2), None);
        def.variants[0].name.push('v');
        assert!(matches!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NameTooLong {
                what: NameKind::Variant,
                ..
            })
        ));
    }

    #[test]
    fn an_input_reads_only_an_earlier_ugen() {
        let mut def = definition();
        def.children[1].inputs[1] = InputSpec::UGen {
            index: 1,
            output: 0,
        };
        assert_eq!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::LaterInput {
                ugen: 1,
                input: 1,
                source: 1
            })
        );
        def.children[0].inputs[0] = InputSpec::UGen {
            index: 1,
            output: 0,
        };
        assert_eq!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::LaterInput {
                ugen: 0,
                input: 0,
                source: 1
            })
        );
    }

    #[test]
    fn an_input_reads_only_an_output_of_its_ugen() {
        let mut def = definition();
        def.children[1].inputs[1] = InputSpec::UGen {
            index: 0,
            output: 1,
        };
        assert_eq!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NoSuchOutput {
                ugen: 1,
                input: 1,
                source: 0,
                output: 1
            })
        );
    }

    #[test]
    fn an_input_reads_only_a_constant_of_the_definition() {
        let mut def = definition();
        def.children[1].inputs[0] = InputSpec::Constant(2);
        assert_eq!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NoSuchConstant {
                ugen: 1,
                input: 0,
                constant: 2
            })
        );
    }

    #[test]
    fn a_parameter_name_points_at_most_at_the_count_of_parameters() {
        let mut def = definition();
        def.control_names.push(ControlName {
            name: "empty".to_owned(),
            index: 1,
        });
        assert_eq!(kind(&def, Version::V2), None);
        def.control_names[1].index = 2;
        assert_eq!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::NoSuchParameter {
                name: "empty".to_owned(),
                index: 2
            })
        );
    }

    #[test]
    fn a_variant_has_one_value_per_parameter() {
        let mut def = definition();
        def.variants[0].values.push(0.25);
        assert_eq!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::VariantValues {
                variant: "limits.low".to_owned(),
                values: 2
            })
        );
    }

    #[test]
    fn version_1_holds_counts_up_to_i16_max() {
        let mut def = definition();
        def.constants.resize(32767, 0.0);
        assert_eq!(kind(&def, Version::V1), None);
        def.constants.push(0.0);
        assert_eq!(
            kind(&def, Version::V1),
            Some(LimitErrorKind::TooMany {
                what: CountKind::Constants,
                count: 32768,
                max: 32767
            })
        );
        assert_eq!(kind(&def, Version::V2), None);
    }

    #[test]
    fn the_variants_and_the_definitions_count_up_to_i16_max_in_version_2() {
        let mut def = definition();
        let variant = def.variants[0].clone();
        def.variants.resize(32768, variant);
        assert!(matches!(
            kind(&def, Version::V2),
            Some(LimitErrorKind::TooMany {
                what: CountKind::Variants,
                ..
            })
        ));
        assert_eq!(check_definitions(32767), Ok(()));
        let error = check_definitions(32768).expect_err("32768 definitions do not fit");
        assert_eq!(error.definition(), "");
        assert!(matches!(
            error.kind(),
            LimitErrorKind::TooMany {
                what: CountKind::Definitions,
                ..
            }
        ));
    }
}
