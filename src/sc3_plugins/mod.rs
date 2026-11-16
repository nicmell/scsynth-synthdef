//! The catalogue of the UGens of sc3-plugins 3.14.0, the plugins that most
//! installations of SuperCollider add to the core: what each class takes and
//! gives, as `generator/sc3_plugins/catalog.scd` measured it in sclang
//! 3.14.1. [`SynthDefBuilder::ugen`](crate::SynthDefBuilder::ugen) adds these
//! classes as it adds those of [`catalog`](crate::catalog).

#[rustfmt::skip]
mod generated;
mod pseudo;

pub(crate) use pseudo::BODIES;

use crate::catalog::{PseudoClass, UGenClass, by_name};

/// The class with this name.
#[must_use]
pub fn lookup(name: &str) -> Option<&'static UGenClass> {
    by_name(all(), name, |class| class.name)
}

/// All the classes, sorted by name.
#[must_use]
pub fn all() -> &'static [UGenClass] {
    &generated::CLASSES
}

/// The pseudo-UGen class outside `UGen` with this name, such as `JPverb`.
#[must_use]
pub fn pseudo_lookup(name: &str) -> Option<&'static PseudoClass> {
    by_name(pseudo_all(), name, |class| class.name)
}

/// All the pseudo-UGen classes outside `UGen` that the crate builds, sorted
/// by name.
#[must_use]
pub fn pseudo_all() -> &'static [PseudoClass] {
    &generated::PSEUDO_CLASSES
}

#[cfg(test)]
mod tests {
    use super::{all, lookup, pseudo_all};
    use crate::catalog::{
        self, Addable, ArgDefault, ArrayExpansion, CheckInputs, FftSize, InputSlot, Outputs,
        SignalRange,
    };
    use crate::rate::Rate;

    const NO_UNIT: &str = "no plugin of sc3-plugins 3.14.0 defines the unit";

    #[test]
    fn the_catalogue_holds_the_classes_of_sc3_plugins_sorted_by_name() {
        // Facts 3, 4 and 7 of the plan of 0.14.
        assert_eq!(all().len(), 491);
        assert!(all().windows(2).all(|pair| pair[0].name < pair[1].name));
        assert!(lookup("DFM1").is_some());
        let mut counts = [0; 3];
        for method in all().iter().flat_map(|class| class.methods) {
            counts[match method.addable {
                Addable::Yes(_) => 0,
                Addable::Pseudo => 1,
                Addable::No(_) => 2,
            }] += 1;
        }
        assert_eq!(counts, [635, 1, 43]);
        let pseudo: Vec<&str> = pseudo_all().iter().map(|class| class.name).collect();
        assert_eq!(pseudo, ["Greyhole", "JPverb"]);
    }

    #[test]
    fn the_names_of_sc3_plugins_are_apart_from_those_of_the_core() {
        let names = all()
            .iter()
            .map(|class| class.name)
            .chain(pseudo_all().iter().map(|class| class.name));
        for name in names {
            assert!(catalog::lookup(name).is_none(), "{name}");
            assert!(catalog::pseudo_lookup(name).is_none(), "{name}");
            assert!(catalog::find(name).is_some() || catalog::find_pseudo(name).is_some());
        }
        assert_eq!(catalog::find("SinOsc"), catalog::lookup("SinOsc"));
    }

    #[test]
    fn seven_classes_have_no_unit() {
        // Fact 7 of the plan of 0.14.
        let without: Vec<&str> = all()
            .iter()
            .filter(|class| {
                class
                    .methods
                    .iter()
                    .any(|method| method.addable == Addable::No(NO_UNIT))
            })
            .map(|class| class.name)
            .collect();
        assert_eq!(
            without,
            [
                "BlitB3D",
                "CQ_Diff",
                "Clipper32",
                "FFTSubbandFlux",
                "RMAFoodChainL",
                "RegaliaMitraEQ",
                "RosslerResL"
            ]
        );
    }

    #[test]
    fn the_units_that_cannot_alias_are_the_176_of_the_plugins() {
        // Fact 8 of the plan of 0.14.
        let flagged = |name| lookup(name).unwrap().cant_alias;
        assert_eq!(all().iter().filter(|class| class.cant_alias).count(), 176);
        for name in [
            "GreyholeRaw",
            "JPverbRaw",
            "HOARotator1",
            "DrumTrack",
            "PanX",
            "VBAP",
        ] {
            assert!(flagged(name), "{name}");
        }
        assert!(!flagged("NHHall"));
        assert!(lookup("CubicDelay").is_none());
    }

    #[test]
    fn each_class_has_the_rule_of_its_check_inputs() {
        // Fact 5 of the plan of 0.14.
        let rule = |name| lookup(name).unwrap().check_inputs;
        assert_eq!(rule("HOAEncoder1"), CheckInputs::NInputs(1));
        assert_eq!(rule("HOARotator1"), CheckInputs::NInputs(4));
        assert_eq!(rule("HOAMirror10"), CheckInputs::NInputs(121));
        assert_eq!(rule("JPverbRaw"), CheckInputs::NInputs(2));
        assert_eq!(rule("EnvDetect"), CheckInputs::SameRateAsFirstInputIfAudio);
        assert_eq!(rule("Summer"), CheckInputs::SameRateAsFirstInput);
        assert_eq!(rule("FM7"), CheckInputs::Custom("FM7"));
        // BFGrainPanner checks input 2 in a checkNInputs of its own.
        assert_eq!(rule("InGrainIBF"), CheckInputs::AudioAt(2));
        let count = |wanted: fn(CheckInputs) -> bool| {
            all()
                .iter()
                .filter(|class| wanted(class.check_inputs))
                .count()
        };
        assert_eq!(count(|rule| matches!(rule, CheckInputs::Valid)), 355);
        assert_eq!(count(|rule| matches!(rule, CheckInputs::NInputs(_))), 113);
        assert_eq!(count(|rule| matches!(rule, CheckInputs::AudioAt(_))), 4);
    }

    #[test]
    fn the_other_facts_of_the_classes_are_those_of_sclang() {
        // Fact 6 of the plan of 0.14.
        let count = |wanted: fn(&catalog::UGenClass) -> bool| {
            all().iter().filter(|class| wanted(class)).count()
        };
        assert_eq!(count(|class| class.is_pure), 10);
        assert_eq!(count(|class| class.is_width_first), 33);
        assert_eq!(count(|class| class.fft_size == Some(FftSize::Chain)), 33);
        let unipolar: Vec<&str> = all()
            .iter()
            .filter(|class| class.signal_range == SignalRange::Unipolar)
            .map(|class| class.name)
            .collect();
        assert_eq!(unipolar, ["GaussTrig"]);
    }

    #[test]
    fn ladspa_takes_its_count_and_a_variable_list_of_arguments() {
        // Fact 4 of the plan of 0.14: the probe spreads the list.
        let method = lookup("LADSPA").unwrap().method("ar").unwrap();
        assert_eq!(method.args[2].default, Some(ArgDefault::Floats(&[])));
        let Addable::Yes(wiring) = method.addable else {
            panic!("LADSPA.ar is not addable");
        };
        assert_eq!(
            wiring.inputs,
            [InputSlot::Arg(0), InputSlot::Arg(1), InputSlot::ArgArray(2)]
        );
        assert_eq!(
            wiring.outputs,
            Outputs::Arg {
                arg: 0,
                rate: Rate::Audio
            }
        );
        assert_eq!(wiring.array_expansion, Some(ArrayExpansion::Elements));
    }
}
