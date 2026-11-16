//! The graphs of the golden files of `PackFFT.new` and of the `pvcalc`
//! methods, built with the crate. `generator/fixtures.scd` holds the sclang
//! code of each.

use scsynth_synthdef::{ArgValue, SynthDef, SynthDefBuilder, UGenInput};

use super::controls::kr;
use super::pseudo::{def, input, out, sine};
use super::{array, by_name};

/// The definitions of a golden file of 0.11.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "packfft" => [packfft_inputs(), packfft_arrays()].concat(),
        _ => panic!("no golden graph is named {name}"),
    }
}

/// `FFT(LocalBuf(8), SinOsc.ar)`.
fn chain(b: &mut SynthDefBuilder) -> UGenInput {
    let buffer = by_name(b, "LocalBuf", "new", &[("numFrames", input(8.0))]);
    let sound = sine(b, "ar", 440.0);
    by_name(
        b,
        "FFT",
        "new",
        &[("buffer", input(buffer)), ("in", input(sound))],
    )
}

/// `PackFFT(chain, bufsize, magsphases, ...)` of the arguments, into
/// `Out.ar(0, IFFT(...))`.
fn pack_out(b: &mut SynthDefBuilder, args: &[(&str, ArgValue)]) {
    let packed = by_name(b, "PackFFT", "new", args);
    let sound = by_name(b, "IFFT", "new", &[("buffer", input(packed))]);
    out(b, "ar", sound);
}

/// The cases of `packfft` that set the inputs, from `packfft_unpack` to
/// `packfft_ugen_frombin`.
fn packfft_inputs() -> Vec<SynthDef> {
    vec![
        def("packfft_unpack", vec![], |b| {
            let c = chain(b);
            let unpack = [("chain", input(c.clone())), ("bufsize", input(8.0))];
            let bins = by_name(b, "UnpackFFT", "new", &unpack);
            let args = [
                ("chain", input(c)),
                ("bufsize", input(8.0)),
                ("magsphases", input(bins)),
            ];
            pack_out(b, &args);
        }),
        def("packfft_numbers", vec![], |b| {
            let c = chain(b);
            let args = [
                ("chain", input(c)),
                ("bufsize", input(8.0)),
                ("magsphases", input(array(&[1.0, 0.0, 1.0, 0.0]))),
                ("frombin", input(1.0)),
                ("tobin", input(2.0)),
                ("zeroothers", input(1.0)),
            ];
            pack_out(b, &args);
        }),
        def("packfft_ugen_size", vec![], |b| {
            let c = chain(b);
            let size = kr(b, "b", 8.0);
            let args = [
                ("chain", input(c)),
                ("bufsize", input(size)),
                ("magsphases", input(array(&[1.0, 0.0]))),
            ];
            pack_out(b, &args);
        }),
        def("packfft_ugen_frombin", vec![], |b| {
            let c = chain(b);
            let frombin = kr(b, "f", 0.0);
            let args = [
                ("chain", input(c)),
                ("bufsize", input(8.0)),
                ("magsphases", input(array(&[1.0, 0.0]))),
                ("frombin", input(frombin)),
            ];
            pack_out(b, &args);
        }),
    ]
}

/// The cases of `packfft` that hold arrays, from `packfft_nested` to
/// `packfft_markup`.
fn packfft_arrays() -> Vec<SynthDef> {
    vec![
        def("packfft_nested", vec![], |b| {
            let c = chain(b);
            let magsphases = UGenInput::Array(vec![array(&[1.0, 2.0]), 0.0.into()]);
            let args = [
                ("chain", input(c)),
                ("bufsize", input(8.0)),
                ("magsphases", input(magsphases)),
            ];
            pack_out(b, &args);
        }),
        def("packfft_empty", vec![], |b| {
            let c = chain(b);
            let args = [
                ("chain", input(c)),
                ("bufsize", input(8.0)),
                ("magsphases", input(UGenInput::Array(vec![]))),
            ];
            pack_out(b, &args);
        }),
        def("packfft_chains", vec![], |b| {
            let buffers = by_name(
                b,
                "LocalBuf",
                "new",
                &[
                    ("numFrames", input(8.0)),
                    ("numChannels", input(array(&[1.0, 1.0]))),
                ],
            );
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let chains = by_name(
                b,
                "FFT",
                "new",
                &[("buffer", input(buffers)), ("in", input(sounds))],
            );
            let args = [
                ("chain", input(chains)),
                ("bufsize", input(8.0)),
                ("magsphases", input(array(&[1.0, 0.0]))),
            ];
            pack_out(b, &args);
        }),
        def("packfft_markup", vec![], |b| {
            let c = chain(b);
            let unpack = [("chain", input(c.clone())), ("bufsize", input(8.0))];
            let bins = by_name(b, "UnpackFFT", "new", &unpack);
            let doubled = b.binary_op("*", bins, 2.0).unwrap();
            let args = [
                ("chain", input(c)),
                ("bufsize", input(8.0)),
                ("magsphases", input(doubled)),
            ];
            pack_out(b, &args);
        }),
    ]
}
