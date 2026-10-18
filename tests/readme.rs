//! The README lists the methods that `ugen` refuses: exactly those that the
//! catalogue marks as not addable.

use std::collections::BTreeSet;

use scsynth_synthdef::catalog::{self, Addable};

const README: &str = include_str!("../README.md");
const HEADING: &str = "### Methods that `ugen` refuses";

fn listed() -> BTreeSet<String> {
    let start = README.find(HEADING).expect("the README lists the methods");
    let section = &README[start + HEADING.len()..];
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

fn refused() -> BTreeSet<String> {
    catalog::all()
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

#[test]
fn the_readme_lists_every_method_that_ugen_refuses() {
    let (listed, refused) = (listed(), refused());
    let extra: Vec<&String> = listed.difference(&refused).collect();
    let missing: Vec<&String> = refused.difference(&listed).collect();
    assert!(
        extra.is_empty() && missing.is_empty(),
        "listed but addable: {extra:?}; refused but not listed: {missing:?}"
    );
    assert_eq!(refused.len(), 104);
}
