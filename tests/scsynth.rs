//! The definitions of the crate, loaded by scsynth 3.14.1. scsynth runs in NRT
//! mode: it opens no audio device and no port, and its output is complete
//! when it exits. The tests are ignored by default:
//!
//! ```sh
//! cargo test --test scsynth -- --ignored
//! ```
//!
//! `SCSYNTH` names the scsynth to run, and `SCSYNTH_PLUGINS` its folder of
//! plugins. Both default to the bundle of `/Applications/SuperCollider.app`.

mod common;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use scsynth_synthdef::{
    Arg, NewUGen, Rate, SynthDef, SynthDefBuilder, read_def_file, write_def_file,
};

const BUNDLE: &str = "/Applications/SuperCollider.app/Contents/Resources";
const EXCEPTION: &str = "exception in GraphDef_Recv";
const TOO_LONG: &str = "exception in GraphDef_Recv: name too long (> 31 chars)";
const EXCEEDED: &str = "exception in GraphDef_Recv: exceeded number of interconnect buffers.";

static RUNS: AtomicUsize = AtomicUsize::new(0);

fn bundle_path(variable: &str, name: &str) -> PathBuf {
    env::var_os(variable).map_or_else(|| PathBuf::from(BUNDLE).join(name), PathBuf::from)
}

fn pad(bytes: &mut Vec<u8>) {
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
}

fn osc_string(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend_from_slice(text.as_bytes());
    bytes.push(0);
    pad(bytes);
}

/// An OSC message whose arguments are blobs or ints.
fn message(address: &str, blobs: &[&[u8]], ints: &[i32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    osc_string(&mut bytes, address);
    let tags = format!(",{}{}", "b".repeat(blobs.len()), "i".repeat(ints.len()));
    osc_string(&mut bytes, &tags);
    for blob in blobs {
        let len = i32::try_from(blob.len()).expect("a blob fits an OSC size");
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(blob);
        pad(&mut bytes);
    }
    for int in ints {
        bytes.extend_from_slice(&int.to_be_bytes());
    }
    bytes
}

/// One bundle of a score of NRT mode: its size, then the bundle.
fn bundle(seconds: u32, message: &[u8]) -> Vec<u8> {
    let mut bundle = Vec::new();
    osc_string(&mut bundle, "#bundle");
    bundle.extend_from_slice(&seconds.to_be_bytes());
    bundle.extend_from_slice(&0_u32.to_be_bytes());
    let len = i32::try_from(message.len()).expect("a message fits an OSC size");
    bundle.extend_from_slice(&len.to_be_bytes());
    bundle.extend_from_slice(message);
    let mut entry = i32::try_from(bundle.len())
        .expect("a bundle fits an OSC size")
        .to_be_bytes()
        .to_vec();
    entry.extend_from_slice(&bundle);
    entry
}

/// `/s_new name id 0 0`: a synth of the definition `name`, at the head of the
/// root group.
fn s_new(name: &str, id: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    osc_string(&mut bytes, "/s_new");
    osc_string(&mut bytes, ",siii");
    osc_string(&mut bytes, name);
    for int in [id, 0, 0] {
        bytes.extend_from_slice(&int.to_be_bytes());
    }
    bytes
}

/// The output of scsynth in NRT mode, on a score that sends each file with
/// `/d_recv` and ends after one second.
fn load(files: &[Vec<u8>]) -> String {
    run(files, &[], None)
}

/// The output of scsynth in NRT mode with `-w wire_buffers`, on a score that
/// sends each file with `/d_recv`, then each message of `after`, and ends
/// after one second.
fn load_at(files: &[Vec<u8>], wire_buffers: usize, after: &[Vec<u8>]) -> String {
    run(files, after, Some(wire_buffers))
}

fn run(files: &[Vec<u8>], after: &[Vec<u8>], wire_buffers: Option<usize>) -> String {
    let dir = env::temp_dir().join(format!(
        "scsynth-synthdef-{}-{}",
        std::process::id(),
        RUNS.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).expect("the temporary folder exists");
    let mut score = Vec::new();
    for file in files {
        score.extend(bundle(0, &message("/d_recv", &[file], &[])));
    }
    for later in after {
        score.extend(bundle(0, later));
    }
    score.extend(bundle(1, &message("/c_set", &[], &[0, 0])));
    let score_path = dir.join("score.osc");
    fs::write(&score_path, score).expect("the score writes");
    let output = Command::new(bundle_path("SCSYNTH", "scsynth"))
        .arg("-D")
        .arg("0")
        .arg("-U")
        .arg(bundle_path("SCSYNTH_PLUGINS", "plugins"))
        .arg("-N")
        .arg(&score_path)
        .arg("_")
        .arg(dir.join("out.aiff"))
        .args(["44100", "AIFF", "int16", "-o", "1"])
        .args(
            wire_buffers
                .map(|count| ["-w".to_owned(), count.to_string()])
                .into_iter()
                .flatten(),
        )
        .output()
        .expect("scsynth runs");
    fs::remove_dir_all(&dir).expect("the temporary folder goes");
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "scsynth failed: {text}");
    text
}

/// `bytes` with the pstring `from` replaced by `to`.
fn renamed(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    let mut pattern = vec![u8::try_from(from.len()).unwrap()];
    pattern.extend_from_slice(from.as_bytes());
    let mut replacement = vec![u8::try_from(to.len()).unwrap()];
    replacement.extend_from_slice(to.as_bytes());
    let start = bytes
        .windows(pattern.len())
        .position(|window| window == pattern)
        .expect("the bytes hold the name");
    [
        &bytes[..start],
        &replacement,
        &bytes[start + pattern.len()..],
    ]
    .concat()
}

/// `Out.kr(0, x)` with the parameter `parameter` and the variant `variant`.
fn definition(name: &str, parameter: &str, variant: &str) -> SynthDef {
    let mut builder = SynthDefBuilder::new(name, vec![Arg::new(parameter, 1.0)]).unwrap();
    let control = builder.arguments()[0].clone();
    builder
        .add(
            NewUGen::new("Out", Rate::Control)
                .inputs([0.0.into(), control])
                .outputs([]),
        )
        .unwrap();
    builder.variant(variant, &[(parameter, vec![2.0])]).unwrap();
    builder.finish().unwrap()
}

#[test]
#[ignore = "runs scsynth 3.14.1"]
fn scsynth_loads_every_golden_graph() {
    let mut files: Vec<Vec<u8>> = common::NAMES
        .iter()
        .map(|name| common::written(name))
        .collect();
    // A last file that scsynth refuses shows that it read the whole score.
    let short = "p".repeat(31);
    let bytes = definition("last", &short, "v").as_bytes().unwrap();
    files.push(renamed(&bytes, &short, &"p".repeat(32)));
    let output = load(&files);
    assert_eq!(output.matches(EXCEPTION).count(), 1, "{output}");
    assert!(output.contains(TOO_LONG), "{output}");
}

#[test]
#[ignore = "runs scsynth 3.14.1"]
fn scsynth_reads_a_parameter_name_of_31_bytes_and_not_of_32() {
    // Fact 8 of the plan of 0.1.
    let short = "p".repeat(31);
    let bytes = definition("parameter", &short, "v").as_bytes().unwrap();
    assert!(!load(std::slice::from_ref(&bytes)).contains(EXCEPTION));
    let long = renamed(&bytes, &short, &"p".repeat(32));
    assert!(load(&[long]).contains(TOO_LONG));
}

#[test]
#[ignore = "runs scsynth 3.14.1"]
fn scsynth_reads_a_variant_name_of_31_bytes_and_not_of_32() {
    // Facts 24 and 38 of the plan of 0.1: the name holds the definition name.
    let short = format!("v.{}", "w".repeat(29));
    let bytes = definition("v", "x", &short[2..]).as_bytes().unwrap();
    assert!(!load(std::slice::from_ref(&bytes)).contains(EXCEPTION));
    let long = renamed(&bytes, &short, &format!("v.{}", "w".repeat(30)));
    assert!(load(&[long]).contains(TOO_LONG));
}

#[test]
#[ignore = "runs scsynth 3.14.1"]
fn scsynth_reads_a_ugen_name_of_31_bytes_and_not_of_32() {
    // Facts 8 and 9 of the plan of 0.1: a name that fits reaches the lookup of the
    // plugin.
    let short = "u".repeat(31);
    let mut builder = SynthDefBuilder::new("ugen", vec![]).unwrap();
    builder
        .add(NewUGen::new(short.as_str(), Rate::Control))
        .unwrap();
    let bytes = builder.finish().unwrap().as_bytes().unwrap();
    assert!(load(std::slice::from_ref(&bytes)).contains(&format!("UGen '{short}' not installed.")));
    let long = renamed(&bytes, &short, &"u".repeat(32));
    assert!(load(&[long]).contains(TOO_LONG));
}

#[test]
#[ignore = "runs scsynth 3.14.1"]
fn scsynth_needs_the_wire_buffers_that_the_crate_counts() {
    // Facts 1 to 5 of the plan of 0.10.
    let mut by_count: BTreeMap<usize, Vec<Vec<u8>>> = BTreeMap::new();
    for path in common::golden_files() {
        let file = read_def_file(&fs::read(&path).unwrap()).unwrap();
        for def in file.synth_defs {
            let count = def.wire_buffers().unwrap();
            by_count
                .entry(count)
                .or_default()
                .push(def.as_bytes().unwrap());
        }
    }
    for (&count, files) in &by_count {
        let loaded = load_at(files, count, &[]);
        assert!(!loaded.contains(EXCEPTION), "-w {count}: {loaded}");
        if count > 0 {
            let refused = load_at(files, count - 1, &[]);
            let exceeded = refused.matches(EXCEEDED).count();
            assert_eq!(exceeded, files.len(), "-w {}: {refused}", count - 1);
        }
    }
    if env::var_os("SCSYNTH_SYNTHDEF_BLESS").is_some() {
        let counts = serde_json::to_string_pretty(&common::golden_wire_buffers()).unwrap();
        fs::write(common::wire_buffers_fixture(), counts + "\n").expect("the fixture writes");
    }
}

#[test]
#[ignore = "runs scsynth 3.14.1"]
fn scsynth_drops_every_definition_of_a_file_with_one_over_its_w() {
    // Fact 5 of the plan of 0.10.
    let defs = read_def_file(&common::golden("wirebufs"))
        .unwrap()
        .synth_defs;
    let names = ["wirebufs_chain", "wirebufs_sixteen"];
    let picked: Vec<SynthDef> = names
        .iter()
        .map(|name| defs.iter().find(|def| def.name == *name).unwrap().clone())
        .collect();
    let counts: Vec<usize> = picked
        .iter()
        .map(|def| def.wire_buffers().unwrap())
        .collect();
    assert_eq!(counts, [1, 16]);
    let file = write_def_file(&picked).unwrap();
    let synths = [s_new(names[0], 1000), s_new(names[1], 1001)];
    let missing =
        |output: &str| names.map(|name| output.contains(&format!("SynthDef {name} not found")));
    let loaded = load_at(std::slice::from_ref(&file), 16, &synths);
    assert_eq!(missing(&loaded), [false, false], "{loaded}");
    let refused = load_at(&[file], 8, &synths);
    assert_eq!(refused.matches(EXCEEDED).count(), 1, "{refused}");
    assert_eq!(missing(&refused), [true, true], "{refused}");
}
