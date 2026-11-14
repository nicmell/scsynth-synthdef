//! The pseudo-UGens: a body for each rate method that the catalogue marks
//! `Addable::Pseudo`. A body adds what its method adds in sclang, in the order
//! in which sclang adds it.

mod basic_ops;
mod beq_suite;
mod buf_io;
mod compander;
mod demand;
mod fft_unpacking;
mod filter;
mod fsin_osc;
mod hilbert;
mod in_out;
pub(crate) mod lang;
mod line;
mod machine_listening;
mod mix;
mod osc;
mod pm_osc;
mod poll;
mod sound_in;
mod splay;

use crate::builder::{SynthDefBuilder, count_to_usize};
use crate::catalog::RateMethod;
use crate::error::BuildError;
use crate::ugen_input::{ArgValue, UGenInput};

/// The arguments of a call of a pseudo-UGen, bound by name, with their
/// defaults. An argument whose default is nil has no value.
pub(crate) struct Args {
    class: &'static str,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
}

impl Args {
    /// The selector of the method: `ar`, `kr`, `ir` or `new`.
    pub(crate) fn selector(&self) -> &'static str {
        self.method.selector
    }

    /// The value of the argument `name`.
    pub(crate) fn value(&self, name: &str) -> Result<ArgValue, BuildError> {
        self.slot(name)?
            .clone()
            .ok_or_else(|| BuildError::MissingArgument {
                class: self.class.to_owned(),
                argument: name.to_owned(),
            })
    }

    /// The input of the argument `name`: a number, an output, or an array of
    /// them.
    pub(crate) fn input(&self, name: &str) -> Result<UGenInput, BuildError> {
        match self.value(name)? {
            ArgValue::Input(input) => Ok(input),
            _ => Err(self.wrong_kind(name, "a number or an output")),
        }
    }

    /// The input of the argument `name`, or none for an argument whose
    /// default is nil.
    pub(crate) fn optional(&self, name: &str) -> Result<Option<UGenInput>, BuildError> {
        match self.slot(name)? {
            None => Ok(None),
            Some(ArgValue::Input(input)) => Ok(Some(input.clone())),
            Some(_) => Err(self.wrong_kind(name, "a number or an output")),
        }
    }

    /// The number of the argument `name`.
    pub(crate) fn constant(&self, name: &str) -> Result<f64, BuildError> {
        match self.value(name)? {
            ArgValue::Input(UGenInput::Number(number)) => Ok(number),
            _ => Err(self.wrong_kind(name, "a number")),
        }
    }

    /// The boolean of the argument `name`.
    pub(crate) fn boolean(&self, name: &str) -> Result<bool, BuildError> {
        match self.value(name)? {
            ArgValue::Bool(value) => Ok(value),
            _ => Err(self.wrong_kind(name, "a boolean")),
        }
    }

    /// The elements of the array argument `name`. A single input is an array
    /// of one, as `asArray` makes it.
    pub(crate) fn array(&self, name: &str) -> Result<Vec<UGenInput>, BuildError> {
        match self.value(name)? {
            ArgValue::Input(UGenInput::Array(items)) => Ok(items),
            ArgValue::Input(input) => Ok(vec![input]),
            _ => Err(self.wrong_kind(name, "an array")),
        }
    }

    /// The text of the argument `name`, or none for an argument whose default
    /// is nil.
    pub(crate) fn text(&self, name: &str) -> Result<Option<String>, BuildError> {
        match self.slot(name)? {
            None => Ok(None),
            Some(ArgValue::Text(text)) => Ok(Some(text.clone())),
            Some(_) => Err(self.wrong_kind(name, "a symbol")),
        }
    }

    /// The count of the argument `name`: a whole number from 1.
    pub(crate) fn count(&self, name: &str) -> Result<usize, BuildError> {
        match self.value(name)? {
            ArgValue::Input(input) => self.count_of(name, &input),
            _ => Err(self.bad_count(name)),
        }
    }

    /// `value` as a count of the argument `name`, as an instance of the
    /// expansion takes it: a whole number from 1.
    pub(crate) fn count_of(&self, name: &str, value: &UGenInput) -> Result<usize, BuildError> {
        match *value {
            UGenInput::Number(count)
                if count >= 1.0 && count.fract() == 0.0 && count <= f64::from(u16::MAX) =>
            {
                Ok(count_to_usize(count))
            }
            _ => Err(self.bad_count(name)),
        }
    }

    fn bad_count(&self, name: &str) -> BuildError {
        BuildError::BadCount {
            class: self.class.to_owned(),
            argument: name.to_owned(),
        }
    }

    pub(crate) fn wrong_kind(&self, name: &str, expected: &'static str) -> BuildError {
        BuildError::WrongKind {
            class: self.class.to_owned(),
            argument: name.to_owned(),
            expected,
        }
    }

    fn slot(&self, name: &str) -> Result<&Option<ArgValue>, BuildError> {
        self.method
            .args
            .iter()
            .position(|arg| arg.name == name)
            .and_then(|position| self.values.get(position))
            .ok_or_else(|| BuildError::UnknownArgument {
                class: self.class.to_owned(),
                argument: name.to_owned(),
            })
    }
}

/// What one rate method adds.
type Body = fn(&mut SynthDefBuilder, &Args) -> Result<UGenInput, BuildError>;

/// The body of each method that the catalogue marks `Addable::Pseudo`, by
/// class and selector.
static BODIES: &[(&str, &str, Body)] = &[
    ("BHiPass4", "ar", beq_suite::b_hi_pass4),
    ("BLowPass4", "ar", beq_suite::b_low_pass4),
    ("BeatTrack", "kr", machine_listening::beat_track),
    ("Changed", "ar", filter::changed),
    ("Changed", "kr", filter::changed),
    ("CompanderD", "ar", compander::compander_d),
    ("Dstutter", "new", demand::dstutter),
    ("DynKlang", "ar", fsin_osc::dyn_klang),
    ("DynKlang", "kr", fsin_osc::dyn_klang),
    ("DynKlank", "ar", fsin_osc::dyn_klank),
    ("DynKlank", "kr", fsin_osc::dyn_klank),
    ("HilbertFIR", "ar", hilbert::hilbert_fir),
    ("Klang", "ar", fsin_osc::klang),
    ("Klank", "ar", fsin_osc::klank),
    ("LinLin", "ar", line::lin_lin_ar),
    ("LinLin", "kr", line::lin_lin_kr),
    ("LinSelectX", "ar", osc::lin_select_x),
    ("LinSelectX", "kr", osc::lin_select_x),
    ("LocalBuf", "new", buf_io::local_buf),
    ("LocalIn", "ar", in_out::local_in),
    ("LocalIn", "kr", in_out::local_in),
    ("Mix", "ar", mix::mix_ar),
    ("Mix", "kr", mix::mix_kr),
    ("Mix", "new", mix::mix_new),
    ("MultiTap", "ar", pm_osc::multi_tap),
    ("NumChannels", "ar", mix::num_channels),
    ("PMOsc", "ar", pm_osc::pm_osc),
    ("PMOsc", "kr", pm_osc::pm_osc),
    ("Poll", "ar", poll::poll),
    ("Poll", "kr", poll::poll),
    ("Poll", "new", poll::poll),
    ("SelectX", "ar", osc::select_x),
    ("SelectX", "kr", osc::select_x),
    ("SelectXFocus", "ar", osc::select_x_focus),
    ("SelectXFocus", "kr", osc::select_x_focus),
    ("SelectXFocus", "new", osc::select_x_focus),
    ("Silent", "ar", line::silent),
    ("SoundIn", "ar", sound_in::sound_in),
    ("Splay", "ar", splay::splay),
    ("Splay", "kr", splay::splay),
    ("SplayAz", "ar", splay::splay_az),
    ("SplayAz", "kr", splay::splay_az),
    ("Sum3", "new", basic_ops::sum3),
    ("Sum4", "new", basic_ops::sum4),
    ("TChoose", "ar", osc::t_choose),
    ("TChoose", "kr", osc::t_choose),
    ("TWChoose", "ar", osc::tw_choose),
    ("TWChoose", "kr", osc::tw_choose),
    ("Tap", "ar", buf_io::tap),
    ("UnpackFFT", "new", fft_unpacking::unpack_fft),
];

/// Runs the body of the method `method` of `class` on its bound arguments.
pub(crate) fn build(
    builder: &mut SynthDefBuilder,
    class: &'static str,
    method: &'static RateMethod,
    values: Vec<Option<ArgValue>>,
) -> Result<UGenInput, BuildError> {
    let body = BODIES
        .iter()
        .find(|(name, selector, _)| *name == class && *selector == method.selector)
        .map(|&(_, _, body)| body)
        .ok_or_else(|| BuildError::NotAddable {
            class: class.to_owned(),
            selector: method.selector.to_owned(),
            reason: "the crate has no body for the method".to_owned(),
        })?;
    body(
        builder,
        &Args {
            class,
            method,
            values,
        },
    )
}

/// The elements of an array, or the value itself as an array of one.
pub(crate) fn elements(value: UGenInput) -> Vec<UGenInput> {
    match value {
        UGenInput::Array(items) => items,
        other => vec![other],
    }
}

/// A number of elements as a number of sclang.
pub(crate) fn number(count: usize) -> UGenInput {
    UGenInput::Number(f64::from(u32::try_from(count).unwrap_or(u32::MAX)))
}

#[cfg(test)]
mod tests {
    use super::BODIES;
    use crate::catalog::{self, Addable, RateMethod};

    fn pseudo<'a>(
        class: &'a str,
        methods: &'a [RateMethod],
    ) -> impl Iterator<Item = (&'a str, &'a str)> {
        methods
            .iter()
            .filter(|method| method.addable == Addable::Pseudo)
            .map(move |method| (class, method.selector))
    }

    #[test]
    fn each_pseudo_method_has_a_body_and_each_body_its_method() {
        let mut methods: Vec<(&str, &str)> = catalog::all()
            .iter()
            .flat_map(|class| pseudo(class.name, class.methods))
            .chain(
                catalog::pseudo_all()
                    .iter()
                    .flat_map(|class| pseudo(class.name, class.methods)),
            )
            .collect();
        methods.sort_unstable();
        let mut bodies: Vec<(&str, &str)> = BODIES
            .iter()
            .map(|&(class, selector, _)| (class, selector))
            .collect();
        bodies.sort_unstable();
        assert_eq!(methods, bodies);
    }

    #[test]
    fn every_method_of_a_pseudo_class_has_a_body() {
        for class in catalog::pseudo_all() {
            for method in class.methods {
                assert_eq!(method.addable, Addable::Pseudo, "{}", class.name);
            }
        }
    }
}
