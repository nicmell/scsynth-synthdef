//! The rewrites of `finishBuild` before the sort (`SynthDef.sc:279`), on the
//! UGens of a build: the copies of PV chains of `addCopiesIfNeeded`.

use crate::builder::{Node, Source};
use crate::catalog::{self, FftSize};
use crate::error::BuildError;
use crate::rate::Rate;

/// The UGens of a build while `finishBuild` rewrites them. The nodes are an
/// arena, and `children` holds the `children` of sclang as indices into it.
pub(crate) struct Graph {
    nodes: Vec<Node>,
    /// The width-first antecedents of each node.
    antecedents: Vec<Vec<usize>>,
    /// Whether each node is a `WidthFirstUGen`.
    width_first: Vec<bool>,
    children: Vec<Option<usize>>,
    /// The `widthFirstUGens` of sclang.
    width_first_order: Vec<usize>,
}

/// The UGens after the rewrites, in the order of the children of sclang.
/// Each input and each width-first antecedent is a position in this order.
pub(crate) struct Rewritten {
    pub(crate) nodes: Vec<Node>,
    pub(crate) width_first: Vec<Vec<usize>>,
}

const NO_LOCAL_BUF: &str = "its chain needs a copy, and the definition has no LocalBuf, so \
     addCopiesIfNeeded of sclang 3.14.1 drops UGens";
const PACK_FFT: &str = "its chain needs a copy, and addCopiesIfNeeded of sclang 3.14.1 fails on \
     the FFT size of a PackFFT";
const NO_FFT_SIZE: &str =
    "its chain needs a copy, and addCopiesIfNeeded of sclang 3.14.1 finds no FFT size for it";

impl Graph {
    /// The UGens of a build, with its width-first UGens in the order of
    /// their creation.
    pub(crate) fn new(nodes: Vec<Node>, width_first: &[usize]) -> Self {
        let mut is_width_first = vec![false; nodes.len()];
        for &node in width_first {
            is_width_first[node] = true;
        }
        Self {
            antecedents: nodes
                .iter()
                .map(|node| width_first[..node.width_first_before].to_vec())
                .collect(),
            width_first: is_width_first,
            children: (0..nodes.len()).map(Some).collect(),
            nodes,
            width_first_order: width_first.to_vec(),
        }
    }

    pub(crate) fn set_input(&mut self, node: usize, input: usize, source: Source) {
        self.nodes[node].inputs[input] = source;
    }

    /// `addCopiesIfNeeded`: a PV chain that several width-first UGens read
    /// gets a copy for each slot of each reader but the last. It gives back
    /// the number of copies, each with a `LocalBuf` for the `MaxLocalBufs`
    /// of the build.
    pub(crate) fn add_copies(&mut self, max_local_bufs: Option<usize>) -> Result<u32, BuildError> {
        let mut copies = 0;
        let mut next = 0;
        while let Some(&chain) = self.width_first_order.get(next) {
            next += 1;
            if !is_pv_chain(&self.nodes[chain].name) {
                continue;
            }
            let readers = self.readers_of_chain(chain);
            let Some((_, but_last)) = readers.split_last() else {
                continue;
            };
            for &reader in but_last {
                for slot in 0..self.nodes[reader].inputs.len() {
                    if reads(self.nodes[reader].inputs[slot], chain) {
                        let copy = self.copy(chain, max_local_bufs)?;
                        self.nodes[reader].inputs[slot] = output(copy);
                        copies += 1;
                    }
                }
            }
        }
        Ok(copies)
    }

    /// The UGens in the order of the children, without the removed ones.
    pub(crate) fn into_rewritten(self) -> Rewritten {
        let order: Vec<usize> = self.children.iter().flatten().copied().collect();
        let mut position = vec![usize::MAX; self.nodes.len()];
        for (at, &node) in order.iter().enumerate() {
            position[node] = at;
        }
        let at = |source: Source| match source {
            Source::Output { ugen, output } => Source::Output {
                ugen: position[ugen],
                output,
            },
            number @ Source::Number(_) => number,
        };
        Rewritten {
            width_first: order
                .iter()
                .map(|&node| {
                    self.antecedents[node]
                        .iter()
                        .map(|&antecedent| position[antecedent])
                        .collect()
                })
                .collect(),
            nodes: order
                .iter()
                .map(|&node| {
                    let mut node = self.nodes[node].clone();
                    node.inputs = node.inputs.into_iter().map(at).collect();
                    node
                })
                .collect(),
        }
    }

    /// The readers that `addCopiesIfNeeded` copies a chain for: the
    /// width-first children that read it, other than a `PV_Copy`, in order.
    fn readers_of_chain(&self, chain: usize) -> Vec<usize> {
        self.children
            .iter()
            .flatten()
            .copied()
            .filter(|&child| {
                self.width_first[child]
                    && self.nodes[child].name != "PV_Copy"
                    && self.nodes[child]
                        .inputs
                        .iter()
                        .any(|&input| reads(input, chain))
            })
            .collect()
    }

    /// A copy of a chain, right after it: a `BufFrames`, a `LocalBuf` and a
    /// `PV_Copy`.
    fn copy(&mut self, chain: usize, max_local_bufs: Option<usize>) -> Result<usize, BuildError> {
        let frames = self.fft_size(chain, chain)?;
        let Some(max_local_bufs) = max_local_bufs else {
            return Err(finish_build(&self.nodes[chain].name, NO_LOCAL_BUF));
        };
        let inputs = vec![Source::Number(1.0), output(frames), output(max_local_bufs)];
        let buf = self.push(ugen("LocalBuf", Rate::Scalar, inputs), Vec::new(), true);
        let mut antecedents = self.antecedents[chain].clone();
        antecedents.push(buf);
        let inputs = vec![output(chain), output(buf)];
        let copy = self.push(ugen("PV_Copy", Rate::Control, inputs), antecedents, true);
        let after = self
            .children
            .iter()
            .position(|&child| child == Some(chain))
            .map_or(self.children.len(), |at| at + 1);
        self.children
            .splice(after..after, [Some(frames), Some(buf), Some(copy)]);
        Ok(copy)
    }

    /// `fftSize` of the chain of `node`, as a new `BufFrames`, for a copy of
    /// `chain`.
    fn fft_size(&mut self, chain: usize, node: usize) -> Result<usize, BuildError> {
        let rule = catalog::lookup(&self.nodes[node].name).and_then(|class| class.fft_size);
        match (rule, self.nodes[node].inputs.first()) {
            (Some(FftSize::BufFrames), Some(&buffer)) => {
                let frames = ugen("BufFrames", Rate::Scalar, vec![buffer]);
                Ok(self.push(frames, Vec::new(), false))
            }
            (Some(FftSize::Chain), Some(&Source::Output { ugen, .. })) => {
                self.fft_size(chain, ugen)
            }
            (Some(FftSize::Input), _) => Err(finish_build(&self.nodes[chain].name, PACK_FFT)),
            _ => Err(finish_build(&self.nodes[chain].name, NO_FFT_SIZE)),
        }
    }

    /// Appends a node to the arena, outside the children.
    fn push(&mut self, node: Node, antecedents: Vec<usize>, width_first: bool) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        self.antecedents.push(antecedents);
        self.width_first.push(width_first);
        if width_first {
            self.width_first_order.push(id);
        }
        id
    }
}

/// A UGen with one output at its rate, as `multiNew` of these classes
/// makes it.
fn ugen(name: &str, rate: Rate, inputs: Vec<Source>) -> Node {
    Node {
        name: name.to_owned(),
        rate,
        inputs,
        outputs: vec![rate],
        special_index: 0,
        width_first_before: 0,
    }
}

fn output(ugen: usize) -> Source {
    Source::Output { ugen, output: 0 }
}

fn reads(input: Source, ugen: usize) -> bool {
    matches!(input, Source::Output { ugen: source, .. } if source == ugen)
}

fn is_pv_chain(name: &str) -> bool {
    catalog::lookup(name).is_some_and(|class| {
        class.name == "PV_ChainUGen" || class.superclasses.contains(&"PV_ChainUGen")
    })
}

fn finish_build(ugen: &str, message: &str) -> BuildError {
    BuildError::FinishBuild {
        ugen: ugen.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{ArgValue, BuildError, NewUGen, Rate, SynthDefBuilder, UGenInput};

    fn input(value: impl Into<UGenInput>) -> ArgValue {
        ArgValue::Input(value.into())
    }

    fn by_name(
        b: &mut SynthDefBuilder,
        class: &str,
        selector: &str,
        args: &[(&str, ArgValue)],
    ) -> UGenInput {
        b.ugen(class, selector, args).unwrap()
    }

    /// The refusal of `finish`, for the graph that `build` adds.
    fn refusal(build: impl FnOnce(&mut SynthDefBuilder)) -> String {
        let mut builder = SynthDefBuilder::new("refused", vec![]).unwrap();
        build(&mut builder);
        match builder.finish() {
            Err(error @ BuildError::FinishBuild { .. }) => error.to_string(),
            other => panic!("{other:?}"),
        }
    }

    /// `FFT(buffer, SinOsc.ar)`.
    fn chain_of(b: &mut SynthDefBuilder, buffer: UGenInput) -> UGenInput {
        let sound = by_name(b, "SinOsc", "ar", &[]);
        by_name(
            b,
            "FFT",
            "new",
            &[("buffer", input(buffer)), ("in", input(sound))],
        )
    }

    fn local_buf(b: &mut SynthDefBuilder) -> UGenInput {
        by_name(b, "LocalBuf", "new", &[("numFrames", input(512.0))])
    }

    /// `Out.ar(0, IFFT(a) + IFFT(c))`.
    fn out_iffts(b: &mut SynthDefBuilder, a: UGenInput, c: UGenInput) {
        let left = by_name(b, "IFFT", "ar", &[("buffer", input(a))]);
        let right = by_name(b, "IFFT", "ar", &[("buffer", input(c))]);
        let sum = b.binary_op("+", left, right).unwrap();
        by_name(
            b,
            "Out",
            "ar",
            &[("bus", input(0.0)), ("channelsArray", input(sum))],
        );
    }

    #[test]
    fn a_chain_needs_a_local_buf_in_the_definition_for_its_copy() {
        // refused_fft_buffer of generator/fixtures.scd.
        let message = refusal(|b| {
            let chain = chain_of(b, 0.0.into());
            let threshold = || ("threshold", input(1.0));
            let above = by_name(
                b,
                "PV_MagAbove",
                "new",
                &[("buffer", input(chain.clone())), threshold()],
            );
            let below = by_name(
                b,
                "PV_MagBelow",
                "new",
                &[("buffer", input(chain)), threshold()],
            );
            out_iffts(b, above, below);
        });
        assert_eq!(
            message,
            "FFT: its chain needs a copy, and the definition has no LocalBuf, so \
             addCopiesIfNeeded of sclang 3.14.1 drops UGens"
        );
    }

    #[test]
    fn a_chain_of_pack_fft_gets_no_copy() {
        // refused_packfft of generator/fixtures.scd: PackFFT(chain, 512, [1, 0]).
        let message = refusal(|b| {
            let buffer = local_buf(b);
            let chain = chain_of(b, buffer);
            let inputs = [
                chain,
                512.0.into(),
                0.0.into(),
                256.0.into(),
                0.0.into(),
                2.0.into(),
                1.0.into(),
                0.0.into(),
            ];
            let pack = b
                .add(
                    NewUGen::new("PackFFT", Rate::Control)
                        .inputs(inputs)
                        .width_first(true),
                )
                .unwrap();
            out_iffts(b, pack.clone(), pack);
        });
        assert_eq!(
            message,
            "PackFFT: its chain needs a copy, and addCopiesIfNeeded of sclang 3.14.1 fails on \
             the FFT size of a PackFFT"
        );
    }

    #[test]
    fn a_chain_without_an_fft_size_gets_no_copy() {
        // refused_ffttrigger of generator/fixtures.scd.
        let message = refusal(|b| {
            let buffer = local_buf(b);
            let chain = by_name(b, "FFTTrigger", "new", &[("buffer", input(buffer))]);
            out_iffts(b, chain.clone(), chain);
        });
        assert_eq!(
            message,
            "FFTTrigger: its chain needs a copy, and addCopiesIfNeeded of sclang 3.14.1 finds \
             no FFT size for it"
        );
    }
}
