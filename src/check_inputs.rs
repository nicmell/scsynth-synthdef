//! The rules of `checkInputs`, which sclang runs on each UGen at the end of a
//! build.

use crate::catalog::CheckInputs;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// Whether the inputs of a UGen at `rate` meet `rule`, or why not. NaN is
/// the part of `checkValidInputs` that `SynthDefBuilder::add` checks.
pub(crate) fn check(rule: CheckInputs, rate: Rate, inputs: &[UGenInput]) -> Result<(), String> {
    let rate_of = |input: usize| inputs.get(input).and_then(UGenInput::rate);
    let audio = |input: usize| {
        if rate_of(input) == Some(Rate::Audio) {
            Ok(())
        } else {
            Err(format!("input {input} is not at audio rate"))
        }
    };
    let same_rate_as_first = || {
        if rate_of(0) == Some(rate) {
            Ok(())
        } else {
            Err(format!("the first input is not at the rate {rate:?}"))
        }
    };
    match rule {
        CheckInputs::Peak if rate == Rate::Control && rate_of(0) == Some(Rate::Audio) => Ok(()),
        CheckInputs::SameRateAsFirstInput | CheckInputs::Peak => same_rate_as_first(),
        CheckInputs::SameRateAsFirstInputIfAudio if rate == Rate::Audio => audio(0),
        CheckInputs::AudioAt(input) if rate == Rate::Audio => audio(input),
        CheckInputs::NInputs(count) if rate == Rate::Audio => (0..count).try_for_each(audio),
        CheckInputs::AudioFrom(first) if rate == Rate::Audio => {
            (first..inputs.len()).try_for_each(audio)
        }
        CheckInputs::BufWr if rate == Rate::Audio => {
            audio(1)?;
            (3..inputs.len()).try_for_each(audio)
        }
        CheckInputs::Duty if rate_of(0) == Some(Rate::Demand) => match rate_of(1) {
            Some(Rate::Demand | Rate::Scalar) => Ok(()),
            reset if reset == Some(rate) => Ok(()),
            _ => Err(format!(
                "the reset is not at demand rate, at scalar rate or at the rate {rate:?}"
            )),
        },
        CheckInputs::T2K => audio(0),
        CheckInputs::Valid
        | CheckInputs::SameRateAsFirstInputIfAudio
        | CheckInputs::AudioAt(_)
        | CheckInputs::NInputs(_)
        | CheckInputs::AudioFrom(_)
        | CheckInputs::BufWr
        | CheckInputs::Duty => Ok(()),
        CheckInputs::AbstractOut { fixed } => {
            if rate == Rate::Audio {
                (fixed..inputs.len()).try_for_each(audio)
            } else if inputs.len() <= fixed {
                Err("no channel follows the fixed inputs".to_owned())
            } else {
                Ok(())
            }
        }
        CheckInputs::Custom(owner) => {
            Err(format!("{owner} checks its inputs with a rule of its own"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::catalog::CheckInputs;
    use crate::rate::Rate;
    use crate::ugen_input::{OutputProxy, UGenInput};

    fn output(rate: Rate) -> UGenInput {
        UGenInput::Output(OutputProxy {
            build: 0,
            ugen: 0,
            output: 0,
            rate,
        })
    }

    #[test]
    fn valid_takes_any_input() {
        assert_eq!(
            check(CheckInputs::Valid, Rate::Audio, &[0.5.into()]),
            Ok(())
        );
    }

    #[test]
    fn the_first_input_has_the_rate_of_the_ugen() {
        let rule = CheckInputs::SameRateAsFirstInput;
        assert_eq!(check(rule, Rate::Audio, &[output(Rate::Audio)]), Ok(()));
        assert!(check(rule, Rate::Audio, &[output(Rate::Control)]).is_err());
        assert!(check(rule, Rate::Control, &[0.5.into()]).is_err());
        assert!(check(rule, Rate::Control, &[]).is_err());
        let rule = CheckInputs::SameRateAsFirstInputIfAudio;
        assert_eq!(check(rule, Rate::Control, &[0.5.into()]), Ok(()));
        assert!(check(rule, Rate::Audio, &[0.5.into()]).is_err());
    }

    #[test]
    fn n_inputs_and_audio_at_count_only_at_audio_rate() {
        let inputs = [output(Rate::Audio), output(Rate::Control)];
        assert_eq!(check(CheckInputs::NInputs(1), Rate::Audio, &inputs), Ok(()));
        assert!(check(CheckInputs::NInputs(2), Rate::Audio, &inputs).is_err());
        assert_eq!(
            check(CheckInputs::NInputs(2), Rate::Control, &inputs),
            Ok(())
        );
        assert!(check(CheckInputs::AudioAt(1), Rate::Audio, &inputs).is_err());
        assert_eq!(check(CheckInputs::AudioAt(0), Rate::Audio, &inputs), Ok(()));
        assert!(check(CheckInputs::AudioFrom(1), Rate::Audio, &inputs).is_err());
    }

    #[test]
    fn an_out_at_audio_rate_takes_only_audio_channels() {
        // Out.ar(0, 0.5), which sclang refuses, fact 34 of the plan of 0.1.
        let rule = CheckInputs::AbstractOut { fixed: 1 };
        assert_eq!(
            check(rule, Rate::Audio, &[0.0.into(), 0.5.into()]),
            Err("input 1 is not at audio rate".to_owned())
        );
        assert_eq!(
            check(rule, Rate::Audio, &[0.0.into(), output(Rate::Audio)]),
            Ok(())
        );
        assert_eq!(
            check(rule, Rate::Control, &[0.0.into(), 0.5.into()]),
            Ok(())
        );
        assert!(check(rule, Rate::Control, &[0.0.into()]).is_err());
    }

    #[test]
    fn buf_wr_takes_an_audio_phase_and_audio_channels_at_audio_rate() {
        // Fact 1 of the plan of 0.5.
        let inputs =
            |phase: Rate, channel: Rate| [0.0.into(), output(phase), 1.0.into(), output(channel)];
        let rule = CheckInputs::BufWr;
        assert_eq!(
            check(rule, Rate::Audio, &inputs(Rate::Audio, Rate::Audio)),
            Ok(())
        );
        assert_eq!(
            check(rule, Rate::Audio, &inputs(Rate::Control, Rate::Audio)),
            Err("input 1 is not at audio rate".to_owned())
        );
        assert_eq!(
            check(rule, Rate::Audio, &inputs(Rate::Audio, Rate::Control)),
            Err("input 3 is not at audio rate".to_owned())
        );
        assert_eq!(
            check(rule, Rate::Control, &inputs(Rate::Control, Rate::Control)),
            Ok(())
        );
    }

    #[test]
    fn duty_takes_a_reset_at_demand_scalar_or_its_rate_after_a_demand_dur() {
        // Fact 2 of the plan of 0.5.
        let duty = |rate: Rate, reset: UGenInput| {
            check(CheckInputs::Duty, rate, &[output(Rate::Demand), reset])
        };
        assert_eq!(duty(Rate::Control, output(Rate::Demand)), Ok(()));
        assert_eq!(duty(Rate::Control, 0.0.into()), Ok(()));
        assert_eq!(duty(Rate::Control, output(Rate::Control)), Ok(()));
        assert!(duty(Rate::Control, output(Rate::Audio)).is_err());
        assert!(duty(Rate::Audio, output(Rate::Control)).is_err());
        let number = [0.5.into(), output(Rate::Audio)];
        assert_eq!(check(CheckInputs::Duty, Rate::Control, &number), Ok(()));
    }

    #[test]
    fn peak_at_control_rate_takes_an_audio_input_and_t2k_takes_only_one() {
        // Facts 3 and 4 of the plan of 0.5.
        let peak = |rate: Rate, input: Rate| check(CheckInputs::Peak, rate, &[output(input)]);
        assert_eq!(peak(Rate::Control, Rate::Audio), Ok(()));
        assert_eq!(peak(Rate::Control, Rate::Control), Ok(()));
        assert_eq!(peak(Rate::Audio, Rate::Audio), Ok(()));
        assert!(peak(Rate::Audio, Rate::Control).is_err());
        let t2k = |input: Rate| check(CheckInputs::T2K, Rate::Control, &[output(input)]);
        assert_eq!(t2k(Rate::Audio), Ok(()));
        assert_eq!(
            t2k(Rate::Control),
            Err("input 0 is not at audio rate".to_owned())
        );
    }

    #[test]
    fn a_custom_rule_is_refused() {
        assert!(check(CheckInputs::Custom("Duty"), Rate::Audio, &[]).is_err());
    }
}
