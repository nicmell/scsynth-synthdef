//! The golden files of `tests/fixtures`, as sclang 3.14.1 wrote them: each
//! one reads, and writes back to its own bytes.

mod common;

use std::fs;

use scsynth_synthdef::{Version, read_def_file, write_def_file, write_def_file_old};

#[test]
fn every_golden_file_writes_back_to_its_bytes() {
    let files = common::golden_files();
    assert_eq!(files.len(), 89, "{files:?}");
    for path in files {
        let bytes = fs::read(&path).expect("a golden file reads");
        let file = read_def_file(&bytes).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        let written = match file.version {
            Version::V2 => write_def_file(&file.synth_defs),
            Version::V1 => write_def_file_old(&file.synth_defs),
            Version::V0 => panic!("{path:?}: sclang writes no version 0"),
        };
        assert_eq!(written.as_deref(), Ok(&bytes[..]), "{path:?}");
    }
}

#[test]
fn only_the_file_of_synth_def_old_has_version_1() {
    for path in common::golden_files() {
        let file = read_def_file(&fs::read(&path).unwrap()).unwrap();
        let expected = if path.ends_with("old.scsyndef") {
            Version::V1
        } else {
            Version::V2
        };
        assert_eq!(file.version, expected, "{path:?}");
    }
}
