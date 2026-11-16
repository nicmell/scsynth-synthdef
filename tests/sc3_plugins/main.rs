//! The golden graphs of sc3-plugins in `tests/fixtures/sc3_plugins`, as
//! `generator/sc3_plugins/fixtures.scd` built them with sclang 3.14.1 and the
//! classes of sc3-plugins 3.14.0: the crate writes the bytes of sclang.

#[path = "../common/mod.rs"]
mod common;

mod cases;
mod methods;
mod scsynth;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use scsynth_synthdef::{read_def_file, write_def_file};

/// The folder of the golden files of sc3-plugins.
const FIXTURES: &str = "tests/fixtures/sc3_plugins";

/// The bytes of a golden file of sc3-plugins.
fn golden(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(format!("{name}.scsyndef"));
    fs::read(path).expect("the golden file reads")
}

/// The file of the counts of wire buffers that scsynth 3.14.1 confirmed.
fn wire_buffers_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join("wire_buffers.json")
}

#[test]
fn every_golden_file_of_sc3_plugins_writes_back_to_its_bytes() {
    let files = common::golden_files_in(FIXTURES);
    assert_eq!(files.len(), 4);
    for path in files {
        let bytes = fs::read(&path).expect("a golden file reads");
        let file = read_def_file(&bytes).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        assert_eq!(
            write_def_file(&file.synth_defs).as_deref(),
            Ok(&bytes[..]),
            "{path:?}"
        );
    }
}

#[test]
fn each_golden_definition_of_sc3_plugins_needs_the_wire_buffers_of_the_fixture() {
    let text = fs::read_to_string(wire_buffers_fixture()).expect("the fixture reads");
    let fixture: BTreeMap<String, BTreeMap<String, usize>> =
        serde_json::from_str(&text).expect("the fixture is JSON");
    let counts = common::wire_buffers_of(&common::golden_files_in(FIXTURES));
    assert_eq!(counts, fixture);
}
