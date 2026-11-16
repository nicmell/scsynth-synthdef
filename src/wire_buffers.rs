//! The wire buffers of a definition, as `DoBufferColoring` of scsynth 3.14.1
//! counts them.

use crate::catalog;
use crate::error::WireBufferError;
use crate::file::Version;
use crate::limits::{self, to_usize};
use crate::rate::Rate;
use crate::synth_def::{InputSpec, SynthDef, UGen};

impl SynthDef {
    /// The number of wire buffers that scsynth 3.14.1 gives the definition:
    /// the most buffers of audio-rate outputs in use at one time, as
    /// [Wire buffers](crate#wire-buffers) tells.
    ///
    /// # Errors
    ///
    /// [`WireBufferError::Limit`] for a definition that breaks a limit of
    /// scsynth, and [`WireBufferError::UnknownUGen`] for a UGen whose class
    /// the catalogue does not hold.
    pub fn wire_buffers(&self) -> Result<usize, WireBufferError> {
        limits::check(self, Version::V2).map_err(WireBufferError::Limit)?;
        let mut readers: Vec<Vec<usize>> = self
            .children
            .iter()
            .map(|ugen| vec![0; ugen.outputs.len()])
            .collect();
        for ugen in &self.children {
            for (index, output) in read_outputs(ugen) {
                readers[index][output] += 1;
            }
        }
        let (mut in_use, mut most) = (0, 0);
        for (position, ugen) in self.children.iter().enumerate() {
            let class =
                catalog::lookup(&ugen.name).ok_or_else(|| WireBufferError::UnknownUGen {
                    ugen: position,
                    name: ugen.name.clone(),
                })?;
            let outputs = ugen
                .outputs
                .iter()
                .filter(|&&rate| rate == Rate::Audio)
                .count();
            if class.cant_alias {
                in_use += outputs;
                most = most.max(in_use);
            }
            if ugen.rate != Rate::Demand {
                for (index, output) in read_outputs(ugen) {
                    if self.children[index].outputs[output] == Rate::Audio {
                        readers[index][output] -= 1;
                        if readers[index][output] == 0 {
                            in_use -= 1;
                        }
                    }
                }
            }
            if !class.cant_alias {
                in_use += outputs;
                most = most.max(in_use);
            }
        }
        Ok(most)
    }
}

/// The UGen and the output that each input of `ugen` reads, for the inputs
/// that read a UGen.
fn read_outputs(ugen: &UGen) -> impl Iterator<Item = (usize, usize)> + '_ {
    ugen.inputs.iter().filter_map(|input| match *input {
        InputSpec::UGen { index, output } => Some((to_usize(index), to_usize(output))),
        InputSpec::Constant(_) => None,
    })
}

#[cfg(test)]
mod tests {
    use crate::error::{LimitErrorKind, WireBufferError};
    use crate::rate::Rate;
    use crate::synth_def::{InputSpec, SynthDef, UGen};

    fn def(children: Vec<UGen>) -> SynthDef {
        SynthDef {
            name: "count".to_owned(),
            constants: vec![0.0],
            controls: Vec::new(),
            control_names: Vec::new(),
            children,
            variants: Vec::new(),
        }
    }

    fn ugen(name: &str, inputs: &[InputSpec], outputs: &[Rate]) -> UGen {
        UGen {
            name: name.to_owned(),
            rate: Rate::Audio,
            inputs: inputs.to_vec(),
            outputs: outputs.to_vec(),
            special_index: 0,
        }
    }

    const ZERO: InputSpec = InputSpec::Constant(0);

    #[test]
    fn a_ugen_outside_the_catalogue_has_no_count() {
        let def = def(vec![
            ugen("SinOsc", &[ZERO, ZERO], &[Rate::Audio]),
            ugen(
                "NoSuchUGen",
                &[InputSpec::UGen {
                    index: 0,
                    output: 0,
                }],
                &[Rate::Audio],
            ),
        ]);
        assert_eq!(
            def.wire_buffers(),
            Err(WireBufferError::UnknownUGen {
                ugen: 1,
                name: "NoSuchUGen".to_owned()
            })
        );
    }

    #[test]
    fn an_input_of_a_later_ugen_breaks_a_limit() {
        let def = def(vec![
            ugen(
                "Out",
                &[
                    ZERO,
                    InputSpec::UGen {
                        index: 1,
                        output: 0,
                    },
                ],
                &[],
            ),
            ugen("SinOsc", &[ZERO, ZERO], &[Rate::Audio]),
        ]);
        let Err(WireBufferError::Limit(error)) = def.wire_buffers() else {
            panic!("the count of a later input");
        };
        assert!(matches!(
            error.kind(),
            LimitErrorKind::LaterInput {
                ugen: 0,
                input: 1,
                source: 1
            }
        ));
    }
}
