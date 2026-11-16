//! The calculation rates of a UGen and of its outputs.

/// The rate at which a UGen, or one of its outputs, computes.
///
/// Rates compare in the order of their numbers: scalar, control, audio,
/// demand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Rate {
    /// One value, computed when the synth starts. Rate number 0.
    Scalar,
    /// One value per control period. Rate number 1.
    Control,
    /// One value per sample. Rate number 2.
    Audio,
    /// One value each time a demand UGen asks for one. Rate number 3.
    Demand,
}

impl Rate {
    /// The number that a definition file stores for the rate, as
    /// `UGen:rateNumber` gives it.
    #[must_use]
    pub const fn rate_number(self) -> i8 {
        match self {
            Self::Scalar => 0,
            Self::Control => 1,
            Self::Audio => 2,
            Self::Demand => 3,
        }
    }

    /// The rate of a rate number, or `None` for a number other than 0 to 3.
    #[must_use]
    pub const fn from_rate_number(number: i8) -> Option<Self> {
        match number {
            0 => Some(Self::Scalar),
            1 => Some(Self::Control),
            2 => Some(Self::Audio),
            3 => Some(Self::Demand),
            _ => None,
        }
    }
}

/// The least of `rates` by the name of its symbol, as `minItem` takes it in
/// `SequenceableCollection:rate`: audio, control, demand, scalar. Demand
/// ranks after control, unlike the order of [`Rate`].
pub(crate) fn least_by_name(rates: impl IntoIterator<Item = Rate>) -> Option<Rate> {
    rates.into_iter().min_by_key(|&rate| name_rank(rate))
}

/// `insertionSortRange` of `sort { |a, b| a.rate < b.rate }`, which moves an
/// item left past every item whose rate name is not less than its own. The
/// stable sort of Rust gives another order for items of one rate.
pub(crate) fn sort_by_rate<T>(items: &mut [T], rate: impl Fn(&T) -> Rate) {
    for i in 1..items.len() {
        let rank = name_rank(rate(&items[i]));
        let mut j = i;
        while j > 0 && name_rank(rate(&items[j - 1])) >= rank {
            j -= 1;
        }
        items[j..=i].rotate_right(1);
    }
}

fn name_rank(rate: Rate) -> u8 {
    match rate {
        Rate::Audio => 0,
        Rate::Control => 1,
        Rate::Demand => 2,
        Rate::Scalar => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::{Rate, least_by_name};

    #[test]
    fn each_rate_has_the_number_of_sclang() {
        let rates = [Rate::Scalar, Rate::Control, Rate::Audio, Rate::Demand];
        for (number, rate) in (0..).zip(rates) {
            assert_eq!(rate.rate_number(), number);
            assert_eq!(Rate::from_rate_number(number), Some(rate));
        }
    }

    #[test]
    fn the_least_rate_by_name_puts_demand_after_control() {
        assert_eq!(
            least_by_name([Rate::Scalar, Rate::Demand, Rate::Control]),
            Some(Rate::Control)
        );
        assert_eq!(
            least_by_name([Rate::Demand, Rate::Scalar]),
            Some(Rate::Demand)
        );
        assert_eq!(least_by_name([]), None);
    }

    #[test]
    fn other_numbers_are_no_rate() {
        for number in [-1, 4, i8::MIN, i8::MAX] {
            assert_eq!(Rate::from_rate_number(number), None);
        }
    }
}
