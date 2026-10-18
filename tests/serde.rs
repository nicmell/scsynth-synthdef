//! A definition through JSON and back, with the feature `serde`.
#![cfg(feature = "serde")]

use scsynth_synthdef::{ControlName, DefFile, InputSpec, Rate, SynthDef, UGen, Variant, Version};

fn definition() -> SynthDef {
    SynthDef {
        name: "json".to_owned(),
        constants: vec![0.0, 0.5],
        controls: vec![440.0],
        control_names: vec![ControlName {
            name: "freq".to_owned(),
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
                name: "SinOsc".to_owned(),
                rate: Rate::Audio,
                inputs: vec![
                    InputSpec::UGen {
                        index: 0,
                        output: 0,
                    },
                    InputSpec::Constant(0),
                ],
                outputs: vec![Rate::Audio],
                special_index: 0,
            },
            UGen {
                name: "Out".to_owned(),
                rate: Rate::Audio,
                inputs: vec![
                    InputSpec::Constant(0),
                    InputSpec::UGen {
                        index: 1,
                        output: 0,
                    },
                ],
                outputs: vec![],
                special_index: 0,
            },
        ],
        variants: vec![Variant {
            name: "json.low".to_owned(),
            values: vec![220.0],
        }],
    }
}

#[test]
fn a_definition_file_round_trips_through_json() {
    let file = DefFile {
        version: Version::V2,
        synth_defs: vec![definition()],
    };
    let json = serde_json::to_string(&file).unwrap();
    assert_eq!(serde_json::from_str::<DefFile>(&json).unwrap(), file);
}

#[test]
fn a_rate_is_a_lowercase_name() {
    assert_eq!(serde_json::to_string(&Rate::Demand).unwrap(), r#""demand""#);
}
