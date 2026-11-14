// The golden files of tests/fixtures: synth definitions as sclang 3.14.1
// builds and writes them, from its core class library alone. Run it from
// the root of the repository, with absolute paths, since ~/.bin/sclang
// changes the directory:
//
//   sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/fixtures.scd"
//
// It exits with 1 if sclang is not 3.14.1, if a class or a method comes from
// outside the core class library, or if finishBuild changes a graph of 0.1
// or 0.2. It also writes tests/fixtures/before_optimize and
// tests/fixtures/arithmetic.json.
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
var buildBoth, writePseudo, rewritten = List.new, writeArithmetic;

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

	write = { |name, defs, folder|
		var stream = CollStream.on(Int8Array.new(256));
		if (defs.size == 1) {
			stream.putAll(defs[0].asBytes);
		} {
			defs.writeDef(stream);
		};
		File.use((folder ? fixtures) +/+ name ++ ".scsyndef", "wb", { |file|
			file.putAll(stream.collection);
		});
		written = written + 1;
	};

	// The full build of SynthDef:build, and the build of finishBuild without
	// optimizeGraph (SynthDef.sc:279), for the graphs of the pseudo-UGens.
	buildBoth = { |name, func|
		[true, false].collect { |optimize|
			var def = SynthDef.newCopyArgs(
				name: name.asSymbol,
				metadata: (),
				children: Array(64)
			);
			protect {
				def.initBuild;
				def.buildUgenGraph(func, nil, nil);
				if (optimize) {
					def.finishBuild;
				} {
					def.addCopiesIfNeeded;
					def.collectConstants;
					def.checkInputs;
					def.topologicalSort;
					def.indexUGens;
				};
			} {
				UGen.buildSynthDef = nil;
			};
			def
		}
	};

	// One file of full builds, and one in before_optimize when optimizeGraph
	// rewrites a case of the file.
	writePseudo = { |name, cases|
		var pairs = cases.collect { |case| buildBoth.(case[0], case[1]) };
		write.(name, pairs.collect(_.first));
		if (pairs.any { |pair| pair[0].asBytes != pair[1].asBytes }) {
			write.(name, pairs.collect(_.last), fixtures +/+ "before_optimize");
			rewritten.add(name);
		};
	};

	// The operators of the arithmetic of the language on Floats, with each
	// float as the 16 hexadecimal digits of its bits.
	writeArithmetic = {
		var bits = { |x|
			x = x.asFloat;
			x.high32Bits.asHexString(8) ++ x.low32Bits.asHexString(8)
		};
		var row = { |items| "[" ++ items.collect { |item| "\"" ++ item ++ "\"" }.join(", ") ++ "]" };
		var values = [-2.5, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.7, 1000.0];
		var binary = [['+', "+"], ['-', "-"], ['*', "*"], ['/', "/"], ['pow', "pow"],
			['%', "mod"], ['round', "round"], ['trunc', "trunc"], ['roundUp', "roundUp"],
			['fold2', "fold2"], ['max', "max"], ['min', "min"], ['absdif', "absdif"]];
		var unary = [\neg, \abs, \reciprocal, \sqrt, \cos, \sin];
		var sections = List.new;
		sections.add("  \"binary\": [\n" ++ binary.collect { |op|
			values.collect { |a|
				values.collect { |b|
					"    " ++ row.([op[1], bits.(a), bits.(b), bits.(a.perform(op[0], b))])
				}
			}.flatten
		}.flatten.join(",\n") ++ "\n  ]");
		sections.add("  \"unary\": [\n" ++ unary.collect { |op|
			values.collect { |a| "    " ++ row.([op.asString, bits.(a), bits.(a.perform(op))]) }
		}.flatten.join(",\n") ++ "\n  ]");
		sections.add("  \"clip\": [\n" ++ values.collect { |x|
			[[0.0, 1.0], [-1.0, 1.0], [0.5, 2.0]].collect { |range|
				"    " ++ row.([x, range[0], range[1], x.clip(range[0], range[1])].collect(bits))
			}
		}.flatten.join(",\n") ++ "\n  ]");
		sections.add("  \"moddif\": [\n" ++ values.collect { |x|
			[[0.0, 1.0], [1.0, 3.0], [2.0, 3.0]].collect { |pair|
				"    " ++ row.([x, pair[0], pair[1], x.moddif(pair[0], pair[1])].collect(bits))
			}
		}.flatten.join(",\n") ++ "\n  ]");
		sections.add("  \"linlin\": [\n" ++ values.collect { |x|
			[[0.0, 1.0, -1.0, 1.0], [-1.0, 1.0, 0.0, 10.0]].collect { |range|
				"    " ++ row.(([x] ++ range ++ [x.linlin(*range)]).collect(bits))
			}
		}.flatten.join(",\n") ++ "\n  ]");
		File.use(fixtures +/+ "arithmetic.json", "w", { |file|
			file.write("{\n" ++ sections.join(",\n") ++ "\n}\n");
		});
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

	// Multichannel expansion. The constants show the order of creation.
	write.("pair", [build.(\pair, { |freqs = #[220, 331], amp = 0.4|
		Out.ar(0, SinOsc.ar(freqs) * amp)
	})]);

	write.("wrap", [build.(\wrap, {
		Out.ar(0, SinOsc.ar([440, 442, 444]) * [0.1, 0.2])
	})]);

	// sum starts from 0, which the fold of + drops.
	write.("voices", [build.(\voices, { |freqs = #[220, 330], places = #[-0.5, 0.5], amp = 0.2|
		Out.ar(0, Pan2.ar(SinOsc.ar(freqs) * amp, places).sum)
	})]);

	write.("nested", [build.(\nested, {
		Out.ar(0, LPF.ar(Pan2.ar(SinOsc.ar([220, 330]), [-0.5, 0.5]), 1000).sum)
	})]);

	// The bus and the count both expand: an In of one channel, then of two.
	write.("in_buses", [build.(\in_buses, {
		Out.ar(0, In.ar([4, 6], [1, 2]).flat)
	})]);

	// madd, after the expansion.
	write.("mul", [build.(\mul, { Out.ar(0, SinOsc.ar(440, 0, 0.1)) })]);

	write.("mul_add", [build.(\mul_add, { Out.ar(0, SinOsc.ar(440, 0, 0.1, 0.5)) })]);

	write.("mul_array", [build.(\mul_array, {
		Out.ar(0, SinOsc.ar([440, 442], 0, [0.1, 0.2, 0.3]))
	})]);

	write.("mul_expanded", [build.(\mul_expanded, {
		Out.ar(0, SinOsc.ar([440, 442], 0, 0.5))
	})]);

	// canBeMulAdd swaps in and mul.
	write.("mul_swap", [build.(\mul_swap, {
		Out.ar(0, SinOsc.kr(1, 0, SinOsc.ar(2), 0.5))
	})]);

	// No order of the inputs can be a MulAdd: a * and a +.
	write.("mul_ops", [build.(\mul_ops, {
		Out.ar(0, SinOsc.kr(1, 0, 0.5, SinOsc.ar(2)))
	})]);

	write.("madd_numbers", [build.(\madd_numbers, {
		Out.ar(0, MulAdd(2, 3, SinOsc.ar(2)))
	})]);

	// The folds of BinaryOpUGen:new1 and MulAdd:new1 that keep a UGen.
	write.("folds", [build.(\folds, {
		Out.ar(0, [
			SinOsc.ar(440) * 1,
			0 - SinOsc.ar(441),
			SinOsc.ar(442, 0, -1),
			SinOsc.ar(443) / -1,
			SinOsc.ar(444) + 0,
			SinOsc.ar(445) * -1
		])
	})]);

	// The pseudo-UGens of 0.3, one file each.
	(fixtures +/+ "before_optimize").mkdir;
	(fixtures +/+ "before_optimize/*.scsyndef").pathMatch.do { |path| File.delete(path) };

	writePseudo.("pmosc", [
		[\pmosc_ar, { Out.ar(0, PMOsc.ar(440, 220, 2)) }],
		[\pmosc_kr, { |car = 10| Out.kr(0, PMOsc.kr(car, 3, 1.5, 0.25, 0.5, 0.5)) }]
	]);
	writePseudo.("changed", [
		[\changed_kr, { Out.kr(0, Changed.kr(SinOsc.kr(1))) }],
		[\changed_ar, { Out.ar(0, Changed.ar(SinOsc.ar(2), 0.1)) }],
		[\changed_array, { Out.kr(0, Changed.kr(SinOsc.kr([1, 2]))) }]
	]);
	writePseudo.("silent", [
		[\silent_1, { Out.ar(0, Silent.ar) }],
		[\silent_2, { Out.ar(0, Silent.ar(2)) }]
	]);
	writePseudo.("dstutter", [
		[\dstutter, { Out.kr(0, Demand.kr(Impulse.kr(1), 0, Dstutter(2, Dseq([1, 2], inf)))) }]
	]);
	writePseudo.("tchoose", [
		[\tchoose_kr, { Out.kr(0, TChoose.kr(Impulse.kr(1), [SinOsc.kr(1), SinOsc.kr(2)])) }],
		[\tchoose_ar, {
			Out.ar(0, TChoose.ar(Impulse.ar(1), [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}]
	]);
	writePseudo.("twchoose", [
		[\twchoose_kr, {
			Out.kr(0, TWChoose.kr(Impulse.kr(1), [SinOsc.kr(1), SinOsc.kr(2)], [0.3, 0.7]))
		}],
		[\twchoose_ar, {
			Out.ar(0, TWChoose.ar(Impulse.ar(1), [SinOsc.ar(1), SinOsc.ar(2)], [1, 3], 1))
		}]
	]);
	writePseudo.("tap", [
		[\tap_1, { Out.ar(0, Tap.ar(0, 1, 0.2)) }],
		[\tap_2, { |time = 0.5| Out.ar(0, Tap.ar(1, 2, time)) }]
	]);
	// With numbers, LinLin computes its result in the language.
	writePseudo.("linlin", [
		[\linlin_ar, { Out.ar(0, LinLin.ar(SinOsc.ar(1), -1, 1, 100, 200)) }],
		[\linlin_kr, { Out.kr(0, LinLin.kr(SinOsc.kr(1), -1, 1, 100, 200)) }],
		[\linlin_numbers, {
			Out.kr(0, [LinLin.kr(0.5, 0, 1, 10, 20), LinLin.ar(0.25, 0, 1, 0, 4)] * SinOsc.kr(1))
		}],
		[\linlin_ugens, { |lo = 100| Out.kr(0, LinLin.kr(SinOsc.kr(1), -1, 1, lo, 200)) }]
	]);
	writePseudo.("soundin", [
		[\soundin_0, { Out.ar(0, SoundIn.ar(0)) }],
		[\soundin_01, { Out.ar(0, SoundIn.ar([0, 1])) }],
		[\soundin_02, { Out.ar(0, SoundIn.ar([0, 2])) }],
		[\soundin_mul, { Out.ar(0, SoundIn.ar(2, 0.5)) }],
		[\soundin_ugen, { |bus = 1| Out.ar(0, SoundIn.ar(bus)) }]
	]);
	writePseudo.("selectx", [
		[\selectx_ar, { |which = 0|
			Out.ar(0, SelectX.ar(which, [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}],
		[\selectx_kr, { Out.kr(0, SelectX.kr(0.5, [SinOsc.kr(1), SinOsc.kr(2)])) }]
	]);
	writePseudo.("blowpass4", [
		[\blowpass4, { Out.ar(0, BLowPass4.ar(SinOsc.ar(1), 800, 0.5)) }],
		[\blowpass4_ugens, { |freq = 800, rq = 0.5|
			Out.ar(0, BLowPass4.ar(SinOsc.ar(1), freq, rq, 0.5))
		}]
	]);
	writePseudo.("bhipass4", [
		[\bhipass4, { Out.ar(0, BHiPass4.ar(SinOsc.ar(1), 800, 0.5)) }]
	]);

	writePseudo.("sum", [
		[\sum3_fold, { Out.ar(0, Sum3(SinOsc.ar(1), SinOsc.ar(2), 0)) }],
		[\sum4_fold, { Out.ar(0, Sum4(SinOsc.ar(1), 0, SinOsc.kr(2), SinOsc.ar(3))) }],
		[\sum3_array, { Out.ar(0, Sum3([SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3), SinOsc.ar(4))) }]
	]);
	writePseudo.("mix", [
		[\mix_1, { Out.ar(0, Mix(SinOsc.ar(1))) }],
		[\mix_2, { Out.ar(0, Mix(SinOsc.ar([1, 2]))) }],
		[\mix_3, { Out.ar(0, Mix(SinOsc.ar([1, 2, 3]))) }],
		[\mix_4, { Out.ar(0, Mix(SinOsc.ar([1, 2, 3, 4]))) }],
		[\mix_5, { Out.ar(0, Mix(SinOsc.ar([1, 2, 3, 4, 5]))) }],
		[\mix_9, { Out.ar(0, Mix(SinOsc.ar((1..9)))) }],
		[\mix_rates3, { Out.ar(0, Mix([SinOsc.kr(1), SinOsc.ar(2), SinOsc.kr(3)])) }],
		[\mix_rates4, { Out.ar(0, Mix([SinOsc.kr(1), SinOsc.ar(2), SinOsc.kr(3), SinOsc.ar(4)])) }],
		[\mix_numbers, { Out.kr(0, [Mix([1, 2]), Mix([1, 2, 3])] * SinOsc.kr(1)) }],
		[\mix_nested, { Out.ar(0, Mix([SinOsc.ar([1, 2]), SinOsc.ar([3, 4]), SinOsc.ar([5, 6])])) }],
		[\mix_ar, { Out.ar(0, Mix.ar([SinOsc.kr(1), SinOsc.kr(2)])) }],
		[\mix_ar_number, { Out.ar(0, Mix.ar([1, 2])) }],
		[\mix_kr, { Out.kr(0, Mix.kr([SinOsc.ar(1), SinOsc.kr(2)])) }],
		[\mix_kr_number, { Out.kr(0, Mix.kr([1, 2])) }]
	]);

	writePseudo.("numchannels", [
		[\numchannels_mix, { Out.ar(0, NumChannels.ar(SinOsc.ar([1, 2, 3, 4]), 2)) }],
		[\numchannels_cut, { Out.ar(0, NumChannels.ar(SinOsc.ar([1, 2, 3]), 2, false)) }],
		[\numchannels_dup, { Out.ar(0, NumChannels.ar(SinOsc.ar(1), 3)) }],
		[\numchannels_one, { Out.ar(0, NumChannels.ar([SinOsc.ar(1)], 1)) }]
	]);
	writePseudo.("selectxfocus", [
		[\selectxfocus, { |which = 0|
			Out.ar(0, SelectXFocus.ar(which, [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}],
		[\selectxfocus_wrap, { |which = 0|
			Out.ar(0, SelectXFocus.ar(which, [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)], 0.5, true))
		}],
		[\selectxfocus_number, {
			Out.kr(0, SelectXFocus.kr(1.25, [SinOsc.kr(1), SinOsc.kr(2), SinOsc.kr(3)]))
		}]
	]);
	writePseudo.("multitap", [
		[\multitap, { Out.ar(0, MultiTap.ar(`[0.1, 0.2], `[0.5, 0.3], SinOsc.ar(1), bufnum: 0)) }],
		[\multitap_levels, {
			Out.ar(0, MultiTap.ar(`[0.1, 0.2, 0.3], `[0.5], SinOsc.ar(1), 0.5, 0, 1))
		}]
	]);
	writePseudo.("splay", [
		[\splay_2, { Out.ar(0, Splay.ar(SinOsc.ar([1, 2]))) }],
		[\splay_3, { Out.ar(0, Splay.ar(SinOsc.ar([1, 2, 3]))) }],
		[\splay_mono, { Out.ar(0, Splay.ar(SinOsc.ar(1))) }],
		[\splay_spread, { |spread = 1| Out.ar(0, Splay.ar(SinOsc.ar([1, 2, 3]), spread)) }],
		[\splay_nested, { Out.ar(0, Splay.ar([SinOsc.ar([1, 2]), SinOsc.ar([3, 4])]).flat) }],
		[\splay_kr, { Out.kr(0, Splay.kr(SinOsc.kr([1, 2, 3]))) }],
		[\splay_level, { |level = 0.5| Out.ar(0, Splay.ar(SinOsc.ar([1, 2]), 1, level)) }],
		[\splay_center, { Out.ar(0, Splay.ar(SinOsc.ar([1, 2]), 0.5, 1, 0.25)) }],
		[\splay_levelcomp_false, { Out.ar(0, Splay.ar(SinOsc.ar([1, 2]), levelComp: false)) }],
		[\splay_levelcomp_number, { Out.ar(0, Splay.ar(SinOsc.ar([1, 2]), levelComp: 0.5)) }],
		[\splay_levelcomp_ugen, { |lc = 0.5| Out.ar(0, Splay.ar(SinOsc.ar([1, 2]), levelComp: lc)) }]
	]);
	writePseudo.("dynklank", [
		[\dynklank_2, { Out.ar(0, DynKlank.ar(`[[800, 1071]], Impulse.ar(2))) }],
		[\dynklank_3, { Out.ar(0, DynKlank.ar(`[[800, 1071, 1353], nil, [1, 1, 1]], Impulse.ar(2))) }],
		[\dynklank_amps, { Out.ar(0, DynKlank.ar(`[[800, 1071], [0.5, 0.3]], Impulse.ar(2))) }],
		[\dynklank_scale, { |scale = 1|
			Out.ar(0, DynKlank.ar(`[[800, 1071], nil, [1, 2]], Impulse.ar(2), scale))
		}],
		[\dynklank_kr, { Out.kr(0, DynKlank.kr(`[[8, 10]], Impulse.kr(2))) }],
		[\dynklank_expand, { Out.ar(0, DynKlank.ar(`[[800, 1071]], Impulse.ar([2, 3]))) }]
	]);
	writePseudo.("dynklang", [
		[\dynklang_2, { Out.ar(0, DynKlang.ar(`[[800, 1000]])) }],
		[\dynklang_amps, { Out.ar(0, DynKlang.ar(`[[800, 1000], [0.3, 0.3], [pi, pi]])) }],
		[\dynklang_kr, { Out.kr(0, DynKlang.kr(`[[1, 2], nil, [0.5, 0.25]], 2, 1)) }]
	]);
	writePseudo.("unpackfft", [
		[\unpackfft, {
			var chain = FFT(0, SinOsc.ar(1));
			Out.kr(0, Demand.kr(Impulse.kr(1), 0, UnpackFFT(chain, 16, 0, 2)))
		}],
		[\unpackfft_default, {
			var chain = FFT(0, SinOsc.ar(1));
			Out.kr(0, Demand.kr(Impulse.kr(1), 0, UnpackFFT(chain, 4)))
		}]
	]);

	// The delays and LinXFade2, which 0.3 makes addable.
	writePseudo.("delays", [
		[\delayn_kr_input, { Out.ar(0, DelayN.ar(SinOsc.kr(1), 0.2, 0.1)) }],
		[\combl_number, { Out.ar(0, CombL.ar(0.5, 0.2, 0.1, 1)) }],
		[\allpassc_zero, { Out.ar(0, AllpassC.ar(0, 0.2, 0.1, 1)) }],
		[\bufdelayc_audio, { Out.ar(0, BufDelayC.ar(0, SinOsc.ar(1), 0.1)) }],
		[\bufcombn_array, { Out.ar(0, BufCombN.ar(0, [SinOsc.kr(1), 0.25], 0.1, 1)) }],
		[\delayn_mul, { Out.ar(0, DelayN.ar(SinOsc.ar(1), 0.2, 0.1, 0.5)) }],
		[\deltapwr, {
			var tap = DelTapWr.ar(0, SinOsc.kr(2));
			Out.ar(0, DelTapRd.ar(0, tap, 0.1))
		}]
	]);
	writePseudo.("linxfade2", [
		[\linxfade2_level, { Out.ar(0, LinXFade2.ar(SinOsc.ar(1), SinOsc.ar(2), 0.5, 0.5)) }],
		[\linxfade2_one, { Out.kr(0, LinXFade2.kr(SinOsc.kr(1), SinOsc.kr(2), -0.5)) }],
		[\linxfade2_ugen, { |level = 0.5|
			Out.ar(0, LinXFade2.ar(SinOsc.ar(1), SinOsc.ar(2), 0, level))
		}]
	]);

	writePseudo.("companderd", [
		[\companderd, { Out.ar(0, CompanderD.ar(SinOsc.ar(1))) }],
		[\companderd_number, { Out.ar(0, CompanderD.ar(0.5, 0.4, 1, 0.5, 0.02, 0.1, 0.5)) }]
	]);
	writePseudo.("hilbertfir", [
		[\hilbertfir, { Out.ar(0, HilbertFIR.ar(SinOsc.ar(1), 0)) }]
	]);
	writePseudo.("linselectx", [
		[\linselectx_ar, { |which = 0|
			Out.ar(0, LinSelectX.ar(which, [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}],
		[\linselectx_kr, { Out.kr(0, LinSelectX.kr(0.5, [SinOsc.kr(1), SinOsc.kr(2)])) }]
	]);
	writePseudo.("splayaz", [
		[\splayaz_3, { Out.ar(0, SplayAz.ar(4, SinOsc.ar([1, 2, 3]))) }],
		[\splayaz_mono, { Out.ar(0, SplayAz.ar(2, SinOsc.ar(1))) }],
		[\splayaz_kr, { Out.kr(0, SplayAz.kr(3, SinOsc.kr([1, 2]))) }],
		[\splayaz_spread, { Out.ar(0, SplayAz.ar(4, SinOsc.ar([1, 2, 3]), SinOsc.kr(0.1))) }],
		[\splayaz_center, { |c = 0| Out.ar(0, SplayAz.ar(2, SinOsc.ar([1, 2]), 1, 1, 2, c)) }],
		[\splayaz_audio_center, {
			Out.ar(0, SplayAz.ar(2, SinOsc.ar([1, 2]), 1, 1, 2, SinOsc.ar(0.1)))
		}]
	]);

	// The rate methods that do more than new1, of 0.4.
	writePseudo.("lag", [
		[\lag_kr, { |freq = 440| Out.kr(0, Lag.kr(freq, 0.1)) }],
		[\lag_ar, { Out.ar(0, Lag.ar(SinOsc.ar(1), 0.01)) }],
		[\lag2_kr, { |freq = 440| Out.kr(0, Lag2.kr(freq, 0.1)) }],
		[\lag3_ar, { Out.ar(0, Lag3.ar(SinOsc.ar(1), 0.01)) }],
		[\ramp_kr, { |freq = 440| Out.kr(0, Ramp.kr(freq, 0.1)) }],
		[\lagud_kr, { |freq = 440| Out.kr(0, LagUD.kr(freq, 0.1, 0.2)) }],
		[\lag2ud_ar, { Out.ar(0, Lag2UD.ar(SinOsc.ar(1), 0.01, 0.02)) }],
		[\lag3ud_kr, { |freq = 440| Out.kr(0, Lag3UD.kr(freq, 0.1, 0.2)) }],
		[\slew_ar, { Out.ar(0, Slew.ar(SinOsc.ar(1), 100, 200)) }],
		[\slew_kr, { |freq = 440| Out.kr(0, Slew.kr(freq)) }],
		[\lag_number, { Out.kr(0, Lag.kr(440, 0.1)) }],
		[\lag_zero, { |freq = 440| Out.kr(0, Lag.kr(freq, 0)) }],
		[\lag_scalar, { |i_freq = 440| Out.kr(0, Lag.kr(i_freq, 0.1)) }],
		[\lagud_zero, { |freq = 440| Out.kr(0, LagUD.kr(freq, 0, 0)) }],
		[\slew_scalar, { |i_freq = 440| Out.kr(0, Slew.kr(i_freq)) }],
		[\lag_madd_number, { Out.kr(0, Lag.kr(2, 0.1, 3, 4)) }],
		[\lag_madd_scalar, { |i_x = 1| Out.kr(0, Lag.kr(i_x, 0.1, 2, 3)) }],
		[\lag_madd, { |freq = 440| Out.kr(0, Lag.kr(freq, 0.1, 2, 3)) }],
		[\lag_array, { |a = 1, b = 2| Out.kr(0, Lag.kr([a, b], 0.1)) }],
		[\lag_array_scalar, { Out.kr(0, Lag.kr([1, 2], 0.1, 2)) }]
	]);
	writePseudo.("outzeros", [
		[\outzeros_one, { Out.ar(0, 0) }],
		[\outzeros_two, { Out.ar(0, [SinOsc.ar(1), 0, 0]) }],
		[\outzeros_negative, { Out.ar(0, [-0.0, SinOsc.ar(1)]) }],
		[\outzeros_replace, { ReplaceOut.ar(0, [0, SinOsc.ar(1)]) }],
		[\outzeros_offset, { OffsetOut.ar(0, 0) }],
		[\outzeros_x, { XOut.ar(0, 0.5, [SinOsc.ar(1), 0]) }],
		[\outzeros_kr, { Out.kr(0, 0) }]
	]);
	writePseudo.("localbuf", [
		[\localbuf_fft, { Out.ar(0, IFFT(FFT(LocalBuf(2048), SinOsc.ar(1)))) }],
		[\localbuf_two, {
			var a = FFT(LocalBuf(1024), SinOsc.ar(1));
			var b = FFT(LocalBuf(2048), SinOsc.ar(2));
			Out.ar(0, [IFFT(a), IFFT(b)])
		}],
		[\localbuf_array, { Out.ar(0, IFFT(FFT(LocalBuf([512, 1024]), SinOsc.ar(1)))) }],
		[\localbuf_channels, { Out.ar(0, PlayBuf.ar(2, LocalBuf(1024, 2))) }]
	]);
	writePseudo.("beattrack", [
		[\beattrack_auto, { Out.kr(0, BeatTrack.kr(SinOsc.ar(1))) }],
		[\beattrack_chain, { Out.kr(0, BeatTrack.kr(FFT(LocalBuf(1024), SinOsc.ar(1)), 0.5)) }]
	]);
	writePseudo.("localin", [
		[\localin_ar, {
			var in = LocalIn.ar(2);
			LocalOut.ar(in * 0.5);
			Out.ar(0, in)
		}],
		[\localin_default, {
			var in = LocalIn.ar(3, [0.1, 0.2]);
			LocalOut.ar(in);
			Out.ar(0, in)
		}],
		[\localin_kr, {
			var in = LocalIn.kr(1, 0.5);
			LocalOut.kr(in + 1);
			Out.kr(0, in)
		}],
		[\localin_zero, {
			var in = LocalIn.ar(2);
			LocalOut.ar([in[0], 0]);
			Out.ar(0, in)
		}]
	]);
	writePseudo.("poll", [
		[\poll_kr, { Poll.kr(10, SinOsc.kr(1)) }],
		[\poll_label, { Poll.ar(Impulse.ar(2), SinOsc.ar(1), "freq", 7) }],
		[\poll_symbol, { Poll.kr(Impulse.kr(2), SinOsc.kr(1), \amp) }],
		[\poll_proxy, { Poll.kr(10, Pan2.kr(SinOsc.kr(1))) }],
		[\poll_integer, { Poll.kr(10, 440) }],
		[\poll_float, { Poll.kr(10, 0.5) }],
		[\poll_new, { |freq = 440| Poll.new(10, freq) }],
		[\poll_new_scalar, { |i_x = 1| Poll.new(10, i_x) }],
		[\poll_out, { Out.ar(0, Poll.ar(10, SinOsc.ar(1))) }],
		[\poll_bytes, { Poll.kr(10, SinOsc.kr(1), "é") }]
	]);
	writePseudo.("symbols", [
		[\sendreply_bytes, { SendReply.kr(Impulse.kr(1), '/é', SinOsc.kr(1)) }]
	]);

	writeArithmetic.();

	"fixtures.scd: before_optimize holds %".format(rewritten).postln;
	"fixtures.scd: wrote % golden files to %".format(written, fixtures).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
