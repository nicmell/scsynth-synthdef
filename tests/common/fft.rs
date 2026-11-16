//! The graphs of the golden files of `PackFFT.new` and of the `pvcalc`
//! methods, built with the crate. `generator/fixtures.scd` holds the sclang
//! code of each.

use scsynth_synthdef::{ArgValue, BuildError, SynthDef, SynthDefBuilder, UGenInput};

use super::controls::kr;
use super::pseudo::{def, fft, ifft, input, local_buf, out, sine};
use super::{array, by_name, element};

/// The definitions of a golden file of 0.11.
pub fn definitions(name: &str) -> Vec<SynthDef> {
    match name {
        "packfft" => [packfft_inputs(), packfft_arrays()].concat(),
        "pvcalc" => [pvcalc(), pvcollect()].concat(),
        _ => panic!("no golden graph is named {name}"),
    }
}

/// `FFT(LocalBuf(8), SinOsc.ar)`.
fn chain(b: &mut SynthDefBuilder) -> UGenInput {
    let buffer = local_buf(b, 8.0, 1.0);
    let sound = sine(b, "ar", 440.0);
    fft(b, buffer, sound)
}

/// `PackFFT(chain, bufsize, magsphases, ...)` of the arguments, into
/// `Out.ar(0, IFFT(...))`.
fn pack_out(b: &mut SynthDefBuilder, args: &[(&str, ArgValue)]) {
    let packed = by_name(b, "PackFFT", "new", args);
    ifft_out(b, packed);
}

/// `Out.ar(0, IFFT(chain))`.
fn ifft_out(b: &mut SynthDefBuilder, chain: UGenInput) {
    let sound = ifft(b, chain);
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
            let buffers = local_buf(b, 8.0, array(&[1.0, 1.0]));
            let sounds = sine(b, "ar", array(&[1.0, 2.0]));
            let chains = fft(b, buffers, sounds);
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

/// `chain.pvcalc(8, func, ...)` of the chain, into `Out.ar(0, IFFT(...))`,
/// with these other arguments.
fn calc(
    name: &str,
    args: &[(&str, ArgValue)],
    func: impl FnOnce(&mut SynthDefBuilder, UGenInput, UGenInput) -> Result<UGenInput, BuildError>,
) -> SynthDef {
    def(name, vec![], |b| {
        let c = chain(b);
        let mut all = vec![("numframes", input(8.0))];
        all.extend(args.iter().cloned());
        let packed = b.pvcalc(c, &all, func).unwrap();
        ifft_out(b, packed);
    })
}

/// `{ |m, p| [m, p] }`: the magnitudes and the phases as they come.
fn identity()
-> impl FnOnce(&mut SynthDefBuilder, UGenInput, UGenInput) -> Result<UGenInput, BuildError> {
    |_, magnitudes, phases| Ok(UGenInput::Array(vec![magnitudes, phases]))
}

fn pvcalc() -> Vec<SynthDef> {
    let bins = |frombin: f64, tobin: f64| [("frombin", input(frombin)), ("tobin", input(tobin))];
    vec![
        calc("pvcalc_identity", &[], identity()),
        calc("pvcalc_mags", &[], |b, m, _| b.binary_op("*", m, 2.0)),
        calc("pvcalc_one_bin", &bins(2.0, 2.0), |_, m, _| Ok(m)),
        calc("pvcalc_one_value", &[], |_, m, _| {
            Ok(UGenInput::Array(vec![element(&m, 0)]))
        }),
        calc("pvcalc_ugen", &[], |_, m, _| Ok(element(&m, 0))),
        calc("pvcalc_two_bins", &bins(0.0, 1.0), |_, m, _| Ok(m)),
        calc("pvcalc_past", &bins(0.0, 10.0), identity()),
        def("pvcalc2", vec![], |b| {
            let c = chain(b);
            let buffer = local_buf(b, 8.0, 1.0);
            let sound = sine(b, "ar", 2.0);
            let d = fft(b, buffer, sound);
            let args = [("chain2", input(d)), ("numframes", input(8.0))];
            let packed = b
                .pvcalc2(c, &args, |b, m, p, m2, _| {
                    let sum = b.binary_op("+", m, m2)?;
                    Ok(UGenInput::Array(vec![sum, p]))
                })
                .unwrap();
            ifft_out(b, packed);
        }),
    ]
}

/// `chain.pvcollect(8, func, ...)` of the chain, into `Out.ar(0, IFFT(...))`,
/// with these other arguments.
fn collect(
    name: &str,
    args: &[(&str, ArgValue)],
    func: impl FnMut(
        &mut SynthDefBuilder,
        UGenInput,
        UGenInput,
        f64,
        f64,
    ) -> Result<UGenInput, BuildError>,
) -> SynthDef {
    def(name, vec![], |b| {
        let c = chain(b);
        let mut all = vec![("numframes", input(8.0))];
        all.extend(args.iter().cloned());
        let packed = b.pvcollect(c, &all, func).unwrap();
        ifft_out(b, packed);
    })
}

fn pvcollect() -> Vec<SynthDef> {
    vec![
        collect("pvcollect_mag", &[], |_, m, _, _, _| Ok(m)),
        collect("pvcollect_three", &[], |_, m, p, _, _| {
            Ok(UGenInput::Array(vec![m, p, 0.0.into()]))
        }),
        collect(
            "pvcollect_bin",
            &[("frombin", input(1.0)), ("tobin", input(3.0))],
            |b, m, _, bin, _| b.binary_op("*", m, bin),
        ),
    ]
}
