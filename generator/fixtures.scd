// The golden files of tests/fixtures: synth definitions as sclang 3.14.1
// builds and writes them, from its core class library alone. Run it from
// the root of the repository, with absolute paths, since ~/.bin/sclang
// changes the directory:
//
//   sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/fixtures.scd"
//
// It exits with 1 if sclang is not 3.14.1, if a class or a method comes from
// outside the core class library, or if a graph that the crate refuses does
// not break in sclang. It also writes tests/fixtures/arithmetic.json.
(
var root = thisProcess.nowExecutingPath.dirname.dirname;
var fixtures = root +/+ "tests" +/+ "fixtures";
var library = Platform.classLibraryDir;
// sclang 3.14.1 runs the code after 1.exit (measured), so the throw stops it.
var fail = { |message|
	("fixtures.scd: " ++ message).postln;
	1.exit;
	Error(message).throw;
};
var core = { |item| item.filenameSymbol.asString.beginsWith(library) };
var build, write, written = 0;
var writeCases, refused, writeArithmetic;

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

	// SynthDef:build.
	build = { |name, func, rates, variants, class, prependArgs, metadata|
		var def = (class ? SynthDef).newCopyArgs(
			name: name.asSymbol,
			variants: variants,
			metadata: metadata ? (),
			children: Array(64)
		);
		protect {
			def.initBuild;
			def.buildUgenGraph(func, rates, prependArgs);
			def.finishBuild;
		} {
			UGen.buildSynthDef = nil;
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

	// One file of full builds. A case is a name, a function, and an Event
	// of the rates, the variants, the prependArgs and the metadata.
	writeCases = { |name, cases|
		write.(name, cases.collect { |case|
			var options = case[2] ? ();
			build.(case[0], case[1], options[\rates], options[\variants], nil,
				options[\prependArgs], options[\metadata])
		});
	};

	// A graph that the crate refuses must break in sclang: the build or the
	// write throws, an AbstractOut goes missing, a UGen comes twice, a name
	// points past the end of the parameters, or a variant writes no values.
	refused = { |name, func, variants|
		var def = SynthDef.newCopyArgs(
			name: name.asSymbol,
			variants: variants,
			metadata: (),
			children: Array(64)
		);
		var before, kept, bytes, plain, broken = false;
		protect {
			try {
				def.initBuild;
				def.buildUgenGraph(func, nil, nil);
				before = def.children.copy;
				def.finishBuild;
			} { broken = true };
		} {
			UGen.buildSynthDef = nil;
		};
		if (broken.not) {
			// The sort leaves nil when it drops every UGen (measured).
			kept = def.children ? [];
			broken = before.select(_.isKindOf(AbstractOut)).any { |out| kept.includes(out).not }
				or: { kept.size != kept.asSet.size }
				or: { (def.allControlNames ? []).any { |cn| cn.index > def.controls.size } };
		};
		if (broken.not) {
			try { bytes = def.asBytes } { broken = true };
		};
		// A variant on a name without a default writes the count of the
		// variants and stops (measured), so the bytes keep the size of the
		// definition without variants.
		if (broken.not and: { variants.size > 0 }) {
			def.variants = nil;
			plain = def.asBytes;
			broken = bytes.size == plain.size;
		};
		if (broken.not) {
			fail.("% does not break in sclang".format(name));
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
	writeCases.("pmosc", [
		[\pmosc_ar, { Out.ar(0, PMOsc.ar(440, 220, 2)) }],
		[\pmosc_kr, { |car = 10| Out.kr(0, PMOsc.kr(car, 3, 1.5, 0.25, 0.5, 0.5)) }]
	]);
	writeCases.("changed", [
		[\changed_kr, { Out.kr(0, Changed.kr(SinOsc.kr(1))) }],
		[\changed_ar, { Out.ar(0, Changed.ar(SinOsc.ar(2), 0.1)) }],
		[\changed_array, { Out.kr(0, Changed.kr(SinOsc.kr([1, 2]))) }]
	]);
	writeCases.("silent", [
		[\silent_1, { Out.ar(0, Silent.ar) }],
		[\silent_2, { Out.ar(0, Silent.ar(2)) }]
	]);
	writeCases.("dstutter", [
		[\dstutter, { Out.kr(0, Demand.kr(Impulse.kr(1), 0, Dstutter(2, Dseq([1, 2], inf)))) }]
	]);
	writeCases.("tchoose", [
		[\tchoose_kr, { Out.kr(0, TChoose.kr(Impulse.kr(1), [SinOsc.kr(1), SinOsc.kr(2)])) }],
		[\tchoose_ar, {
			Out.ar(0, TChoose.ar(Impulse.ar(1), [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}]
	]);
	writeCases.("twchoose", [
		[\twchoose_kr, {
			Out.kr(0, TWChoose.kr(Impulse.kr(1), [SinOsc.kr(1), SinOsc.kr(2)], [0.3, 0.7]))
		}],
		[\twchoose_ar, {
			Out.ar(0, TWChoose.ar(Impulse.ar(1), [SinOsc.ar(1), SinOsc.ar(2)], [1, 3], 1))
		}]
	]);
	writeCases.("tap", [
		[\tap_1, { Out.ar(0, Tap.ar(0, 1, 0.2)) }],
		[\tap_2, { |time = 0.5| Out.ar(0, Tap.ar(1, 2, time)) }]
	]);
	// With numbers, LinLin computes its result in the language.
	writeCases.("linlin", [
		[\linlin_ar, { Out.ar(0, LinLin.ar(SinOsc.ar(1), -1, 1, 100, 200)) }],
		[\linlin_kr, { Out.kr(0, LinLin.kr(SinOsc.kr(1), -1, 1, 100, 200)) }],
		[\linlin_numbers, {
			Out.kr(0, [LinLin.kr(0.5, 0, 1, 10, 20), LinLin.ar(0.25, 0, 1, 0, 4)] * SinOsc.kr(1))
		}],
		[\linlin_ugens, { |lo = 100| Out.kr(0, LinLin.kr(SinOsc.kr(1), -1, 1, lo, 200)) }]
	]);
	writeCases.("soundin", [
		[\soundin_0, { Out.ar(0, SoundIn.ar(0)) }],
		[\soundin_01, { Out.ar(0, SoundIn.ar([0, 1])) }],
		[\soundin_02, { Out.ar(0, SoundIn.ar([0, 2])) }],
		[\soundin_mul, { Out.ar(0, SoundIn.ar(2, 0.5)) }],
		[\soundin_ugen, { |bus = 1| Out.ar(0, SoundIn.ar(bus)) }]
	]);
	writeCases.("selectx", [
		[\selectx_ar, { |which = 0|
			Out.ar(0, SelectX.ar(which, [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}],
		[\selectx_kr, { Out.kr(0, SelectX.kr(0.5, [SinOsc.kr(1), SinOsc.kr(2)])) }]
	]);
	writeCases.("blowpass4", [
		[\blowpass4, { Out.ar(0, BLowPass4.ar(SinOsc.ar(1), 800, 0.5)) }],
		[\blowpass4_ugens, { |freq = 800, rq = 0.5|
			Out.ar(0, BLowPass4.ar(SinOsc.ar(1), freq, rq, 0.5))
		}]
	]);
	writeCases.("bhipass4", [
		[\bhipass4, { Out.ar(0, BHiPass4.ar(SinOsc.ar(1), 800, 0.5)) }]
	]);

	writeCases.("sum", [
		[\sum3_fold, { Out.ar(0, Sum3(SinOsc.ar(1), SinOsc.ar(2), 0)) }],
		[\sum4_fold, { Out.ar(0, Sum4(SinOsc.ar(1), 0, SinOsc.kr(2), SinOsc.ar(3))) }],
		[\sum3_array, { Out.ar(0, Sum3([SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3), SinOsc.ar(4))) }]
	]);
	writeCases.("mix", [
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

	writeCases.("numchannels", [
		[\numchannels_mix, { Out.ar(0, NumChannels.ar(SinOsc.ar([1, 2, 3, 4]), 2)) }],
		[\numchannels_cut, { Out.ar(0, NumChannels.ar(SinOsc.ar([1, 2, 3]), 2, false)) }],
		[\numchannels_dup, { Out.ar(0, NumChannels.ar(SinOsc.ar(1), 3)) }],
		[\numchannels_one, { Out.ar(0, NumChannels.ar([SinOsc.ar(1)], 1)) }]
	]);
	writeCases.("selectxfocus", [
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
	writeCases.("multitap", [
		[\multitap, { Out.ar(0, MultiTap.ar(`[0.1, 0.2], `[0.5, 0.3], SinOsc.ar(1), bufnum: 0)) }],
		[\multitap_levels, {
			Out.ar(0, MultiTap.ar(`[0.1, 0.2, 0.3], `[0.5], SinOsc.ar(1), 0.5, 0, 1))
		}]
	]);
	writeCases.("splay", [
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
	writeCases.("dynklank", [
		[\dynklank_2, { Out.ar(0, DynKlank.ar(`[[800, 1071]], Impulse.ar(2))) }],
		[\dynklank_3, { Out.ar(0, DynKlank.ar(`[[800, 1071, 1353], nil, [1, 1, 1]], Impulse.ar(2))) }],
		[\dynklank_amps, { Out.ar(0, DynKlank.ar(`[[800, 1071], [0.5, 0.3]], Impulse.ar(2))) }],
		[\dynklank_scale, { |scale = 1|
			Out.ar(0, DynKlank.ar(`[[800, 1071], nil, [1, 2]], Impulse.ar(2), scale))
		}],
		[\dynklank_kr, { Out.kr(0, DynKlank.kr(`[[8, 10]], Impulse.kr(2))) }],
		[\dynklank_expand, { Out.ar(0, DynKlank.ar(`[[800, 1071]], Impulse.ar([2, 3]))) }]
	]);
	writeCases.("dynklang", [
		[\dynklang_2, { Out.ar(0, DynKlang.ar(`[[800, 1000]])) }],
		[\dynklang_amps, { Out.ar(0, DynKlang.ar(`[[800, 1000], [0.3, 0.3], [pi, pi]])) }],
		[\dynklang_kr, { Out.kr(0, DynKlang.kr(`[[1, 2], nil, [0.5, 0.25]], 2, 1)) }]
	]);
	writeCases.("unpackfft", [
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
	writeCases.("delays", [
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
	writeCases.("linxfade2", [
		[\linxfade2_level, { Out.ar(0, LinXFade2.ar(SinOsc.ar(1), SinOsc.ar(2), 0.5, 0.5)) }],
		[\linxfade2_one, { Out.kr(0, LinXFade2.kr(SinOsc.kr(1), SinOsc.kr(2), -0.5)) }],
		[\linxfade2_ugen, { |level = 0.5|
			Out.ar(0, LinXFade2.ar(SinOsc.ar(1), SinOsc.ar(2), 0, level))
		}]
	]);

	writeCases.("companderd", [
		[\companderd, { Out.ar(0, CompanderD.ar(SinOsc.ar(1))) }],
		[\companderd_number, { Out.ar(0, CompanderD.ar(0.5, 0.4, 1, 0.5, 0.02, 0.1, 0.5)) }]
	]);
	writeCases.("hilbertfir", [
		[\hilbertfir, { Out.ar(0, HilbertFIR.ar(SinOsc.ar(1), 0)) }]
	]);
	writeCases.("linselectx", [
		[\linselectx_ar, { |which = 0|
			Out.ar(0, LinSelectX.ar(which, [SinOsc.ar(1), SinOsc.ar(2), SinOsc.ar(3)]))
		}],
		[\linselectx_kr, { Out.kr(0, LinSelectX.kr(0.5, [SinOsc.kr(1), SinOsc.kr(2)])) }]
	]);
	writeCases.("splayaz", [
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
	writeCases.("lag", [
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
	writeCases.("outzeros", [
		[\outzeros_one, { Out.ar(0, 0) }],
		[\outzeros_two, { Out.ar(0, [SinOsc.ar(1), 0, 0]) }],
		[\outzeros_negative, { Out.ar(0, [-0.0, SinOsc.ar(1)]) }],
		[\outzeros_replace, { ReplaceOut.ar(0, [0, SinOsc.ar(1)]) }],
		[\outzeros_offset, { OffsetOut.ar(0, 0) }],
		[\outzeros_x, { XOut.ar(0, 0.5, [SinOsc.ar(1), 0]) }],
		[\outzeros_kr, { Out.kr(0, 0) }]
	]);
	writeCases.("localbuf", [
		[\localbuf_fft, { Out.ar(0, IFFT(FFT(LocalBuf(2048), SinOsc.ar(1)))) }],
		[\localbuf_two, {
			var a = FFT(LocalBuf(1024), SinOsc.ar(1));
			var b = FFT(LocalBuf(2048), SinOsc.ar(2));
			Out.ar(0, [IFFT(a), IFFT(b)])
		}],
		[\localbuf_array, { Out.ar(0, IFFT(FFT(LocalBuf([512, 1024]), SinOsc.ar(1)))) }],
		[\localbuf_channels, { Out.ar(0, PlayBuf.ar(2, LocalBuf(1024, 2))) }]
	]);
	writeCases.("beattrack", [
		[\beattrack_auto, { Out.kr(0, BeatTrack.kr(SinOsc.ar(1))) }],
		[\beattrack_chain, { Out.kr(0, BeatTrack.kr(FFT(LocalBuf(1024), SinOsc.ar(1)), 0.5)) }]
	]);
	writeCases.("localin", [
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
	writeCases.("poll", [
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
	writeCases.("symbols", [
		[\sendreply_bytes, { SendReply.kr(Impulse.kr(1), '/é', SinOsc.kr(1)) }]
	]);

	// The rules of their own, Klank, Klang and PingPong, of 0.5.
	writeCases.("bufwr", [
		[\bufwr_ar, {
			var buf = LocalBuf(1024, 2);
			BufWr.ar(SinOsc.ar([1, 2]), buf, Phasor.ar(0, 1, 0, 1024))
		}],
		[\bufwr_kr, {
			var buf = LocalBuf(64);
			BufWr.kr(SinOsc.kr(1), buf, Phasor.kr(0, 1, 0, 64), 0)
		}]
	]);
	writeCases.("duty", [
		[\duty_kr, { Out.kr(0, Duty.kr(Dseq([0.1, 0.2], inf), 0, Dseq([1, 2, 3], inf))) }],
		[\duty_ar, { Out.ar(0, Duty.ar(Dseq([0.1, 0.2], inf), Impulse.ar(1), Dseq([1, 2], inf))) }],
		[\duty_demand_reset, {
			Out.kr(0, Duty.kr(Dseq([0.1], inf), Dseq([1, 0], inf), Dwhite(0, 1)))
		}],
		[\duty_number, { Out.kr(0, Duty.kr(0.5, 0, Dseq([1, 2], inf))) }],
		[\tduty_ar, { Out.ar(0, TDuty.ar(Dseq([0.1, 0.2], inf), 0, 1, 0, 1)) }]
	]);
	writeCases.("peak", [
		[\peak_kr_audio, { Out.kr(0, Peak.kr(SinOsc.ar(1), Impulse.kr(1))) }],
		[\peak_ar, { Out.ar(0, Peak.ar(SinOsc.ar(1), Impulse.ar(2))) }],
		[\runningmax_kr, { Out.kr(0, RunningMax.kr(SinOsc.kr(1))) }],
		[\runningmin_ar, { Out.ar(0, RunningMin.ar(SinOsc.ar(1))) }]
	]);
	writeCases.("t2k", [
		[\t2k, { Out.kr(0, T2K.kr(Impulse.ar(10))) }]
	]);
	writeCases.("klank", [
		[\klank_3, {
			Out.ar(0, Klank.ar(`[[800, 1071, 1353], [0.3, 0.2, 0.1], [1, 1, 1]], Impulse.ar(2)))
		}],
		[\klank_amps, { Out.ar(0, Klank.ar(`[[800, 1071], [0.5, 0.3]], Impulse.ar(2))) }],
		[\klank_times, { Out.ar(0, Klank.ar(`[[800, 1071], nil, [1, 2]], Impulse.ar(2))) }],
		[\klank_wrap, { Out.ar(0, Klank.ar(`[[800, 1071, 1353], [0.5, 0.3]], Impulse.ar(2))) }],
		[\klank_scale, { |scale = 1| Out.ar(0, Klank.ar(`[[800, 1071]], Impulse.ar(2), scale)) }],
		[\klank_expand, { Out.ar(0, Klank.ar(`[[800, 1071]], Impulse.ar([2, 3]))) }]
	]);
	writeCases.("klang", [
		[\klang_3, { Out.ar(0, Klang.ar(`[[800, 1000], [0.3, 0.3], [pi, pi]])) }],
		[\klang_default, { Out.ar(0, Klang.ar(`[[800, 1000, 1200]])) }],
		[\klang_scale, { Out.ar(0, Klang.ar(`[[800, 1000]], SinOsc.kr(0.1, 0, 0.1, 1))) }]
	]);
	writeCases.("pingpong", [
		[\pingpong, {
			var buf = LocalBuf(44100, 2);
			Out.ar(0, PingPong.ar(buf, [SinOsc.ar(1), SinOsc.ar(2)], 0.25))
		}],
		[\pingpong_rotate, {
			var buf = LocalBuf(44100, 2);
			Out.ar(0, PingPong.ar(buf, [SinOsc.ar(1), SinOsc.ar(2)], 0.25, 0.5, 0))
		}],
		[\pingpong_negative, {
			var buf = LocalBuf(44100, 3);
			Out.ar(0, PingPong.ar(buf, SinOsc.ar([1, 2, 3]), 0.25, 0.7, -1))
		}],
		[\pingpong_ugens, { |fb = 0.5|
			var buf = LocalBuf(44100, 2);
			Out.ar(0, PingPong.ar(buf, [SinOsc.ar(1), SinOsc.ar(2)], SinOsc.kr(0.1, 0, 0.1, 0.2), fb))
		}],
		[\pingpong_bufnum, { Out.ar(0, PingPong.ar(0, [SinOsc.ar(1), SinOsc.ar(2)], 0.25)) }]
	]);

	// The arrays inside array arguments, of 0.6.
	writeCases.("nestedout", [
		[\nestedout_two, { Out.ar(0, [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]) }],
		[\nestedout_bus, { Out.ar([0, 2], [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]) }],
		[\nestedout_pan2, { Out.ar(0, Pan2.ar(SinOsc.ar([440, 660]), [-0.5, 0.5])) }],
		[\nestedout_deep, {
			Out.ar(0, [[[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)], SinOsc.ar(4)])
		}],
		[\nestedout_zeros, { Out.ar(0, [[0, SinOsc.ar(1)], 0]) }],
		[\nestedout_kr, { Out.kr(0, [[SinOsc.kr(1), SinOsc.kr(2)], 0]) }],
		[\nestedout_x, { XOut.ar(0, 0.5, [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]) }]
	]);
	writeCases.("nestedwire", [
		[\nestedwire_bufwr, {
			var buf = LocalBuf(1024, 2);
			BufWr.ar([[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)], buf, Phasor.ar(0, 1, 0, 1024))
		}],
		[\nestedwire_recordbuf, {
			RecordBuf.ar([[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)], LocalBuf(1024, 2))
		}],
		[\nestedwire_demand, {
			var lists = [[Dseq([1, 2], inf), Dseq([3, 4], inf)], Dwhite(0, 1, inf)];
			Out.kr(0, Demand.kr(Impulse.kr(1), 0, lists))
		}],
		[\nestedwire_dseq, { Out.kr(0, Demand.kr(Impulse.kr(1), 0, Dseq([[1, 2], 3], inf))) }],
		[\nestedwire_select, {
			Out.ar(0, Select.ar(SinOsc.kr(1), [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]))
		}],
		[\nestedwire_twindex, { Out.kr(0, TWindex.kr(Impulse.kr(1), [[0.2, 0.8], 0.5])) }]
	]);
	writeCases.("nestedreply", [
		[\nestedreply_two, {
			var values = [[SinOsc.kr(1), SinOsc.kr(2)], [SinOsc.kr(3), SinOsc.kr(4)]];
			SendReply.kr(Impulse.kr(1), '/a', values)
		}],
		[\nestedreply_wrap, {
			SendReply.kr(Impulse.kr([1, 2, 3]), '/b', [[SinOsc.kr(1), SinOsc.kr(2)], SinOsc.kr(3)])
		}],
		[\nestedreply_first, {
			SendReply.kr(Impulse.kr(1), '/c', [SinOsc.kr(1), [SinOsc.kr(2), SinOsc.kr(3)]])
		}],
		[\nestedreply_ar, { SendReply.ar(Impulse.ar(1), '/d', [[SinOsc.ar(1)], [SinOsc.ar(2)]]) }]
	]);
	writeCases.("nestedenv", [
		[\nestedenv_two, {
			var envelopes = [[0, 2, -99, -99, 1, 0.1, 1, 0, 0, 0.9, 1, 0], [0, 1, -99, -99, 1, 0.5, 1, 0]];
			Out.kr(0, EnvGen.kr(envelopes))
		}],
		[\nestedenv_gates, { |gate = 1|
			var envelopes = [[0, 1, -99, -99, 1, 0.1, 1, 0], [0, 1, -99, -99, 1, 0.2, 1, 0]];
			Out.ar(0, EnvGen.ar(envelopes, [gate, 1, gate * 0.5]))
		}]
	]);
	writeCases.("nestedpseudo", [
		[\nestedpseudo_selectx, {
			Out.ar(0, SelectX.ar(SinOsc.kr(1), [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]))
		}],
		[\nestedpseudo_linselectx, {
			Out.kr(0, LinSelectX.kr(SinOsc.kr(1), [[SinOsc.kr(2), SinOsc.kr(3)], SinOsc.kr(4)]))
		}],
		[\nestedpseudo_selectxfocus, {
			Out.ar(0, SelectXFocus.ar(SinOsc.kr(1), [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]))
		}],
		[\nestedpseudo_tchoose, {
			Out.ar(0, TChoose.ar(Impulse.ar(1), [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]))
		}],
		[\nestedpseudo_twchoose, {
			var weights = [[0.2, 0.8], 0.5];
			Out.ar(0, TWChoose.ar(Impulse.ar(1), [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)], weights))
		}],
		[\nestedpseudo_pingpong, {
			var buf = LocalBuf(44100, 2);
			Out.ar(0, PingPong.ar(buf, [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)], 0.25))
		}],
		[\nestedpseudo_multitap, {
			var in = [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)];
			Out.ar(0, MultiTap.ar(`[0.1, 0.2], `[0.5, 0.3], in, bufnum: 0))
		}],
		[\nestedpseudo_multitap_times, {
			Out.ar(0, MultiTap.ar(`[[0.1, 0.2], 0.3], `[0.5], SinOsc.ar(1), bufnum: 0))
		}],
		[\nestedpseudo_localin, { Out.ar(0, LocalIn.ar(2, [[0.1, 0.2], 0.3])) }],
		[\nestedpseudo_numchannels, {
			Out.ar(0, NumChannels.ar([[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3), SinOsc.ar(4)], 2))
		}],
		[\nestedpseudo_splayaz, {
			Out.ar(0, SplayAz.ar(2, [[SinOsc.ar(1), SinOsc.ar(2)], SinOsc.ar(3)]))
		}]
	]);

	// optimizeGraph and the copies of PV chains, of 0.7. A function of its
	// own: sclang 3.14.1 refuses one function with more literals, with
	// "Selector table too big".
	{
		writeCases.("optimize", [
			[\optimize_muladd, { Out.ar(0, SinOsc.ar * 0.5 + 0.25) }],
			[\optimize_muladd_swap, { Out.ar(0, 0.5 * SinOsc.ar + 0.25) }],
			[\optimize_muladd_right, { Out.ar(0, 0.25 + (SinOsc.ar * 0.5)) }],
			[\optimize_args, { |amp = 0.1| Out.ar(0, SinOsc.ar * amp + 0.5) }],
			[\optimize_muladd_kr, { Out.kr(0, SinOsc.kr * LFNoise1.kr + 2) }],
			[\optimize_shared, {
				var x = SinOsc.ar * 0.5;
				Out.ar(0, [x + 0.25, x])
			}],
			[\optimize_kr_audio, { Out.ar(0, (SinOsc.kr * 2) + SinOsc.ar) }],
			[\optimize_sum3, { Out.ar(0, SinOsc.ar(1) + SinOsc.ar(2) + SinOsc.ar(3)) }],
			[\optimize_sum3_right, { Out.ar(0, SinOsc.ar(1) + (SinOsc.ar(2) + SinOsc.ar(3))) }],
			[\optimize_sum4, { Out.ar(0, SinOsc.ar(1) + SinOsc.ar(2) + SinOsc.ar(3) + SinOsc.ar(4)) }],
			[\optimize_sum5, {
				Out.ar(0, SinOsc.ar(1) + SinOsc.ar(2) + SinOsc.ar(3) + SinOsc.ar(4) + SinOsc.ar(5))
			}],
			[\optimize_twice, {
				var a = SinOsc.ar(1) + SinOsc.ar(2);
				Out.ar(0, a + a)
			}],
			[\optimize_addneg, { Out.ar(0, SinOsc.ar(1) + SinOsc.ar(2).neg) }],
			[\optimize_negadd, { Out.ar(0, SinOsc.ar(1).neg + SinOsc.ar(2)) }],
			[\optimize_subneg, { Out.ar(0, (SinOsc.ar(1) + SinOsc.ar(2)) - SinOsc.ar(3).neg) }],
			[\optimize_demand, {
				var sum = Dseq([1, 2], inf) + Dseq([3, 4], inf) + Dseq([5], inf);
				Out.kr(0, Demand.kr(Impulse.kr(1), 0, sum))
			}]
		]);
		writeCases.("deadcode", [
			[\deadcode_chain, {
				SinOsc.ar(SinOsc.ar(1) * 100);
				Out.ar(0, SinOsc.ar)
			}],
			[\deadcode_proxy, {
				DC.ar(1) * 2;
				Out.ar(0, SinOsc.ar)
			}],
			[\deadcode_detectsilence, {
				DetectSilence.ar(SinOsc.ar(2));
				Out.ar(0, SinOsc.ar)
			}],
			[\deadcode_arg, { |freq = 440|
				freq * 2;
				Out.ar(0, SinOsc.ar)
			}],
			[\deadcode_constant, {
				SinOsc.ar(1234);
				Out.ar(0, SinOsc.ar)
			}],
			[\deadcode_revisit, {
				var x = SinOsc.ar(1);
				var m = x * 2;
				var a = m + 1;
				m * a;
				Out.ar(0, a)
			}],
			[\deadcode_revisit_swapped, {
				var x = SinOsc.ar(1);
				var m = x * 2;
				var a = m + 1;
				a * m;
				Out.ar(0, a)
			}],
			[\deadcode_readers, {
				var x = SinOsc.ar(1);
				var s = x * 2 + 1;
				x * 3;
				x * 4;
				Out.ar(0, s)
			}]
		]);
		writeCases.("pvcopy", [
			[\pvcopy_three, {
				var chain = FFT(LocalBuf(512), SinOsc.ar);
				var pvs = [PV_MagAbove(chain, 1), PV_MagBelow(chain, 1), PV_MagClip(chain, 1)];
				Out.ar(0, pvs.collect { |pv| IFFT(pv) })
			}],
			[\pvcopy_slots, {
				var chain = FFT(LocalBuf(512), SinOsc.ar);
				Out.ar(0, [IFFT(PV_Mul(chain, chain)), IFFT(chain)])
			}],
			[\pvcopy_deep, {
				var chain = PV_MagAbove(FFT(LocalBuf(512), SinOsc.ar), 1);
				Out.ar(0, IFFT(chain) + IFFT(chain))
			}],
			[\pvcopy_analysis, {
				var chain = FFT(LocalBuf(512), SinOsc.ar);
				Out.ar(0, IFFT(PV_MagAbove(chain, 1)) * SpecCentroid.kr(chain))
			}],
			[\pvcopy_buffer, {
				var buf = LocalBuf(16);
				var chain = FFT(0, SinOsc.ar);
				Out.ar(0, [IFFT(PV_MagAbove(chain, 1)), IFFT(PV_MagBelow(chain, 1)), PlayBuf.ar(1, buf)])
			}]
		]);
		[
			[\refused_fft_buffer, {
				var chain = FFT(0, SinOsc.ar);
				Out.ar(0, IFFT(PV_MagAbove(chain, 1)) + IFFT(PV_MagBelow(chain, 1)))
			}],
			[\refused_packfft, {
				var chain = PackFFT(FFT(LocalBuf(512), SinOsc.ar), 512, [1, 0]);
				Out.ar(0, IFFT(chain) + IFFT(chain))
			}],
			[\refused_ffttrigger, {
				var chain = FFTTrigger(LocalBuf(512));
				Out.ar(0, IFFT(chain) + IFFT(chain))
			}],
			[\refused_readers, {
				var x = SinOsc.ar(1);
				var s = x * 2 + 1;
				x * 3;
				x * 4;
				x * 5;
				Out.ar(0, s)
			}],
			[\refused_neg_zero, { Out.ar(0, SinOsc.ar + UnaryOpUGen('neg', 0)) }],
			[\refused_neg_numbers, {
				Out.ar(0, SinOsc.ar * BinaryOpUGen('+', 2, UnaryOpUGen('neg', 3)))
			}]
		].do { |case| refused.(case[0], case[1]) };
	}.value;

	// The controls inside the graph, of 0.8, in functions of their own for
	// the same reason.
	{
		writeCases.("controlugens", [
			[\controlugens_names, { Out.ar(0, SinOsc.ar(Control.names([\a, \b]).kr([1, 2]))) }],
			[\controlugens_names_short, { Out.ar(0, SinOsc.ar(Control.names([\a]).kr([1, 2]))) }],
			// b points at the end of the parameters, as the name of an empty
			// array does.
			[\controlugens_names_long, { Out.ar(0, SinOsc.ar(Control.names([\a, \b]).kr([1]))) }],
			[\controlugens_names_ir, { Out.ar(0, SinOsc.ar(Control.names([\a, \b]).ir([1, 2]))) }],
			[\controlugens_trig_names, { Out.ar(0, SinOsc.ar(TrigControl.names([\t]).kr(1))) }],
			[\controlugens_audio_names, { Out.ar(0, SinOsc.ar(AudioControl.names([\in]).ar(0))) }],
			[\controlugens_after_arg, { |freq = 440|
				Out.ar(0, SinOsc.ar(freq) * Control.names([\amp]).kr(0.1))
			}],
			[\controlugens_names_trig, {
				Control.names([\a, \b]);
				Out.ar(0, SinOsc.ar(TrigControl.kr([1, 2])))
			}],
			[\controlugens_unnamed, { Out.ar(0, SinOsc.ar(Control.kr([1, 2]))) }],
			[\controlugens_nested, { Out.ar(0, SinOsc.ar(Control.kr([[1, 2], 3]).flat)) }],
			[\controlugens_trig_ir, { Out.ar(0, SinOsc.ar(TrigControl.ir(1))) }],
			[\lagcontrol_two, { Out.ar(0, SinOsc.ar(LagControl.kr([1, 2], [0.1, 0.2]))) }],
			[\lagcontrol_number, { Out.ar(0, SinOsc.ar(LagControl.kr([1, 2], 0.1))) }],
			[\lagcontrol_seventeen, { Out.kr(0, LagControl.kr((1..17), 0.1)) }],
			[\lagcontrol_nested, {
				Out.ar(0, SinOsc.ar(LagControl.kr([[1, 2], 3], [0.1, 0.2]).flat))
			}],
			[\lagcontrol_arg_lag, { |lag = 0.1|
				Out.ar(0, SinOsc.ar(LagControl.kr([1, 2], [lag, 0.2])))
			}],
			[\lagcontrol_names, {
				Control.names([\a, \b]);
				Out.ar(0, SinOsc.ar(LagControl.kr([1, 2], [0.1, 0.2])))
			}],
			[\lagcontrol_ar, { Out.ar(0, SinOsc.ar(LagControl.ar(1, 0.1))) }],
			[\lagcontrol_ar_two, { Out.ar(0, SinOsc.ar(LagControl.ar([1, 2], [0.1, 0.2]))) }],
			[\controlugens_variant, {
				Out.ar(0, SinOsc.ar(Control.names([\a, \b]).kr([1, 2])))
			}, (variants: (x: [b: 5]))]
		]);
		[
			[\refused_control_empty, { Control.kr([]); Out.ar(0, SinOsc.ar) }],
			[\refused_lagcontrol_sizes, { Out.ar(0, SinOsc.ar(LagControl.kr([1, 2], [0.1]))) }],
			[\refused_names_past, { Out.ar(0, SinOsc.ar(Control.names([\a, \b, \c]).kr([1]))) }],
			[\refused_variant_default, {
				Out.ar(0, SinOsc.ar(Control.names([\a, \b]).kr([1, 2])))
			}, (x: [a: 3])],
			[\refused_variant_past, {
				Out.ar(0, SinOsc.ar(Control.names([\a, \b]).kr([1, 2])))
			}, (x: [b: [5, 6]])]
		].do { |case| refused.(case[0], case[1], case[2]) };
	}.value;
	{
		writeCases.("defwrap", [
			[\defwrap_arg, { |freq = 440|
				Out.ar(0, SynthDef.wrap({ |amp = 0.1| SinOsc.ar(freq) * amp }))
			}],
			[\defwrap_prepend, {
				Out.ar(0, SynthDef.wrap({ |sig, amp = 0.1| sig * amp }, nil, [SinOsc.ar]))
			}],
			[\defwrap_rates, {
				Out.ar(0, SynthDef.wrap({ |amp = 0.1, i_x = 2| SinOsc.ar(i_x) * amp }, [0.2]))
			}],
			[\defwrap_same_name, { |amp = 0.1|
				Out.ar(0, SynthDef.wrap({ |amp = 0.2| SinOsc.ar * amp }) * amp)
			}],
			[\defwrap_twice, {
				Out.ar(0, SynthDef.wrap({ |a = 1| SinOsc.ar(a) }) + SynthDef.wrap({ |b = 2| SinOsc.ar(b) }))
			}],
			// a and c share the index 0. After the wrap, b is the last name
			// again, so Control.kr gives it a default, which the variant needs.
			[\defwrap_names, {
				Control.names([\a, \b]);
				SynthDef.wrap({ |c = 7| c });
				Out.ar(0, SinOsc.ar(Control.kr([1, 2])))
			}, (variants: (x: [b: 5]))],
			[\defwrap_variant, { |amp = 0.1|
				Out.ar(0, SynthDef.wrap({ |amp = 0.2| SinOsc.ar * amp }))
			}, (variants: (x: [amp: 0.5]))],
			[\prepend, { |bus, freq = 440| Out.ar(bus, SinOsc.ar(freq)) }, (prependArgs: [0])],
			[\prepend_array, { |bus, freq = 440|
				Out.ar(bus, SinOsc.ar(freq))
			}, (prependArgs: [[0, 1]])]
		]);
	}.value;
	{
		// One argument without a default per spec of the table.
		var names = Spec.specs.keys.asArray.sort;
		var func = ("{ |" ++ names.join(", ") ++ "| Out.kr(0, [" ++ names.join(", ") ++ "]) }")
			.interpret;
		writeCases.("specs", [[\specs_args, func, (metadata: (specs: ()))]]);
	}.value;
	{
		var cases = List.new;
		{
			cases.addAll([
				[\namedcontrol_one, { Out.ar(0, SinOsc.ar(\freq.kr(440))) }],
				[\namedcontrol_twice, { Out.ar(0, SinOsc.ar(\freq.kr(440)) + SinOsc.ar(\freq.kr)) }],
				[\namedcontrol_after_arg, { |amp = 0.1| Out.ar(0, SinOsc.ar(\freq.kr(440)) * amp) }],
				[\namedcontrol_arg_name, { |freq = 440|
					Out.ar(0, SinOsc.ar(\freq.kr(220)) + SinOsc.ar(freq))
				}],
				[\namedcontrol_rates, { Out.ar(0, SinOsc.ar(\a.ir(1)) * \b.tr(2) + \c.ar(3) + \d.kr(4)) }],
				[\namedcontrol_prefixes, { Out.ar(0, SinOsc.ar(\i_a.kr(1)) * \t_b.kr(2) + \a_c.kr(3)) }],
				[\namedcontrol_prefix_rate, {
					Out.ar(0, SinOsc.ar(\i_x.ar(1)) + \t_y.ir(1) + \i_z.tr(1) + \a_w.ir(1))
				}],
				[\namedcontrol_new, {
					Out.ar(0, SinOsc.ar(NamedControl(\a, 1, \audio)) + NamedControl(\b, 2)
						+ NamedControl(\c, 3, \foo) + NamedControl(\d, 4, \trigger)
						+ NamedControl(\e, 5, \scalar))
				}],
				[\namedcontrol_array, { Out.ar(0, SinOsc.ar(\freqs.kr([440, 660]))) }],
				[\namedcontrol_nested, { Out.ar(0, SinOsc.ar(\x.kr([[1, 2], [3, 4]]).flat)) }],
				[\namedcontrol_no_value, { Out.ar(0, SinOsc.ar(\freq.kr)) }],
				[\namedcontrol_one_value, { Out.ar(0, SinOsc.ar(\x.kr([440]))) }]
			]);
		}.value;
		{
			cases.addAll([
				[\namedcontrol_lag, { Out.ar(0, SinOsc.ar(\freq.kr(440, 0.1))) }],
				[\namedcontrol_lag_zero, { Out.ar(0, SinOsc.ar(\freq.kr(440, 0))) }],
				[\namedcontrol_lag_array, { Out.ar(0, SinOsc.ar(\freqs.kr([440, 660], [0.1, 0.2]))) }],
				[\namedcontrol_lag_one_for_two, { Out.ar(0, SinOsc.ar(\freqs.kr([440, 660], 0.1))) }],
				[\namedcontrol_lag_seventeen, { Out.kr(0, \x.kr((1..17), 0.1)) }],
				[\namedcontrol_lag_twice, {
					Out.ar(0, SinOsc.ar(\freq.kr(440, 0.1)) + SinOsc.ar(\freq.kr(440, 0.1)))
				}],
				[\namedcontrol_lag_then_none, {
					Out.ar(0, SinOsc.ar(\freq.kr(440, 0.1)) + SinOsc.ar(\freq.kr(440)))
				}],
				[\namedcontrol_none_then_lag, {
					Out.ar(0, SinOsc.ar(\freq.kr(440)) + SinOsc.ar(\freq.kr(440, 0.1)))
				}],
				[\namedcontrol_lag_ugen, { Out.ar(0, SinOsc.ar(\freq.kr(440, \lt.kr(0.2)))) }],
				[\namedcontrol_lag_ugen_twice, {
					Out.ar(0, SinOsc.ar(\freq.kr(440, \lt.kr(0.2))) + SinOsc.ar(\freq.kr(440, \lt.kr(0.2))))
				}],
				[\namedcontrol_ugen_then_fixed, {
					Out.ar(0, SinOsc.ar(\a.kr(1, \lt.kr(0.2))) + SinOsc.ar(\a.kr(1, 0.1)))
				}],
				[\namedcontrol_fixed_ugen, { Out.ar(0, SinOsc.ar(\a.kr(1, \lt.kr(0.2), true))) }]
			]);
		}.value;
		{
			cases.addAll([
				[\namedcontrol_lag_ugens, {
					Out.ar(0, SinOsc.ar(\x.kr([1, 2], [\l1.kr(0.1), \l2.kr(0.2)])))
				}],
				[\namedcontrol_lag_ugens_one_value, {
					Out.ar(0, SinOsc.ar(\x.kr(1, [\l1.kr(0.1), \l2.kr(0.2)])))
				}],
				[\namedcontrol_lag_ugen_two_values, { Out.ar(0, SinOsc.ar(\x.kr([1, 2], \l.kr(0.1)))) }],
				[\namedcontrol_lag_mixed, { Out.ar(0, SinOsc.ar(\x.kr([1, 2], [0.1, \l.kr(0.2)]))) }],
				// A prefix drops the fixed lag.
				[\namedcontrol_prefix_lag, { Out.ar(0, SinOsc.ar(\x_freq.kr(440, 0.1)) + \a_.kr(1, 0.1)) }],
				[\namedcontrol_ir_lag, {
					Out.ar(0, SinOsc.ar(NamedControl.ir(\a, 1, 0.1)) * NamedControl.tr(\b, 1, 0.1))
				}],
				[\namedcontrol_ar_lag, { Out.ar(0, SinOsc.ar(\a.ar(1, 0.1))) }],
				[\namedcontrol_ir_lag_ugen, { Out.ar(0, SinOsc.ar(NamedControl.ir(\a, 1, \lt.kr(0.2)))) }],
				[\namedcontrol_tr_lag_ugen, { Out.ar(0, SinOsc.ar(\t_x.kr(1, \lt.kr(0.2)))) }],
				[\namedcontrol_ar_lag_ugen, { Out.ar(0, SinOsc.ar(\a.ar(1, \lt.kr(0.2)))) }]
			]);
		}.value;
		{
			cases.addAll([
				[\namedcontrol_wrap, {
					Out.ar(0, SinOsc.ar(\freq.kr(440)) + SynthDef.wrap({ SinOsc.ar(\freq.kr) }))
				}],
				[\namedcontrol_control_names, {
					Out.ar(0, SinOsc.ar(Control.names([\a]).kr(1)) + SinOsc.ar(\a.kr(2)))
				}],
				[\namedcontrol_spec, {
					Out.ar(0, SinOsc.ar(\freq.kr(spec: \freq)) * \amp.kr(0.5, spec: \amp)
						* \rq.kr(spec: \rq) + SinOsc.ar(\freq.kr(440)))
				}],
				[\namedcontrol_variant, { |freq = 440|
					Out.ar(0, SinOsc.ar(freq) + SinOsc.ar(\freq.kr(220)))
				}, (variants: (x: [freq: 330]))],
				// The graph of the test of sc-app3.
				[\namedcontrol_markup, {
					var freq = \freq.kr(440, 0.1);
					var osc = SinOsc.ar(freq);
					var amp = \amp.kr(0.1);
					Out.ar(0, osc * amp)
				}]
			]);
		}.value;
		writeCases.("namedcontrol", cases.asArray);
		[
			[\refused_nc_values, { Out.ar(0, SinOsc.ar(\freq.kr(440)) + SinOsc.ar(\freq.kr(220))) }],
			[\refused_nc_rate, { Out.ar(0, SinOsc.ar(\freq.kr(440)) + SinOsc.ar(\freq.ar(440))) }],
			[\refused_nc_fixed_lags, { Out.ar(0, SinOsc.ar(\a.kr(1, 0.1)) + SinOsc.ar(\a.kr(1, 0.2))) }],
			[\refused_nc_fixed_then_ugen, {
				Out.ar(0, SinOsc.ar(\a.kr(1, 0.1)) + SinOsc.ar(\a.kr(1, \lt.kr(0.2))))
			}],
			[\refused_nc_zero_fixed, { Out.ar(0, SinOsc.ar(\freqs.kr([440, 660], [0.1, 0]))) }],
			[\refused_nc_zero_lag, { Out.ar(0, SinOsc.ar(\x.kr([1, 2], [0, \l.kr(0.2)]))) }],
			[\refused_nc_spec, { Out.ar(0, SinOsc.ar(\amp.kr(spec: \nosuchspec))) }],
			[\refused_nc_second_spec, {
				Out.ar(0, SinOsc.ar(\freq.kr(spec: \freq)) + SinOsc.ar(\freq.kr(spec: \amp)))
			}],
			[\refused_nc_output_value, { Out.ar(0, SinOsc.ar(\x.kr(SinOsc.kr))) }],
			[\refused_nc_empty, { Out.ar(0, SinOsc.ar(\x.kr([]))) }],
			[\refused_nc_lag_sizes, { Out.ar(0, SinOsc.ar(\x.kr([1, 2], [0.1, 0.2, 0.3]))) }]
		].do { |case| refused.(case[0], case[1]) };
	}.value;

	// The envelopes, of 0.9, in functions of their own for the same reason.
	{
		var cases = List.new;
		{
			cases.addAll([
				[\env_new, { Out.kr(0, EnvGen.kr(Env([0, 1, 0], [1, 1]))) }],
				[\env_wrap_times, { Out.kr(0, EnvGen.kr(Env([0, 1, 0.5, 0], [1]))) }],
				[\env_default, { Out.kr(0, EnvGen.kr(Env.new)) }],
				[\env_perc, { Out.kr(0, EnvGen.kr(Env.perc)) }],
				[\env_perc_args, { Out.kr(0, EnvGen.kr(Env.perc(0.1, 2, 0.5, \exp))) }],
				[\env_adsr, { Out.kr(0, EnvGen.kr(Env.adsr)) }],
				[\env_asr, { Out.kr(0, EnvGen.kr(Env.asr)) }],
				[\env_linen, { Out.kr(0, EnvGen.kr(Env.linen)) }],
				[\env_triangle, { Out.kr(0, EnvGen.kr(Env.triangle(2))) }],
				[\env_sine, { Out.kr(0, EnvGen.kr(Env.sine)) }],
				[\env_cutoff, { Out.kr(0, EnvGen.kr(Env.cutoff)) }],
				// -100.dbamp ends the envelope.
				[\env_cutoff_exp, { Out.kr(0, EnvGen.kr(Env.cutoff(0.1, 1, \exp))) }],
				[\env_dadsr, { Out.kr(0, EnvGen.kr(Env.dadsr)) }],
				[\env_step, { Out.kr(0, EnvGen.kr(Env.step([1, 2, 3], [1, 1, 1]))) }],
				[\env_new_clear, { Out.kr(0, EnvGen.kr(Env.newClear(3))) }],
				[\env_new_clear_two, { Out.kr(0, EnvGen.kr(Env.newClear(2, 2))) }],
				// No segments make an envelope of one level.
				[\env_new_clear_none, { Out.kr(0, EnvGen.kr(Env.newClear(0))) }],
				[\env_new_clear_none_two, { Out.kr(0, EnvGen.kr(Env.newClear(0, 2))) }],
				// The triplets sort by time, and the curve 2 goes to the first
				// segment.
				[\env_xyc, { Out.kr(0, EnvGen.kr(Env.xyc([[2, 0.5], [0, 1, 2], [3, 0]]))) }],
				[\env_pairs, { Out.kr(0, EnvGen.kr(Env.pairs([[0, 1], [2, 0.5], [3, 0]], \exp))) }]
			]);
		}.value;
		{
			cases.addAll([
				[\env_shapes, {
					Out.kr(0, EnvGen.kr(Env([0, 1, 0, 1, 0, 1, 0, 1, 0], 1 ! 8,
						[\step, \hold, \lin, \exp, \sin, \wel, \sqr, \cub])))
				}],
				[\env_shape_names, {
					Out.kr(0, EnvGen.kr(Env([0, 1, 0, 1, 0, 1, 0], 1 ! 6,
						[\linear, \exponential, \sine, \welch, \squared, \cubed])))
				}],
				[\env_curves, { Out.kr(0, EnvGen.kr(Env([0, 1, 0], [1, 1], [\exp, 3]))) }],
				[\env_curve_ugen, { Out.kr(0, EnvGen.kr(Env([0, 1], [1], \c.kr(-4)))) }],
				[\env_multi, { Out.kr(0, EnvGen.kr(Env([0, [1, 2], 0], [1, 1]))) }],
				[\env_multi_curve, {
					Out.kr(0, EnvGen.kr(Env([0, 1, 0], [1, 1], [[\lin, \exp], \sin])))
				}],
				[\env_nodes, { Out.kr(0, EnvGen.kr(Env([0, 1, 0.5, 0], [1, 1, 1], \lin, 2, 1))) }],
				[\env_ugen_level, { Out.kr(0, EnvGen.kr(Env([0, \l.kr(1), 0], [1, 1]))) }],
				// p * s comes before the gate.
				[\env_adsr_ugens, {
					Out.kr(0, EnvGen.kr(Env.adsr(peakLevel: \p.kr(2), sustainLevel: \s.kr(0.5)),
						\gate.kr(1)))
				}],
				[\env_adsr_bias, { Out.kr(0, EnvGen.kr(Env.adsr(bias: \b.kr(0.1)))) }],
				[\env_circle, { Out.kr(0, EnvGen.kr(Env.circle([0, 1, 0], [1, 1, 2], \exp))) }],
				[\env_delay, { Out.kr(0, EnvGen.kr(Env.adsr.delay(0.5))) }],
				[\env_one_level, { Out.kr(0, EnvGen.kr(Env([1], []))) }],
				[\env_kr, { Out.kr(0, Env.perc.kr(2, \gate.kr(1))) }],
				[\env_ar, { Out.ar(0, Env.sine.ar) }]
			]);
		}.value;
		writeCases.("env", cases.asArray);
	}.value;
	{
		writeCases.("ienv", [
			[\ienv_new, { Out.kr(0, IEnvGen.kr(Env([0, 1, 0], [1, 2]), \i.kr(0))) }],
			[\ienv_offset, { Out.kr(0, IEnvGen.kr(Env([0, 1, 0], [1, 2], \exp, offset: 0.5), 0.5)) }],
			[\ienv_ugen_times, {
				Out.kr(0, IEnvGen.kr(Env([0, 1, 0], [\t1.kr(1), \t2.kr(2)]), 0.5))
			}],
			// t + 1 + 1 of the sum of the times becomes a Sum3.
			[\ienv_ugen_time_numbers, {
				Out.kr(0, IEnvGen.kr(Env([0, 1, 0, 1], [\t.kr(1), 1, 1]), 0.5))
			}],
			[\ienv_multi, { Out.kr(0, IEnvGen.kr(Env([0, [1, 2], 0], [1, 2]), 0.5)) }],
			// Three instances, with the channels 0, 1 and 0.
			[\ienv_multi_index, {
				Out.kr(0, IEnvGen.kr(Env([0, [1, 2], 0], [1, 2]), [0.25, 0.5, 0.75]))
			}],
			[\ienv_madd, { Out.ar(0, IEnvGen.ar(Env.perc, \i.ar(0), 0.5, 0.1)) }],
			[\ienv_xyc, { Out.kr(0, IEnvGen.kr(Env.xyc([[1, 1], [2, 0.5], [4, 0]]), 1.5)) }]
		]);
		writeCases.("varlag", [
			[\varlag_kr, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2)) }],
			[\varlag_ar, { Out.ar(0, VarLag.ar(\x.ar(1), 0.2)) }],
			[\varlag_lin, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, warp: \lin)) }],
			[\varlag_one, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, warp: 1)) }],
			[\varlag_exp, { Out.ar(0, VarLag.ar(\x.ar(1), \t.kr(0.2), 2, \exp)) }],
			[\varlag_three, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, -2, 3)) }],
			[\varlag_start, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, start: 0.5)) }],
			[\varlag_ugen_warp, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, warp: \w.kr(5))) }],
			[\varlag_array, { Out.kr(0, VarLag.kr([\x.kr(1), \y.kr(2)], 0.2)) }],
			[\varlag_array_warp, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, warp: [5, 1])) }],
			[\varlag_madd, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, mul: 2, add: 1)) }],
			[\varlag_scalar, { Out.kr(0, VarLag.kr(3, 0.2, mul: 2, add: 1)) }],
			[\varlag_time_zero, { Out.kr(0, VarLag.kr(\x.kr(1), 0)) }]
		]);
		writeCases.("envgate", [
			[\envgate_new, { Out.ar(0, SinOsc.ar * EnvGate.new) }],
			[\envgate_args, { Out.ar(0, SinOsc.ar * EnvGate(0, \g.kr(1), 0.5, 0, \lin)) }],
			[\envgate_twice, { Out.ar(0, SinOsc.ar * EnvGate.new + (SinOsc.ar(2) * EnvGate.new)) }],
			[\envgate_level, { Out.ar(0, SinOsc.ar * EnvGate(\l.kr(0.5))) }],
			// The graph of the test of sc-app3.
			[\envgate_markup, { |freq = 440|
				var lag = VarLag.kr(freq, 0.2);
				var osc = SinOsc.ar(lag);
				var env = EnvGate.new;
				Out.ar(0, osc * env)
			}]
		]);
		[
			[\refused_env_no_levels, { Out.kr(0, EnvGen.kr(Env([], []))) }],
			[\refused_env_shape, { Out.kr(0, EnvGen.kr(Env([0, 1], [1], \foo))) }],
			[\refused_env_cutoff_shape, { Out.kr(0, EnvGen.kr(Env.cutoff(0.1, 1, \foo))) }],
			[\refused_env_step, { Out.kr(0, EnvGen.kr(Env.step([1, 2], [1]))) }],
			[\refused_varlag_warp, { Out.kr(0, VarLag.kr(\x.kr(1), 0.2, warp: \foo)) }],
			[\refused_varlag_rate, { Out.ar(0, VarLag.ar(\x.kr(1), 0.2, warp: \lin)) }]
		].do { |case| refused.(case[0], case[1]) };
	}.value;

	writeArithmetic.();

	"fixtures.scd: wrote % golden files to %".format(written, fixtures).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
