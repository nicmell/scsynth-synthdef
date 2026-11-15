//! The envelopes of sclang: `Env`, its curves, and the arrays that `EnvGen`
//! reads.

use crate::error::BuildError;
use crate::pseudo;
use crate::ugen_input::UGenInput;

/// An envelope of sclang, `Env`: its levels, its times, its curves, its
/// nodes and its offset, as a constructor of sclang leaves them.
/// [`SynthDefBuilder::env`](crate::SynthDefBuilder::env) makes one, and
/// `EnvGen` and `IEnvGen` take one as their `envelope`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Env {
    /// The levels: numbers, outputs, or arrays of them for channels.
    pub levels: Vec<UGenInput>,
    /// The time of each segment, one fewer than the levels.
    pub times: Vec<UGenInput>,
    /// The curves: one curve, or one per segment, in turn.
    pub curves: Curve,
    /// The node where the envelope holds until its gate closes.
    pub release_node: Option<f64>,
    /// The node where the envelope loops back to from its release node.
    pub loop_node: Option<f64>,
    /// The offset of the times, which only `IEnvGen` reads.
    pub offset: UGenInput,
}

/// A curve of an envelope, as `Env` takes it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Curve {
    /// A shape by name, such as `lin`, `exp` or `sin`.
    Shape(String),
    /// A number or an output: the shape 5, with this curvature.
    Value(UGenInput),
    /// One curve per segment, in turn, or one per channel inside a segment.
    Array(Vec<Curve>),
}

impl From<&str> for Curve {
    fn from(name: &str) -> Self {
        Self::Shape(name.to_owned())
    }
}

impl From<String> for Curve {
    fn from(name: String) -> Self {
        Self::Shape(name)
    }
}

impl From<f64> for Curve {
    fn from(curvature: f64) -> Self {
        Self::Value(curvature.into())
    }
}

impl From<UGenInput> for Curve {
    fn from(input: UGenInput) -> Self {
        match input {
            UGenInput::Array(items) => Self::Array(items.into_iter().map(Self::from).collect()),
            value => Self::Value(value),
        }
    }
}

impl From<Vec<Curve>> for Curve {
    fn from(curves: Vec<Curve>) -> Self {
        Self::Array(curves)
    }
}

/// The shapes of `Env.shapeNames` in sclang 3.14.1.
const SHAPES: &[(&str, f64)] = &[
    ("step", 0.0),
    ("lin", 1.0),
    ("linear", 1.0),
    ("exp", 2.0),
    ("exponential", 2.0),
    ("sin", 3.0),
    ("sine", 3.0),
    ("wel", 4.0),
    ("welch", 4.0),
    ("sqr", 6.0),
    ("squared", 6.0),
    ("cub", 7.0),
    ("cubed", 7.0),
    ("hold", 8.0),
];

/// The number of the shape `name`, as `Env.shapeNames` has it.
pub(crate) fn shape_named(name: &str) -> Option<f64> {
    SHAPES
        .iter()
        .find(|(shape, _)| *shape == name)
        .map(|&(_, number)| number)
}

impl Env {
    /// An envelope as `Env.new` makes it: the times wrap to one fewer than
    /// the levels.
    pub(crate) fn from_parts(
        levels: Vec<UGenInput>,
        times: Vec<UGenInput>,
        curves: Curve,
        release_node: Option<f64>,
        loop_node: Option<f64>,
        offset: UGenInput,
    ) -> Result<Self, BuildError> {
        let Some(segments) = levels.len().checked_sub(1) else {
            return Err(bad("Env.new", "an envelope needs a level"));
        };
        if times.is_empty() && segments > 0 {
            return Err(bad("Env.new", "an envelope with segments needs a time"));
        }
        let times = times.into_iter().cycle().take(segments).collect();
        Ok(Self {
            levels,
            times,
            curves,
            release_node,
            loop_node,
            offset,
        })
    }

    /// `asArray`: the envelope as `EnvGen` reads it. One channel gives its
    /// array, and several give an array of them.
    ///
    /// # Errors
    ///
    /// [`BuildError::BadEnv`] for a shape that sclang does not know, and for
    /// a level or a curve that the envelope lacks.
    pub fn as_array(&self) -> Result<UGenInput, BuildError> {
        self.as_multichannel_array().map(unbubble)
    }

    /// `asMultichannelArray`: the array of [`as_array`](Self::as_array) of
    /// each channel, one channel included.
    ///
    /// # Errors
    ///
    /// As [`as_array`](Self::as_array).
    pub fn as_multichannel_array(&self) -> Result<UGenInput, BuildError> {
        let node = |node: Option<f64>| UGenInput::Number(node.unwrap_or(-99.0));
        let mut contents = vec![
            self.level(0)?,
            pseudo::number(self.times.len()),
            node(self.release_node),
            node(self.loop_node),
        ];
        for (index, time) in self.times.iter().enumerate() {
            let curve = self.curve(index)?;
            contents.push(self.level(index + 1)?);
            contents.push(time.clone());
            contents.push(shape_number(curve)?);
            contents.push(curve_value(curve));
        }
        Ok(flop(&contents))
    }

    /// `delay`: the envelope after `delay` seconds at its first level. Its
    /// nodes move one on, and its offset is 0, as `Env.new` gives it.
    #[must_use]
    pub fn delay(&self, delay: impl Into<UGenInput>) -> Self {
        let levels = self
            .levels
            .first()
            .into_iter()
            .chain(&self.levels)
            .cloned()
            .collect();
        let times = std::iter::once(delay.into())
            .chain(self.times.iter().cloned())
            .collect();
        let curves = match &self.curves {
            Curve::Array(curves) => Curve::Array(
                std::iter::once(Curve::from("lin"))
                    .chain(curves.iter().cloned())
                    .collect(),
            ),
            curve => curve.clone(),
        };
        Self {
            levels,
            times,
            curves,
            release_node: self.release_node.map(|node| node + 1.0),
            loop_node: self.loop_node.map(|node| node + 1.0),
            offset: UGenInput::Number(0.0),
        }
    }

    /// The level `index`.
    pub(crate) fn level(&self, index: usize) -> Result<UGenInput, BuildError> {
        self.levels
            .get(index)
            .cloned()
            .ok_or_else(|| bad("Env", "the envelope lacks a level for its times"))
    }

    /// The curve of the segment `index`: `curves.asArray.wrapAt(index)`.
    pub(crate) fn curve(&self, index: usize) -> Result<&Curve, BuildError> {
        match &self.curves {
            Curve::Array(curves) if curves.is_empty() => {
                Err(bad("Env", "the envelope has no curve"))
            }
            Curve::Array(curves) => Ok(&curves[index % curves.len()]),
            curve => Ok(curve),
        }
    }
}

/// `Env.shapeNumber`: the shape of a curve, an array of them for an array.
pub(crate) fn shape_number(curve: &Curve) -> Result<UGenInput, BuildError> {
    match curve {
        Curve::Shape(name) => shape_named(name)
            .map(UGenInput::Number)
            .ok_or_else(|| bad("Env", &format!("no shape is named {name}"))),
        Curve::Value(UGenInput::Number(number)) if number.is_nan() => {
            Err(bad("Env", "a curvature of NaN has no shape"))
        }
        Curve::Value(_) => Ok(UGenInput::Number(5.0)),
        Curve::Array(curves) => curves
            .iter()
            .map(shape_number)
            .collect::<Result<_, _>>()
            .map(|shapes| unbubble(UGenInput::Array(shapes))),
    }
}

/// `curveValue`: the curvature of a curve, 0 for a shape by name.
pub(crate) fn curve_value(curve: &Curve) -> UGenInput {
    match curve {
        Curve::Shape(_) => UGenInput::Number(0.0),
        Curve::Value(value) => value.clone(),
        Curve::Array(curves) => UGenInput::Array(curves.iter().map(curve_value).collect()),
    }
}

/// `flop`: one array per channel, where an array wraps and anything else
/// repeats.
pub(crate) fn flop(contents: &[UGenInput]) -> UGenInput {
    let channels = contents
        .iter()
        .map(|item| match item {
            UGenInput::Array(items) => items.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(0);
    UGenInput::Array(
        (0..channels)
            .map(|channel| {
                UGenInput::Array(
                    contents
                        .iter()
                        .map(|item| match item {
                            UGenInput::Array(items) if !items.is_empty() => {
                                items[channel % items.len()].clone()
                            }
                            other => other.clone(),
                        })
                        .collect(),
                )
            })
            .collect(),
    )
}

/// `unbubble`: the element of an array of one.
pub(crate) fn unbubble(input: UGenInput) -> UGenInput {
    match input {
        UGenInput::Array(mut items) if items.len() == 1 => items.remove(0),
        other => other,
    }
}

pub(crate) fn bad(method: &str, message: &str) -> BuildError {
    BuildError::BadEnv {
        method: method.to_owned(),
        message: message.to_owned(),
    }
}
