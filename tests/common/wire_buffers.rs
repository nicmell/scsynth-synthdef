//! The graphs of the golden file of the count of wire buffers, built with
//! `ugen`. `generator/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{NewUGen, Rate, SynthDef, SynthDefBuilder, UGenInput};

use super::pseudo::{def, demand_env_gen, dseq, impulse, input, out, rule_out, sine};
use super::{array, by_name, element};

/// The definitions of `wirebufs`.
pub fn definitions() -> Vec<SynthDef> {
    [rules(), buffers_and_demand(), grains_and_pans()].concat()
}

/// `Duty.ar(0.1, 0, Dseq([sound, 0], inf))` into `Out.ar`.
fn demand_duty(b: &mut SynthDefBuilder, sound: UGenInput) {
    let list = by_name(
        b,
        "Dseq",
        "new",
        &[
            ("list", input(UGenInput::Array(vec![sound, 0.0.into()]))),
            ("repeats", input(f64::INFINITY)),
        ],
    );
    let args = [
        ("dur", input(0.1)),
        ("reset", input(0.0)),
        ("level", input(list)),
    ];
    rule_out(b, "Duty", "ar", &args);
}

fn rules() -> Vec<SynthDef> {
    vec![
        def("wirebufs_chain", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let args = [("in", input(sound)), ("freq", input(1000.0))];
            rule_out(b, "LPF", "ar", &args);
        }),
        def("wirebufs_square", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let square = b.binary_op("*", sound.clone(), sound).unwrap();
            out(b, "ar", square);
        }),
        def("wirebufs_pan2", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            rule_out(b, "Pan2", "ar", &[("in", input(sound))]);
        }),
        def("wirebufs_control", vec![], |b| {
            let sound = sine(b, "kr", 440.0);
            out(b, "kr", sound);
        }),
        def("wirebufs_sixteen", vec![], |b| {
            let freqs: Vec<f64> = (1..=16).map(|n| f64::from(n) * 100.0).collect();
            let sounds = sine(b, "ar", array(&freqs));
            out(b, "ar", sounds);
        }),
        def("wirebufs_demand_audio", vec![], |b| {
            let sound = sine(b, "ar", 1.0);
            demand_duty(b, sound);
        }),
        def("wirebufs_demand_control", vec![], |b| {
            let sound = sine(b, "kr", 1.0);
            demand_duty(b, sound);
        }),
        def("wirebufs_unread", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let pan = by_name(b, "Pan2", "ar", &[("in", input(sound))]);
            out(b, "ar", element(&pan, 0));
            let a = sine(b, "ar", 2.0);
            let c = sine(b, "ar", 3.0);
            let product = b.binary_op("*", a, c).unwrap();
            by_name(
                b,
                "Out",
                "ar",
                &[("bus", input(1.0)), ("channelsArray", input(product))],
            );
        }),
    ]
}

/// The units that cannot alias, from `PlayBuf` to `DemandEnvGen`.
fn buffers_and_demand() -> Vec<SynthDef> {
    vec![
        def("wirebufs_playbuf", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let args = [
                ("numChannels", input(1.0)),
                ("bufnum", input(0.0)),
                ("rate", input(sound)),
            ];
            rule_out(b, "PlayBuf", "ar", &args);
        }),
        def("wirebufs_playbuf_control", vec![], |b| {
            let args = [("numChannels", input(1.0)), ("bufnum", input(0.0))];
            rule_out(b, "PlayBuf", "ar", &args);
        }),
        def("wirebufs_tgrains", vec![], |b| {
            let trigger = impulse(b, "ar", 10.0);
            let args = [
                ("numChannels", input(2.0)),
                ("trigger", input(trigger)),
                ("bufnum", input(0.0)),
            ];
            rule_out(b, "TGrains", "ar", &args);
        }),
        def("wirebufs_demand", vec![], |b| {
            let trig = impulse(b, "ar", 10.0);
            let list = dseq(b, &[1.0, 2.0]);
            let args = [
                ("trig", input(trig)),
                ("reset", input(0.0)),
                ("demandUGens", input(list)),
            ];
            rule_out(b, "Demand", "ar", &args);
        }),
        def("wirebufs_duty", vec![], |b| duty(b, "Duty")),
        def("wirebufs_tduty", vec![], |b| duty(b, "TDuty")),
        def("wirebufs_demandenvgen", vec![], |b| {
            demand_env_gen(b, |b| vec![("gate", input(impulse(b, "ar", 1.0)))]);
        }),
    ]
}

/// The units that cannot alias, from `GrainIn` to `PartConv`.
fn grains_and_pans() -> Vec<SynthDef> {
    vec![
        def("wirebufs_grainin", vec![], |b| {
            let trigger = impulse(b, "ar", 10.0);
            let sound = sine(b, "ar", 440.0);
            let args = [
                ("numChannels", input(2.0)),
                ("trigger", input(trigger)),
                ("dur", input(0.1)),
                ("in", input(sound)),
            ];
            rule_out(b, "GrainIn", "ar", &args);
        }),
        def("wirebufs_grainsin", vec![], |b| grains(b, "GrainSin")),
        def("wirebufs_grainfm", vec![], |b| grains(b, "GrainFM")),
        def("wirebufs_grainbuf", vec![], |b| {
            let trigger = impulse(b, "ar", 10.0);
            let args = [
                ("numChannels", input(2.0)),
                ("trigger", input(trigger)),
                ("dur", input(0.1)),
                ("sndbuf", input(0.0)),
            ];
            rule_out(b, "GrainBuf", "ar", &args);
        }),
        def("wirebufs_warp1", vec![], |b| {
            let pointer = sine(b, "ar", 440.0);
            let args = [
                ("numChannels", input(1.0)),
                ("bufnum", input(0.0)),
                ("pointer", input(pointer)),
            ];
            rule_out(b, "Warp1", "ar", &args);
        }),
        def("wirebufs_balance2", vec![], |b| {
            let left = sine(b, "ar", 440.0);
            let right = sine(b, "ar", 2.0);
            let args = [("left", input(left)), ("right", input(right))];
            rule_out(b, "Balance2", "ar", &args);
        }),
        def("wirebufs_panb2", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            rule_out(b, "PanB2", "ar", &[("in", input(sound))]);
        }),
        def("wirebufs_panaz", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let args = [("numChans", input(2.0)), ("in", input(sound))];
            rule_out(b, "PanAz", "ar", &args);
        }),
        def("wirebufs_decodeb2", vec![], |b| {
            let w = sine(b, "ar", 440.0);
            let x = sine(b, "ar", 2.0);
            let y = sine(b, "ar", 3.0);
            // `ugen` refuses DecodeB2.ar.
            let decoded = b
                .add(
                    NewUGen::new("DecodeB2", Rate::Audio)
                        .inputs([w, x, y, 0.5.into()])
                        .outputs([Rate::Audio; 2]),
                )
                .unwrap();
            out(b, "ar", decoded);
        }),
        def("wirebufs_partconv", vec![], |b| {
            let sound = sine(b, "ar", 440.0);
            let args = [
                ("in", input(sound)),
                ("fftsize", input(2048.0)),
                ("irbufnum", input(0.0)),
            ];
            rule_out(b, "PartConv", "ar", &args);
        }),
    ]
}

/// `class.ar(0.1, Impulse.ar(1), Dseq([1, 2], inf))` into `Out.ar`.
fn duty(b: &mut SynthDefBuilder, class: &str) {
    let reset = impulse(b, "ar", 1.0);
    let list = dseq(b, &[1.0, 2.0]);
    let args = [
        ("dur", input(0.1)),
        ("reset", input(reset)),
        ("level", input(list)),
    ];
    rule_out(b, class, "ar", &args);
}

/// `class.ar(2, Impulse.ar(10))` into `Out.ar`.
fn grains(b: &mut SynthDefBuilder, class: &str) {
    let trigger = impulse(b, "ar", 10.0);
    let args = [("numChannels", input(2.0)), ("trigger", input(trigger))];
    rule_out(b, class, "ar", &args);
}
