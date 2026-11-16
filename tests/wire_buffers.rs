//! The count of wire buffers of each golden definition, against
//! `tests/fixtures/wire_buffers.json`. The acceptance test of scsynth writes
//! that file with `SCSYNTH_SYNTHDEF_BLESS=1`, once scsynth 3.14.1 loads each
//! definition at its count and refuses it at one less.

mod common;

use std::collections::BTreeMap;
use std::fs;

#[test]
fn each_golden_definition_needs_the_wire_buffers_of_the_fixture() {
    let text = fs::read_to_string(common::wire_buffers_fixture()).expect("the fixture reads");
    let fixture: BTreeMap<String, BTreeMap<String, usize>> =
        serde_json::from_str(&text).expect("the fixture is JSON");
    assert_eq!(common::golden_wire_buffers(), fixture);
}
