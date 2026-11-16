//! The case files of sc3-plugins, built by hand: the expansion over counts,
//! arrays and a variable list of arguments, PV chains, and the reverbs.
//! `generator/sc3_plugins/fixtures.scd` holds the sclang code of each.

use scsynth_synthdef::{Arg, ArgValue, SynthDef, SynthDefBuilder, UGenInput, write_def_file};

use super::common::{arg, array, by_name};
use super::golden;

/// A builder without arguments.
fn builder(name: &str) -> SynthDefBuilder {
    SynthDefBuilder::new(name, Vec::new()).unwrap()
}

fn sine(b: &mut SynthDefBuilder, selector: &str, freq: f64) -> UGenInput {
    by_name(b, "SinOsc", selector, &[("freq", freq.into())])
}

/// `Out.ar(0, channels)` or `Out.kr(0, channels)`, then the definition.
fn out(mut b: SynthDefBuilder, selector: &str, channels: UGenInput) -> SynthDef {
    by_name(
        &mut b,
        "Out",
        selector,
        &[("bus", 0.0.into()), ("channelsArray", channels.into())],
    );
    b.finish().unwrap()
}

fn sines(b: &mut SynthDefBuilder, selector: &str, freqs: &[f64]) -> UGenInput {
    UGenInput::Array(freqs.iter().map(|&freq| sine(b, selector, freq)).collect())
}

/// `FFT(LocalBuf(frames), SinOsc.ar)`.
fn chain(b: &mut SynthDefBuilder, frames: f64) -> UGenInput {
    let buffer = by_name(b, "LocalBuf", "new", &[("numFrames", frames.into())]);
    let input = sine(b, "ar", 440.0);
    by_name(
        b,
        "FFT",
        "new",
        &[("buffer", buffer.into()), ("in", input.into())],
    )
}

fn expansion() -> Vec<SynthDef> {
    let mut defs = Vec::new();
    let mut b = builder("expansion_panx");
    let input = sine(&mut b, "ar", 440.0);
    let args = [
        ("numChans", array(&[2.0, 3.0]).into()),
        ("in", input.into()),
        ("pos", 0.25.into()),
    ];
    let pan = by_name(&mut b, "PanX", "ar", &args);
    defs.push(out(b, "ar", pan));
    let mut b = builder("expansion_vbap");
    let input = sine(&mut b, "ar", 440.0);
    let args = [
        ("numChans", 4.0.into()),
        ("in", input.into()),
        ("bufnum", 0.0.into()),
        ("azimuth", array(&[0.0, 45.0]).into()),
    ];
    let vbap = by_name(&mut b, "VBAP", "ar", &args);
    defs.push(out(b, "ar", vbap));
    let mut b = builder("expansion_grainbufj");
    let trigger = by_name(&mut b, "Impulse", "ar", &[("freq", 10.0.into())]);
    let args: [(&str, ArgValue); 9] = [
        ("numChannels", array(&[1.0, 2.0]).into()),
        ("trigger", trigger.into()),
        ("dur", 0.1.into()),
        ("sndbuf", 0.0.into()),
        ("rate", 1.0.into()),
        ("pos", 0.0.into()),
        ("loop", 0.0.into()),
        ("interp", 2.0.into()),
        ("grainAmp", 0.5.into()),
    ];
    let grains = by_name(&mut b, "GrainBufJ", "ar", &args);
    defs.push(out(b, "ar", grains));
    defs.extend([logger(false), logger(true)]);
    let mut b = builder("expansion_ntube");
    let input = sine(&mut b, "ar", 440.0);
    let args = [
        ("input", input.into()),
        ("lossarray", array(&[0.9, 0.8, 0.7]).into()),
        ("karray", array(&[0.1, 0.2]).into()),
        ("delaylengtharray", array(&[0.01, 0.02, 0.03]).into()),
        ("mul", 0.5.into()),
    ];
    let tube = by_name(&mut b, "NTube", "ar", &args);
    defs.push(out(b, "ar", tube));
    let mut b = builder("expansion_arraymax");
    let inputs = sines(&mut b, "kr", &[1.0, 2.0, 3.0]);
    let max = by_name(&mut b, "ArrayMax", "kr", &[("array", inputs.into())]);
    defs.push(out(b, "kr", max));
    defs.extend([ladspa(false), ladspa(true)]);
    let mut b = builder("expansion_chromagram");
    let fft = chain(&mut b, 2048.0);
    let args = [
        ("fft", fft.into()),
        ("fftsize", 2048.0.into()),
        ("n", array(&[12.0, 24.0]).into()),
    ];
    let chroma = by_name(&mut b, "Chromagram", "kr", &args);
    defs.push(out(b, "kr", chroma));
    defs
}

/// `Logger.kr` of two sines, or of an array of two sines and a third.
fn logger(nested: bool) -> SynthDef {
    let name = if nested {
        "expansion_logger_nested"
    } else {
        "expansion_logger"
    };
    let mut b = builder(name);
    let inputs = if nested {
        let pair = sines(&mut b, "kr", &[1.0, 2.0]);
        UGenInput::Array(vec![pair, sine(&mut b, "kr", 3.0)])
    } else {
        sines(&mut b, "kr", &[1.0, 2.0])
    };
    let trig = by_name(&mut b, "Impulse", "kr", &[("freq", 1.0.into())]);
    let args = [
        ("inputArray", inputs.into()),
        ("trig", trig.into()),
        ("bufnum", 0.0.into()),
        ("reset", 0.0.into()),
    ];
    by_name(&mut b, "Logger", "kr", &args);
    b.finish().unwrap()
}

/// `LADSPA.ar(1, 1049, x, 0.5)`, with a sine as `x` or an array of two.
fn ladspa(nested: bool) -> SynthDef {
    let name = if nested {
        "expansion_ladspa_nested"
    } else {
        "expansion_ladspa"
    };
    let mut b = builder(name);
    let input = if nested {
        sines(&mut b, "ar", &[1.0, 2.0])
    } else {
        sine(&mut b, "ar", 440.0)
    };
    let args = [
        ("nChans", 1.0.into()),
        ("id", 1049.0.into()),
        ("args", UGenInput::Array(vec![input, 0.5.into()]).into()),
    ];
    let plugin = by_name(&mut b, "LADSPA", "ar", &args);
    out(b, "ar", plugin)
}

fn pv() -> Vec<SynthDef> {
    let mut b = builder("pv_copies");
    let fft = chain(&mut b, 512.0);
    let inverse = |b: &mut SynthDefBuilder, class: &str| {
        let bins = by_name(b, class, "new", &[("buffer", fft.clone().into())]);
        by_name(b, "IFFT", "new", &[("buffer", bins.into())])
    };
    let invert = inverse(&mut b, "PV_Invert");
    let even = inverse(&mut b, "PV_EvenBin");
    let sum = b.binary_op("+", invert, even).unwrap();
    let copies = out(b, "ar", sum);
    let mut b = builder("pv_magmuladd");
    let fft = chain(&mut b, 512.0);
    let args = [
        ("buffer", fft.into()),
        ("mul", 2.0.into()),
        ("add", 0.5.into()),
    ];
    let bins = by_name(&mut b, "PV_MagMulAdd", "new", &args);
    let sound = by_name(&mut b, "IFFT", "new", &[("buffer", bins.into())]);
    vec![copies, out(b, "ar", sound)]
}

fn reverbs() -> Vec<SynthDef> {
    let mut defs = Vec::new();
    let mut b = builder("reverbs_greyhole_mono");
    let input = sine(&mut b, "ar", 440.0);
    let wet = by_name(&mut b, "Greyhole", "ar", &[("in", input.into())]);
    defs.push(out(b, "ar", wet));
    let mut b = builder("reverbs_greyhole_three");
    let inputs = sines(&mut b, "ar", &[1.0, 2.0, 3.0]);
    let args = [
        ("in", inputs.into()),
        ("delayTime", 1.5.into()),
        ("feedback", 0.5.into()),
    ];
    let wet = by_name(&mut b, "Greyhole", "ar", &args);
    defs.push(out(b, "ar", wet));
    let mut b = SynthDefBuilder::new("reverbs_jpverb", vec![Arg::new("t60", 2.0)]).unwrap();
    let t60 = arg(&b, "t60");
    let inputs = sines(&mut b, "ar", &[1.0, 2.0]);
    let wet = by_name(
        &mut b,
        "JPverb",
        "ar",
        &[("in", inputs.into()), ("t60", t60.into())],
    );
    defs.push(out(b, "ar", wet));
    let mut b = builder("reverbs_jpverb_nested");
    let pair = sines(&mut b, "ar", &[1.0, 2.0]);
    let inputs = UGenInput::Array(vec![pair, sine(&mut b, "ar", 3.0)]);
    let args = [("in", inputs.into()), ("highcut", 4000.0.into())];
    let wet = by_name(&mut b, "JPverb", "ar", &args);
    defs.push(out(b, "ar", wet));
    let mut b = builder("reverbs_nhhall");
    let inputs = sines(&mut b, "ar", &[1.0, 2.0]);
    let args = [("in", inputs.into()), ("rt60", 2.0.into())];
    let wet = by_name(&mut b, "NHHall", "ar", &args);
    defs.push(out(b, "ar", wet));
    let mut b = builder("reverbs_nhhall_nested");
    let pair = sines(&mut b, "ar", &[1.0, 2.0]);
    let inputs = UGenInput::Array(vec![pair, sine(&mut b, "ar", 3.0)]);
    let args = [("in", inputs.into()), ("rt60", array(&[1.0, 2.0]).into())];
    let wet = by_name(&mut b, "NHHall", "ar", &args);
    defs.push(out(b, "ar", wet));
    defs
}

#[test]
fn the_case_files_write_the_bytes_of_sclang() {
    for (name, defs) in [
        ("expansion", expansion()),
        ("pv", pv()),
        ("reverbs", reverbs()),
    ] {
        assert_eq!(write_def_file(&defs).unwrap(), golden(name), "{name}");
    }
}
