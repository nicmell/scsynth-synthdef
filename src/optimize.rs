//! The rewrites of `finishBuild` before the sort (`SynthDef.sc:279`), on the
//! UGens of a build: the copies of PV chains of `addCopiesIfNeeded`, then
//! `optimizeGraph`.

use crate::builder::{BINARY_OP, MUL_ADD, Node, Source, UNARY_OP, can_be_mul_add};
use crate::catalog::{self, FftSize, Optimize};
use crate::error::BuildError;
use crate::operator;
use crate::rate::{Rate, least_by_name, sort_by_rate};

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
    /// The `synthIndex` of each node during `optimizeGraph`: its slot in
    /// `children`.
    slots: Vec<usize>,
    /// The `descendants` of each node during `optimizeGraph`, as an index
    /// into `arrays`. A replacement shares the array of the UGen that it
    /// replaces.
    readers: Vec<Option<usize>>,
    arrays: Vec<Readers>,
}

/// An Array of sclang that holds the readers of a UGen.
struct Readers {
    items: Vec<usize>,
    /// The size up to which `add` grows this Array. Beyond it, `add` gives
    /// back a new Array, which nobody keeps.
    room: usize,
}

impl Readers {
    /// `Set:asArray.sort` of these readers, with the room that sclang 3.14.1
    /// gives it (measured).
    fn new(items: Vec<usize>) -> Self {
        let room = if items.is_empty() {
            2
        } else {
            items.len().next_power_of_two().max(4)
        };
        Self { items, room }
    }

    /// `remove`: the first match goes.
    fn remove(&mut self, item: usize) {
        if let Some(at) = self.items.iter().position(|&found| found == item) {
            self.items.remove(at);
        }
    }
}

/// A UGen that a rewrite of `optimizeGraph` fuses into the UGen that reads
/// it.
#[derive(Clone, Copy)]
enum Fused {
    Plus,
    Times,
    Neg,
    Sum3,
}

impl Fused {
    fn matches(self, node: &Node) -> bool {
        let binary = |op| {
            node.name == BINARY_OP
                && operator::binary_index(op) == Some(node.special_index)
                && node.inputs.len() == 2
        };
        match self {
            Self::Plus => binary("+"),
            Self::Times => binary("*"),
            Self::Neg => {
                node.name == UNARY_OP
                    && operator::unary_index("neg") == Some(node.special_index)
                    && node.inputs.len() == 1
            }
            Self::Sum3 => node.name == SUM3 && node.inputs.len() == 3,
        }
    }
}

/// A step of `optimizeGraph` on its stack, in the order of the recursion of
/// sclang.
#[derive(Clone, Copy)]
enum Step {
    /// `optimizeGraph` of a UGen.
    Visit(usize),
    /// The rest of `performDeadCodeElimination` of a UGen, from an input on.
    Inputs { node: usize, input: usize },
}

/// A rewrite of a `+` or a `-` with its two inputs.
type Rewrite = fn(&mut Graph, usize, (Source, Source)) -> Fusion;

/// The replacement of a rewrite, and the UGen that it fuses.
type Fusion = Result<Option<(usize, usize)>, BuildError>;

const SUM3: &str = "Sum3";
const SUM4: &str = "Sum4";

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
const FOLDED: &str = "optimizeGraph of sclang 3.14.1 folds its rewrite into a number or an \
     input, then throws or writes a UGen twice";

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
            slots: vec![usize::MAX; nodes.len()],
            readers: vec![None; nodes.len()],
            nodes,
            width_first_order: width_first.to_vec(),
            arrays: Vec::new(),
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
            if !is_kind_of(&self.nodes[chain].name, "PV_ChainUGen") {
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

    /// `optimizeGraph`: each child, in order, removes itself when nothing
    /// reads it, and a `+` or a `-` fuses with its inputs. A graph that
    /// sclang 3.14.1 breaks or throws on refuses.
    pub(crate) fn optimize_graph(&mut self) -> Result<(), BuildError> {
        self.init_readers();
        let mut steps: Vec<Step> = self
            .children
            .iter()
            .flatten()
            .rev()
            .map(|&node| Step::Visit(node))
            .collect();
        while let Some(step) = steps.pop() {
            self.step(step, &mut steps)?;
        }
        Ok(())
    }

    /// The `descendants` of `initTopoSort`: the readers of the outputs of
    /// each child and the UGens that wait for it as a width-first
    /// antecedent, once each, in the order of the children.
    fn init_readers(&mut self) {
        let count = self.nodes.len();
        self.slots = vec![usize::MAX; count];
        for (slot, &child) in self.children.iter().enumerate() {
            if let Some(child) = child {
                self.slots[child] = slot;
            }
        }
        let mut readers = vec![Vec::new(); count];
        for &child in self.children.iter().flatten() {
            let sources = self.nodes[child]
                .inputs
                .iter()
                .filter_map(|input| match *input {
                    Source::Output { ugen, .. } => Some(ugen),
                    Source::Number(_) => None,
                });
            for source in sources.chain(self.antecedents[child].iter().copied()) {
                readers[source].push(child);
            }
        }
        self.readers = vec![None; count];
        self.arrays = Vec::with_capacity(count);
        for &child in self.children.iter().flatten() {
            let mut items = std::mem::take(&mut readers[child]);
            items.dedup();
            self.readers[child] = Some(self.arrays.len());
            self.arrays.push(Readers::new(items));
        }
    }

    /// `optimizeGraph` of one UGen, or the rest of its dead-code elimination.
    /// A UGen that nothing reads goes, as `performDeadCodeElimination` does:
    /// it leaves the readers of each direct input, in order, and the input
    /// gets a visit. An output of a `MultiOutUGen` comes through an
    /// `OutputProxy`, which has no readers, so the visit stops there (fact 7
    /// of the plan of 0.7). A `+` or a `-` that a UGen reads gets its
    /// rewrites, and the replacement gets a visit at once.
    fn step(&mut self, step: Step, steps: &mut Vec<Step>) -> Result<(), BuildError> {
        match step {
            Step::Visit(node) => {
                let optimize = optimize_of(&self.nodes[node].name);
                if optimize == Optimize::Nothing {
                    return Ok(());
                }
                if self.reader_count(node) == 0 {
                    steps.push(Step::Inputs { node, input: 0 });
                } else if optimize == Optimize::BinaryOp {
                    if let Some(replacement) = self.rewrite(node)? {
                        steps.push(Step::Visit(replacement));
                    }
                }
            }
            Step::Inputs { node, input } => {
                let Some(&source) = self.nodes[node].inputs.get(input) else {
                    self.remove_ugen(node);
                    return Ok(());
                };
                steps.push(Step::Inputs {
                    node,
                    input: input + 1,
                });
                if let Source::Output { ugen, .. } = source {
                    if !is_kind_of(&self.nodes[ugen].name, "MultiOutUGen") {
                        if let Some(array) = self.readers[ugen] {
                            self.arrays[array].remove(node);
                        }
                        steps.push(Step::Visit(ugen));
                    }
                }
            }
        }
        Ok(())
    }

    /// The first rewrite of a `+` or a `-` that holds, which takes its place:
    /// `optimizeAdd` tries a `Sum3`, a `Sum4`, a `MulAdd` and a `-`, and
    /// `optimizeSub` tries a `+`.
    fn rewrite(&mut self, node: usize) -> Result<Option<usize>, BuildError> {
        let Some(pair) = self.pair(node) else {
            return Ok(None);
        };
        let rewrites: &[Rewrite] = match operator::binary_name(self.nodes[node].special_index) {
            Some("+") => &[
                Self::fuse_sum3,
                Self::fuse_sum4,
                Self::fuse_mul_add,
                Self::add_neg,
            ],
            Some("-") => &[Self::sub_neg],
            _ => &[],
        };
        for rewrite in rewrites {
            if let Some((replacement, deleted)) = rewrite(self, node, pair)? {
                self.remove_ugen(deleted);
                self.adopt(node, replacement, deleted);
                self.replace_ugen(node, replacement);
                return Ok(Some(replacement));
            }
        }
        Ok(None)
    }

    /// `optimizeToSum3`: a `+` that only this `+` reads becomes a `Sum3`
    /// with the other input, or a `Sum4` when it is both inputs.
    fn fuse_sum3(&mut self, this: usize, (a, b): (Source, Source)) -> Fusion {
        if self.rate(a) == Rate::Demand || self.rate(b) == Rate::Demand {
            return Ok(None);
        }
        for (sum, other) in [(a, b), (b, a)] {
            if let Some(plus) = self.fusable(sum, Fused::Plus) {
                let (x, y) = (self.nodes[plus].inputs[0], self.nodes[plus].inputs[1]);
                let replacement = if a == b {
                    self.sum(this, SUM4, vec![x, x, y, y])?
                } else {
                    self.sum(this, SUM3, vec![x, y, other])?
                };
                return Ok(Some((replacement, plus)));
            }
        }
        Ok(None)
    }

    /// `optimizeToSum4`: a `Sum3` that only this `+` reads becomes a `Sum4`
    /// with the other input.
    fn fuse_sum4(&mut self, this: usize, (a, b): (Source, Source)) -> Fusion {
        if a == b || self.rate(a) == Rate::Demand || self.rate(b) == Rate::Demand {
            return Ok(None);
        }
        for (sum, other) in [(a, b), (b, a)] {
            if let Some(sum3) = self.fusable(sum, Fused::Sum3) {
                let mut inputs = self.nodes[sum3].inputs.clone();
                inputs.push(other);
                return Ok(Some((self.sum(this, SUM4, inputs)?, sum3)));
            }
        }
        Ok(None)
    }

    /// `optimizeToMulAdd`: a `*` that only this `+` reads becomes a `MulAdd`
    /// with the other input, if `canBeMulAdd` takes its inputs in one order.
    fn fuse_mul_add(&mut self, this: usize, (a, b): (Source, Source)) -> Fusion {
        if a == b {
            return Ok(None);
        }
        for (product, other) in [(a, b), (b, a)] {
            let Some(times) = self.fusable(product, Fused::Times) else {
                continue;
            };
            let (x, y) = (self.nodes[times].inputs[0], self.nodes[times].inputs[1]);
            for (input, mul) in [(x, y), (y, x)] {
                if can_be_mul_add(self.rate(input), self.rate(mul), self.rate(other)) {
                    return Ok(Some((self.mul_add(this, input, mul, other)?, times)));
                }
            }
        }
        Ok(None)
    }

    /// `optimizeAddNeg`: `a + b.neg` becomes `a - b`, and `a.neg + b`
    /// becomes `b - a`, for a `neg` that only this `+` reads.
    fn add_neg(&mut self, this: usize, (a, b): (Source, Source)) -> Fusion {
        if a == b {
            return Ok(None);
        }
        for (other, negated) in [(a, b), (b, a)] {
            if let Some(neg) = self.fusable(negated, Fused::Neg) {
                let inner = self.nodes[neg].inputs[0];
                return Ok(Some((self.binary(this, "-", other, inner, true)?, neg)));
            }
        }
        Ok(None)
    }

    /// `optimizeSub`: `a - b.neg` becomes `a + b`, for a `neg` that only
    /// this `-` reads.
    fn sub_neg(&mut self, this: usize, (a, b): (Source, Source)) -> Fusion {
        let Some(neg) = self.fusable(b, Fused::Neg) else {
            return Ok(None);
        };
        let inner = self.nodes[neg].inputs[0];
        Ok(Some((self.binary(this, "+", a, inner, false)?, neg)))
    }

    /// `Sum3.new1` or `Sum4.new1` of a rewrite: a new node, outside the
    /// children. An input of 0 folds, which refuses.
    fn sum(
        &mut self,
        this: usize,
        class: &str,
        mut inputs: Vec<Source>,
    ) -> Result<usize, BuildError> {
        if inputs.iter().any(|&input| is_number(input, 0.0)) {
            return Err(finish_build(&self.nodes[this].name, FOLDED));
        }
        let rate =
            least_by_name(inputs.iter().map(|&input| self.rate(input))).unwrap_or(Rate::Scalar);
        sort_by_rate(&mut inputs, |&input| self.rate(input));
        Ok(self.push(ugen(class, rate, inputs), Vec::new(), false))
    }

    /// `MulAdd.new1` of a rewrite, for inputs that `canBeMulAdd` takes in
    /// this order: a new node, outside the children. A `mul` of 0, 1 or -1
    /// and an `add` of 0 fold, which refuses.
    fn mul_add(
        &mut self,
        this: usize,
        input: Source,
        mul: Source,
        add: Source,
    ) -> Result<usize, BuildError> {
        let folds = [-1.0, 0.0, 1.0].iter().any(|&value| is_number(mul, value));
        if folds || is_number(add, 0.0) {
            return Err(finish_build(&self.nodes[this].name, FOLDED));
        }
        let rate = least_by_name([input, mul, add].map(|source| self.rate(source)));
        let node = ugen(MUL_ADD, rate.unwrap_or(Rate::Scalar), vec![input, mul, add]);
        Ok(self.push(node, Vec::new(), false))
    }

    /// The operator `-` of sclang, or `BinaryOpUGen.new` of `+`, in a
    /// rewrite: a new node, outside the children. The folds of
    /// `BinaryOpUGen:new1` refuse, and so does the operator on two numbers,
    /// which sclang computes.
    fn binary(
        &mut self,
        this: usize,
        op: &str,
        a: Source,
        b: Source,
        computes: bool,
    ) -> Result<usize, BuildError> {
        let numbers = matches!((a, b), (Source::Number(_), Source::Number(_)));
        if is_number(a, 0.0) || is_number(b, 0.0) || (computes && numbers) {
            return Err(finish_build(&self.nodes[this].name, FOLDED));
        }
        let special_index =
            operator::binary_index(op).ok_or_else(|| BuildError::UnknownOperator(op.to_owned()))?;
        let mut node = ugen(BINARY_OP, self.rate(a).max(self.rate(b)), vec![a, b]);
        node.special_index = special_index;
        Ok(self.push(node, Vec::new(), false))
    }

    /// `optimizeUpdateDescendants`: the readers of each input of the
    /// replacement get the replacement in place of `this`, and lose
    /// `deleted`. A full array loses the replacement (fact 7 of the plan of
    /// 0.7).
    fn adopt(&mut self, this: usize, replacement: usize, deleted: usize) {
        for input in &self.nodes[replacement].inputs {
            let Source::Output { ugen, .. } = *input else {
                continue;
            };
            let Some(array) = self.readers[ugen] else {
                continue;
            };
            let readers = &mut self.arrays[array];
            if readers.items.len() < readers.room {
                readers.items.push(replacement);
                readers.remove(this);
            }
            readers.remove(deleted);
        }
    }

    /// `replaceUGen`: `b` takes the slot, the width-first antecedents and the
    /// readers of `a`, and each child that reads `a` reads `b`.
    fn replace_ugen(&mut self, a: usize, b: usize) {
        self.antecedents[b] = std::mem::take(&mut self.antecedents[a]);
        self.readers[b] = self.readers[a];
        self.slots[b] = self.slots[a];
        if let Some(slot) = self.children.get_mut(self.slots[a]) {
            *slot = Some(b);
        }
        for &child in self.children.iter().flatten() {
            for input in &mut self.nodes[child].inputs {
                if reads(*input, a) {
                    *input = output(b);
                }
            }
        }
    }

    /// `removeUGen`: the slot of the UGen empties.
    fn remove_ugen(&mut self, node: usize) {
        if let Some(slot) = self.children.get_mut(self.slots[node]) {
            *slot = None;
        }
    }

    /// The UGen of `source` when it is of the kind `fused` and only one UGen
    /// reads it.
    fn fusable(&self, source: Source, fused: Fused) -> Option<usize> {
        let Source::Output { ugen, .. } = source else {
            return None;
        };
        (fused.matches(&self.nodes[ugen]) && self.reader_count(ugen) == 1).then_some(ugen)
    }

    fn pair(&self, node: usize) -> Option<(Source, Source)> {
        match self.nodes[node].inputs[..] {
            [a, b] => Some((a, b)),
            _ => None,
        }
    }

    fn reader_count(&self, node: usize) -> usize {
        self.readers[node].map_or(0, |array| self.arrays[array].items.len())
    }

    fn rate(&self, source: Source) -> Rate {
        match source {
            Source::Number(_) => Rate::Scalar,
            Source::Output { ugen, output } => usize::try_from(output)
                .ok()
                .and_then(|output| self.nodes[ugen].outputs.get(output))
                .copied()
                .unwrap_or(self.nodes[ugen].rate),
        }
    }

    /// The UGens in the order of the children, without the removed ones. A
    /// kept UGen that reads a removed UGen refuses: sclang 3.14.1 then drops
    /// UGens from the definition (fact 7 of the plan of 0.7).
    pub(crate) fn into_rewritten(self) -> Result<Rewritten, BuildError> {
        let order: Vec<usize> = self.children.iter().flatten().copied().collect();
        let mut position = vec![usize::MAX; self.nodes.len()];
        for (at, &node) in order.iter().enumerate() {
            position[node] = at;
        }
        for &child in &order {
            for input in &self.nodes[child].inputs {
                if let Source::Output { ugen, .. } = *input {
                    if position[ugen] == usize::MAX {
                        let message = format!(
                            "optimizeGraph of sclang 3.14.1 removes it while {} reads it, so \
                             sclang drops UGens",
                            self.nodes[child].name
                        );
                        return Err(finish_build(&self.nodes[ugen].name, &message));
                    }
                }
            }
        }
        let width_first = order
            .iter()
            .map(|&node| {
                self.antecedents[node]
                    .iter()
                    .map(|&antecedent| position[antecedent])
                    .filter(|&at| at != usize::MAX)
                    .collect()
            })
            .collect();
        let mut nodes: Vec<Option<Node>> = self.nodes.into_iter().map(Some).collect();
        let nodes = order
            .iter()
            .filter_map(|&node| nodes[node].take())
            .map(|mut node| {
                for input in &mut node.inputs {
                    if let Source::Output { ugen, .. } = input {
                        *ugen = position[*ugen];
                    }
                }
                node
            })
            .collect();
        Ok(Rewritten { nodes, width_first })
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
        let rule = catalog::find(&self.nodes[node].name).and_then(|class| class.fft_size);
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
        self.readers.push(None);
        self.slots.push(usize::MAX);
        if width_first {
            self.width_first_order.push(id);
        }
        id
    }
}

fn optimize_of(name: &str) -> Optimize {
    catalog::find(name).map_or(Optimize::Nothing, |class| class.optimize)
}

/// Whether the class `name` is `kind` or a subclass of it, as `isKindOf`
/// tests it.
fn is_kind_of(name: &str, kind: &str) -> bool {
    catalog::find(name)
        .is_some_and(|class| class.name == kind || class.superclasses.contains(&kind))
}

/// Whether `source` is the number `number`, as `==` compares them in
/// sclang.
#[allow(clippy::float_cmp)]
fn is_number(source: Source, number: f64) -> bool {
    matches!(source, Source::Number(value) if value == number)
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

fn finish_build(ugen: &str, message: &str) -> BuildError {
    BuildError::FinishBuild {
        ugen: ugen.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::Readers;
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
    fn a_dead_chain_of_ten_thousand_ugens_goes_on_the_stack_of_a_test_thread() {
        let mut builder = SynthDefBuilder::new("chain", vec![]).unwrap();
        let mut sound = by_name(&mut builder, "SinOsc", "ar", &[]);
        for _ in 0..10_000 {
            sound = builder.binary_op("*", sound, 2.0).unwrap();
        }
        assert_eq!(builder.finish().unwrap().children.len(), 0);
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

    fn sine(b: &mut SynthDefBuilder, freq: f64) -> UGenInput {
        by_name(b, "SinOsc", "ar", &[("freq", input(freq))])
    }

    fn out(b: &mut SynthDefBuilder, channels: UGenInput) {
        by_name(
            b,
            "Out",
            "ar",
            &[("bus", input(0.0)), ("channelsArray", input(channels))],
        );
    }

    #[test]
    fn an_array_of_readers_has_the_room_of_sclang() {
        // Measured in sclang 3.14.1: fact 7 of the plan of 0.7.
        let room = |count| Readers::new(vec![0; count]).room;
        assert_eq!([0, 1, 4, 5, 8, 65].map(room), [2, 4, 4, 8, 8, 128]);
    }

    #[test]
    fn a_full_array_of_readers_loses_a_ugen_that_a_mul_add_reads() {
        // refused_readers of generator/fixtures.scd: x has four readers.
        let message = refusal(|b| {
            let x = sine(b, 1.0);
            let product = b.binary_op("*", x.clone(), 2.0).unwrap();
            let sum = b.binary_op("+", product, 1.0).unwrap();
            for factor in [3.0, 4.0, 5.0] {
                b.binary_op("*", x.clone(), factor).unwrap();
            }
            out(b, sum);
        });
        assert_eq!(
            message,
            "SinOsc: optimizeGraph of sclang 3.14.1 removes it while MulAdd reads it, so sclang \
             drops UGens"
        );
    }

    #[test]
    fn a_rewrite_that_folds_into_an_input_is_refused() {
        // refused_neg_zero of generator/fixtures.scd: x + neg(0) becomes x - 0.
        let message = refusal(|b| {
            let x = sine(b, 440.0);
            let zero = b.unary_op("neg", 0.0).unwrap();
            let sum = b.binary_op("+", x, zero).unwrap();
            out(b, sum);
        });
        assert_eq!(
            message,
            "BinaryOpUGen: optimizeGraph of sclang 3.14.1 folds its rewrite into a number or an \
             input, then throws or writes a UGen twice"
        );
    }

    #[test]
    fn a_rewrite_that_folds_into_a_number_is_refused() {
        // refused_neg_numbers of generator/fixtures.scd: 2 + neg(3) becomes 2 - 3.
        let message = refusal(|b| {
            let x = sine(b, 440.0);
            let three = b.unary_op("neg", 3.0).unwrap();
            let sum = b.binary_op("+", 2.0, three).unwrap();
            let product = b.binary_op("*", x, sum).unwrap();
            out(b, product);
        });
        assert!(message.starts_with("BinaryOpUGen: optimizeGraph of sclang 3.14.1 folds"));
    }
}
