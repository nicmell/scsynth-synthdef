//! The operators of `UnaryOpUGen` and `BinaryOpUGen`, by name and by special
//! index, as `initSpecialSelectors` and `Opcodes.h` of sclang 3.14.1 give
//! them.

#[rustfmt::skip]
mod generated;

/// The special index of the unary operator `name`, such as 0 for `neg`.
#[must_use]
pub fn unary_index(name: &str) -> Option<i16> {
    index_of(&generated::UNARY, name)
}

/// The name of the unary operator with a special index.
#[must_use]
pub fn unary_name(index: i16) -> Option<&'static str> {
    name_of(&generated::UNARY, index)
}

/// The special index of the binary operator `name`, such as 2 for `*`.
#[must_use]
pub fn binary_index(name: &str) -> Option<i16> {
    index_of(&generated::BINARY, name)
}

/// The name of the binary operator with a special index.
#[must_use]
pub fn binary_name(index: i16) -> Option<&'static str> {
    name_of(&generated::BINARY, index)
}

fn index_of(table: &[&str], name: &str) -> Option<i16> {
    let position = table.iter().position(|&operator| operator == name)?;
    i16::try_from(position).ok()
}

fn name_of(table: &[&'static str], index: i16) -> Option<&'static str> {
    table.get(usize::try_from(index).ok()?).copied()
}

#[cfg(test)]
mod tests {
    use super::{binary_index, binary_name, unary_index, unary_name};

    #[test]
    fn the_special_indices_are_those_that_sclang_measured() {
        // Fact 25 of the plan.
        assert_eq!(binary_index("+"), Some(0));
        assert_eq!(binary_index("*"), Some(2));
        assert_eq!(unary_index("neg"), Some(0));
        assert_eq!(unary_index("midicps"), Some(17));
    }

    #[test]
    fn a_name_and_its_index_lead_to_each_other() {
        assert_eq!(binary_name(25), Some("pow"));
        assert_eq!(unary_name(5), Some("abs"));
        assert_eq!(binary_name(-1), None);
        assert_eq!(unary_name(i16::MAX), None);
        assert_eq!(unary_index("+"), None);
        assert_eq!(binary_index("neg"), None);
    }
}
