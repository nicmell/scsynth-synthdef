// The golden files of tests/fixtures: synth definitions as sclang 3.14.1
// builds and writes them, from its core class library alone. Run it from
// the root of the repository, with absolute paths, since ~/.bin/sclang
// changes the directory:
//
//   sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/fixtures.scd"
//
// It exits with 1 if sclang is not 3.14.1, if a class or a method comes from
// outside the core class library, or if finishBuild changes a graph.
(
var root = thisProcess.nowExecutingPath.dirname.dirname;
var fixtures = root +/+ "tests" +/+ "fixtures";
var library = Platform.classLibraryDir;
var fail = { |message|
	("fixtures.scd: " ++ message).postln;
	1.exit;
};
var core = { |item| item.filenameSymbol.asString.beginsWith(library) };
var build, write, written = 0;

try {
	if (Main.version != "3.14.1") {
		fail.("sclang % is not 3.14.1".format(Main.version));
	};
	Class.allClasses.do { |class|
		if (core.(class).not) {
			fail.("% comes from %".format(class, class.filenameSymbol));
		};
		(class.methods ? []).do { |method|
			if (core.(method).not) {
				fail.("% comes from %".format(method, method.filenameSymbol));
			};
		};
	};

	// SynthDef:build, which also fails when finishBuild adds, removes or
	// rewires a UGen: addCopiesIfNeeded and optimizeGraph must leave the
	// graph as the function built it.
	build = { |name, func, rates, variants, class|
		var def = (class ? SynthDef).newCopyArgs(
			name: name.asSymbol,
			variants: variants,
			metadata: (),
			children: Array(64)
		);
		var before;
		protect {
			def.initBuild;
			def.buildUgenGraph(func, rates, nil);
			before = def.children.collect { |ugen| [ugen, ugen.inputs.copy] };
			def.finishBuild;
		} {
			UGen.buildSynthDef = nil;
		};
		if (def.children.size != before.size or: {
			before.any { |pair|
				def.children.includes(pair[0]).not or: { pair[0].inputs != pair[1] }
			}
		}) {
			fail.("finishBuild changes the graph of %".format(name));
		};
		def
	};

	write = { |name, defs|
		var stream = CollStream.on(Int8Array.new(256));
		if (defs.size == 1) {
			stream.putAll(defs[0].asBytes);
		} {
			defs.writeDef(stream);
		};
		File.use(fixtures +/+ name ++ ".scsyndef", "wb", { |file|
			file.putAll(stream.collection);
		});
		written = written + 1;
	};

	write.("sine", [build.(\sine, { |out = 0, freq = 440, amp = 0.1|
		Out.ar(out, SinOsc.ar(freq) * amp)
	})]);

	// Every rate, by entry and by prefix. i_k is scalar despite its \kr.
	write.("rates", [build.(\rates, { |freq = 440, gate = 1, src = 0, n = 2,
		i_k = 3, a_in = 0, t_x = 0, amps = #[1, 2], level = 0.5|
		var sig = SinOsc.ar(freq) * amps[0] * amps[1] * level;
		Out.ar(0, sig * gate * src * n * i_k * a_in * t_x)
	}, [0.1, \tr, \ar, \ir, \kr, nil, nil, 0.2])]);

	// 17 lagged values: a LagControl of 16 and one of 1.
	write.("lags17", [build.(\lags17, {
		|values = #[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]|
		Out.kr(0, values)
	}, [0.1])]);

	write.("multi_out", [build.(\multi_out, { |pos = 0|
		var in = In.ar(2, 2);
		Out.ar(0, Pan2.ar(in[0], pos));
		Out.ar(2, in[1]);
	})]);

	// a fans out to c and d, which meet again in c - d. The sort runs the
	// chain of a before b, which was created before c.
	write.("sort", [build.(\sort, {
		var a = SinOsc.ar(220);
		var b = LFSaw.ar(1);
		var c = LPF.ar(a, 800);
		var d = HPF.ar(a, 400);
		var e = RLPF.ar(b, 600);
		Out.ar(0, [c - d, e]);
	})]);

	// x comes after the FFT, so it waits for it.
	write.("fft", [build.(\fft, {
		var chain = FFT(0, In.ar(0));
		var x = SinOsc.ar(2);
		chain = PV_MagAbove(chain, 0.5);
		Out.ar(0, IFFT(chain) * x);
	})]);

	write.("demand", [build.(\demand, {
		Out.kr(0, Demand.kr(Impulse.kr(4), 0, Dseq([60, 62, 64], inf)))
	})]);

	// 1 and 1.0 are one constant. 1.0 + 1e-12 is another, which float32
	// writes as 1.0 too.
	write.("constants", [build.(\constants, {
		Out.kr(0, [1, 1.0, 0.5, 0.5, inf, -inf, 1.0 + 1e-12, 2])
	})]);

	// No +, no - of a neg, no fold: optimizeGraph leaves these alone.
	write.("operators", [build.(\operators, { |freq = 440, i_n = 2|
		var a = SinOsc.ar(freq);
		Out.ar(0, [a.abs, a.neg.pow(2), (a - freq).max(0.1), a < 0.5]);
		Out.kr(0, [freq.midicps, i_n * 2, Demand.kr(Impulse.kr(1), 0, Dwhite(0, 1) * 3)]);
	})]);

	write.("variant", [build.(\variant, { |freq = 440, amp = 0.1|
		Out.ar(0, SinOsc.ar(freq) * amp)
	}, nil, (low: [freq: 220]))]);

	write.("two_defs", [
		build.(\first, { Out.kr(0, DC.kr(1)) }),
		build.(\second, { |level = 0.5| Out.kr(1, DC.kr(level)) })
	]);

	// SynthDefOld.new fails in 3.14.1, so build sets one up directly.
	write.("old", [build.(\old, { |freq = 440|
		Out.ar(0, SinOsc.ar(freq))
	}, nil, (low: [freq: 220]), SynthDefOld)]);

	write.("envgen", [build.(\envgen, { |gate = 1|
		Out.ar(0, SinOsc.ar * EnvGen.kr(Env.perc(0.01, 1), gate, doneAction: 2))
	})]);

	write.("sendreply", [build.(\sendreply, { |freq = 440|
		SendReply.kr(Impulse.kr(1), '/tick', [freq, 7], 3)
	})]);

	// a has no value, and its name points at the end of the parameters.
	write.("empty_array", [build.(\empty_array, { |b = 1, a = #[]|
		Out.kr(0, b)
	})]);

	"fixtures.scd: wrote % golden files to %".format(written, fixtures).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
