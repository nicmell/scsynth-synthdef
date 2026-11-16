//! The golden definitions of sc3-plugins, loaded by scsynth 3.14.1 in NRT
//! mode with the plugins of sc3-plugins, as `common/nrt.rs` runs it. The
//! tests are ignored by default:
//!
//! ```sh
//! cargo test --features sc3-plugins --test sc3_plugins -- --ignored
//! ```
//!
//! `SCSYNTH_SC3_PLUGINS` names the folder of the plugins of sc3-plugins, by
//! default the folder `SC3plugins` of the Extensions.

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use scsynth_synthdef::catalog::Addable;
use scsynth_synthdef::{NewUGen, Rate, SynthDef, SynthDefBuilder, read_def_file, sc3_plugins};

use super::common::nrt::{self, EXCEEDED, EXCEPTION};
use super::common::{golden_files_in, wire_buffers_of};
use super::{FIXTURES, wire_buffers_fixture};

/// The plugins of the bundle and those of sc3-plugins, for `-U`.
fn plugins() -> OsString {
    let folder = env::var_os("SCSYNTH_SC3_PLUGINS").unwrap_or_else(|| {
        let home = PathBuf::from(env::var_os("HOME").expect("a home folder"));
        home.join("Library/Application Support/SuperCollider/Extensions/SC3plugins")
            .into_os_string()
    });
    let mut plugins = nrt::plugins();
    plugins.push(":");
    plugins.push(folder);
    plugins
}

/// Each definition of the golden files of sc3-plugins.
fn definitions() -> Vec<SynthDef> {
    golden_files_in(FIXTURES)
        .iter()
        .flat_map(|path| read_def_file(&fs::read(path).unwrap()).unwrap().synth_defs)
        .collect()
}

#[test]
#[ignore = "runs scsynth 3.14.1 with sc3-plugins"]
fn scsynth_loads_every_golden_graph_of_sc3_plugins() {
    // Fact 7 of the plan of 0.14: a unit for each class with an addable
    // method.
    // Some definitions of HOA need more than the 64 wire buffers of -w.
    let defs = definitions();
    let most = defs.iter().map(|def| def.wire_buffers().unwrap()).max();
    let files: Vec<Vec<u8>> = defs.iter().map(|def| def.as_bytes().unwrap()).collect();
    let output = nrt::run(&plugins(), &files, &[], most);
    assert!(!output.contains(EXCEPTION), "{output}");
}

#[test]
#[ignore = "runs scsynth 3.14.1 with sc3-plugins"]
fn scsynth_has_no_unit_for_the_seven_classes_that_ugen_refuses() {
    // Fact 7 of the plan of 0.14.
    let refused = |class: &&scsynth_synthdef::catalog::UGenClass| {
        class.methods.iter().any(|method| {
            matches!(method.addable, Addable::No(reason) if reason.contains("defines the unit"))
        })
    };
    let classes: Vec<&str> = sc3_plugins::all()
        .iter()
        .filter(refused)
        .map(|class| class.name)
        .collect();
    assert_eq!(classes.len(), 7);
    for class in classes {
        let mut builder = SynthDefBuilder::new("unit", vec![]).unwrap();
        builder.add(NewUGen::new(class, Rate::Audio)).unwrap();
        let bytes = builder.finish().unwrap().as_bytes().unwrap();
        let output = nrt::run(&plugins(), &[bytes], &[], None);
        assert!(
            output.contains(&format!("UGen '{class}' not installed.")),
            "{output}"
        );
    }
}

#[test]
#[ignore = "runs scsynth 3.14.1 with sc3-plugins"]
fn scsynth_needs_the_wire_buffers_that_the_crate_counts_for_sc3_plugins() {
    // Fact 8 of the plan of 0.14: the units that cannot alias.
    let mut by_count: BTreeMap<usize, Vec<Vec<u8>>> = BTreeMap::new();
    for def in definitions() {
        let count = def.wire_buffers().unwrap();
        by_count
            .entry(count)
            .or_default()
            .push(def.as_bytes().unwrap());
    }
    for (&count, files) in &by_count {
        let loaded = nrt::run(&plugins(), files, &[], Some(count));
        assert!(!loaded.contains(EXCEPTION), "-w {count}: {loaded}");
        if count > 0 {
            let refused = nrt::run(&plugins(), files, &[], Some(count - 1));
            let exceeded = refused.matches(EXCEEDED).count();
            assert_eq!(exceeded, files.len(), "-w {}: {refused}", count - 1);
        }
    }
    if env::var_os("SCSYNTH_SYNTHDEF_BLESS").is_some() {
        let counts = wire_buffers_of(&golden_files_in(FIXTURES));
        let text = serde_json::to_string_pretty(&counts).unwrap();
        fs::write(wire_buffers_fixture(), text + "\n").expect("the fixture writes");
    }
}
