//! The pseudo-UGens of sc3-plugins: a body for each rate method that the
//! catalogue of sc3-plugins marks `Addable::Pseudo`, with the `Args` of the
//! bodies of the core.

mod deind_ugens;
mod nh_ugens;

use crate::pseudo::Body;

/// The body of each method that the catalogue of sc3-plugins marks
/// `Addable::Pseudo`, by class and selector.
pub(crate) static BODIES: &[(&str, &str, Body)] = &[
    ("Greyhole", "ar", deind_ugens::greyhole),
    ("JPverb", "ar", deind_ugens::jp_verb),
    ("NHHall", "ar", nh_ugens::nh_hall),
];

#[cfg(test)]
mod tests {
    use super::BODIES;
    use crate::builder::SynthDefBuilder;
    use crate::catalog::{Addable, RateMethod};
    use crate::error::BuildError;
    use crate::sc3_plugins::{all, pseudo_all};
    use crate::ugen_input::UGenInput;

    /// A builder with a sine at audio rate.
    fn sine() -> (SynthDefBuilder, UGenInput) {
        let mut builder = SynthDefBuilder::new("reverb", vec![]).unwrap();
        let sine = builder.ugen("SinOsc", "ar", &[]).unwrap();
        (builder, sine)
    }

    #[test]
    fn greyhole_and_jp_verb_read_the_first_and_the_last_channel() {
        for (class, raw) in [("Greyhole", "GreyholeRaw"), ("JPverb", "JPverbRaw")] {
            let (mut builder, sine) = sine();
            builder.ugen(class, "ar", &[("in", sine.into())]).unwrap();
            let def = builder.finish().unwrap();
            let ugen = def.children.iter().find(|ugen| ugen.name == raw).unwrap();
            assert_eq!(ugen.inputs[0], ugen.inputs[1], "{class}");
            assert_eq!(ugen.outputs.len(), 2, "{class}");
        }
    }

    #[test]
    fn nh_hall_takes_two_channels_and_gives_two() {
        let (mut builder, sine) = sine();
        let error = builder
            .ugen("NHHall", "ar", &[("in", sine.clone().into())])
            .unwrap_err();
        assert!(matches!(error, BuildError::WrongKind { ref argument, .. } if argument == "in"));
        let stereo = UGenInput::Array(vec![sine.clone(), sine]);
        let hall = builder
            .ugen("NHHall", "ar", &[("in", stereo.into())])
            .unwrap();
        assert!(matches!(hall, UGenInput::Array(ref outputs) if outputs.len() == 2));
    }

    #[test]
    fn each_pseudo_method_has_a_body_and_each_body_its_method() {
        let classes: Vec<(&str, &[RateMethod])> = all()
            .iter()
            .map(|class| (class.name, class.methods))
            .chain(pseudo_all().iter().map(|class| (class.name, class.methods)))
            .collect();
        let mut methods: Vec<(&str, &str)> = classes
            .iter()
            .flat_map(|&(class, methods)| {
                methods
                    .iter()
                    .filter(|method| method.addable == Addable::Pseudo)
                    .map(move |method| (class, method.selector))
            })
            .collect();
        methods.sort_unstable();
        let mut bodies: Vec<(&str, &str)> = BODIES
            .iter()
            .map(|&(class, selector, _)| (class, selector))
            .collect();
        bodies.sort_unstable();
        assert_eq!(methods, bodies);
    }
}
