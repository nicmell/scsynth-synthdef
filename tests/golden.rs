//! The golden files of `tests/fixtures`, as sclang 3.14.1 wrote them: each
//! one reads, and writes back to its own bytes.

mod common;

use std::fs;

use scsynth_synthdef::{Version, read_def_file, write_def_file, write_def_file_old};

#[test]
fn every_golden_file_writes_back_to_its_bytes_and_only_old_has_version_1() {
    for path in common::golden_files() {
        let bytes = fs::read(&path).expect("a golden file reads");
        let file = read_def_file(&bytes).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        let old = path.ends_with("old.scsyndef");
        assert_eq!(
            file.version,
            if old { Version::V1 } else { Version::V2 },
            "{path:?}"
        );
        let written = if old {
            write_def_file_old(&file.synth_defs)
        } else {
            write_def_file(&file.synth_defs)
        };
        assert_eq!(written.as_deref(), Ok(&bytes[..]), "{path:?}");
    }
}
