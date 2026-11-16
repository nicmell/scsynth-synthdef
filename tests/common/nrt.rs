//! scsynth 3.14.1 in NRT mode, for the acceptance tests: it opens no audio
//! device and no port, and its output is complete when it exits.
//!
//! `SCSYNTH` names the scsynth to run, and `SCSYNTH_PLUGINS` its folder of
//! plugins. Both default to the bundle of `/Applications/SuperCollider.app`.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BUNDLE: &str = "/Applications/SuperCollider.app/Contents/Resources";
pub const EXCEPTION: &str = "exception in GraphDef_Recv";
pub const TOO_LONG: &str = "exception in GraphDef_Recv: name too long (> 31 chars)";
pub const EXCEEDED: &str = "exception in GraphDef_Recv: exceeded number of interconnect buffers.";

static RUNS: AtomicUsize = AtomicUsize::new(0);

fn bundle_path(variable: &str, name: &str) -> PathBuf {
    env::var_os(variable).map_or_else(|| PathBuf::from(BUNDLE).join(name), PathBuf::from)
}

/// The folder of the plugins of the bundle. With `-U`, scsynth loads no
/// other plugin, such as those of the Extensions folder.
pub fn plugins() -> OsString {
    bundle_path("SCSYNTH_PLUGINS", "plugins").into_os_string()
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
pub fn s_new(name: &str, id: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    osc_string(&mut bytes, "/s_new");
    osc_string(&mut bytes, ",siii");
    osc_string(&mut bytes, name);
    for int in [id, 0, 0] {
        bytes.extend_from_slice(&int.to_be_bytes());
    }
    bytes
}

/// The output of scsynth in NRT mode with `-U plugins`, and with `-w
/// wire_buffers` if there are some. The score sends each file with
/// `/d_recv`, then each message of `after`, and ends after one second.
pub fn run(
    plugins: &OsStr,
    files: &[Vec<u8>],
    after: &[Vec<u8>],
    wire_buffers: Option<usize>,
) -> String {
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
        .arg(plugins)
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
