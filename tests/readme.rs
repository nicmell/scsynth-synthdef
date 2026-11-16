//! The README lists the methods that `ugen` refuses: exactly those that the
//! catalogue marks as not addable, and with the feature `sc3-plugins`, those
//! of the catalogue of sc3-plugins.

use std::collections::BTreeSet;

use scsynth_synthdef::catalog::{self, Addable, UGenClass};

const README: &str = include_str!("../README.md");

/// The methods that the section under `heading` lists.
fn listed(heading: &str) -> BTreeSet<String> {
    let start = README.find(heading).expect("the README lists the methods");
    let section = &README[start + heading.len()..];
    let section = &section[..section.find("\n#").unwrap_or(section.len())];
    section
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|token| {
            token.split_once('.').is_some_and(|(class, selector)| {
                !class.is_empty() && matches!(selector, "ar" | "kr" | "ir" | "new")
            })
        })
        .map(str::to_owned)
        .collect()
}

/// The methods of `classes` that are not addable.
fn refused(classes: &[UGenClass]) -> BTreeSet<String> {
    classes
        .iter()
        .flat_map(|class| {
            class
                .methods
                .iter()
                .filter(|method| matches!(method.addable, Addable::No(_)))
                .map(|method| format!("{}.{}", class.name, method.selector))
        })
        .collect()
}

fn assert_listed(heading: &str, classes: &[UGenClass], count: usize) {
    let (listed, refused) = (listed(heading), refused(classes));
    let extra: Vec<&String> = listed.difference(&refused).collect();
    let missing: Vec<&String> = refused.difference(&listed).collect();
    assert!(
        extra.is_empty() && missing.is_empty(),
        "listed but addable: {extra:?}; refused but not listed: {missing:?}"
    );
    assert_eq!(refused.len(), count);
}

#[test]
fn the_readme_lists_every_method_that_ugen_refuses() {
    assert_listed("### Methods that `ugen` refuses", catalog::all(), 18);
}

#[cfg(feature = "sc3-plugins")]
#[test]
fn the_readme_lists_every_method_of_sc3_plugins_that_ugen_refuses() {
    let classes = scsynth_synthdef::sc3_plugins::all();
    assert_listed(
        "### Methods of sc3-plugins that `ugen` refuses",
        classes,
        43,
    );
}
