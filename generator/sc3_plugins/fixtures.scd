// The golden files of tests/fixtures/sc3_plugins: synth definitions with the
// UGens of sc3-plugins 3.14.0, as sclang 3.14.1 builds and writes them. Run
// it from the root of the repository, with absolute paths, since
// ~/.bin/sclang changes the directory:
//
//   sclang -l "$PWD/generator/sc3_plugins/sclang_conf.yaml" \
//     "$PWD/generator/sc3_plugins/fixtures.scd"
//
// It reads the addable methods from generator/sc3_plugins/catalog.json, the
// cases of the bodies from tests/fixtures/sc3_plugins/bodies.json, and the
// writers from generator/goldens.scd. It exits with 1 if sclang is not
// 3.14.1, if a class or a method comes from outside the core library and the
// folder SC3plugins, or if a graph that the crate refuses does not break in
// sclang.
(
var root = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var fixtures = root +/+ "tests" +/+ "fixtures" +/+ "sc3_plugins";
var library = Platform.classLibraryDir;
var folder = LanguageConfig.includePaths.detect { |path| path.endsWith("SC3plugins") };
// sclang 3.14.1 runs the code after 1.exit (measured), so the throw stops it.
var fail = { |message|
	("sc3_plugins/fixtures.scd: " ++ message).postln;
	1.exit;
	Error(message).throw;
};
var known = { |item|
	var path = item.filenameSymbol.asString;
	path.beginsWith(library) or: { folder.notNil and: { path.beginsWith(folder) } }
};
var goldens, build, write, writeCases, refused, catalog, definition, methods = List.new;
var valueCode, callCode, bodies = List.new;

try {
	if (Main.version != "3.14.1") {
		fail.("sclang % is not 3.14.1".format(Main.version));
	};
	if (folder.isNil) { fail.("the conf names no folder SC3plugins") };
	Class.allClasses.do { |class|
		if (known.(class).not) {
			fail.("% comes from %".format(class, class.filenameSymbol));
		};
		(class.methods ? []).do { |method|
			if (known.(method).not) {
				fail.("% comes from %".format(method, method.filenameSymbol));
			};
		};
	};
	fixtures.mkdir;
	goldens = (root +/+ "generator" +/+ "goldens.scd").load.(fixtures, fail);
	#build, write, writeCases, refused = goldens.atAll([\build, \write, \writeCases, \refused]);
	catalog = (root +/+ "generator" +/+ "sc3_plugins" +/+ "catalog.json").parseJSONFile;

	// The definition `m<index>` of an addable method. Each signal is a control
	// at the rate of the method, each count 2, each array two such controls,
	// and each symbol /abcd. A UGen with outputs gets mul 0.5 and add 0.25,
	// where the method applies them, and an Out of its result.
	definition = { |index, class, method|
		var selector = method["selector"];
		var prefix = switch (selector) { "ar" } { "a_" } { "ir" } { "i_" } { "" };
		var outputs = method["outputs"]["fixed"].isNil or: { method["outputs"]["fixed"].notEmpty };
		var madd = method["madd"] == "true";
		var names = List.new, values = List.new, call, body;
		method["args"].do { |spec, position|
			var name = spec["name"], kind = spec["kind"], control = prefix ++ "s" ++ position;
			var signal = 1000.25 + (17 * position);
			case
			{ madd and: { name == "mul" } } { values.add(if (outputs) { "0.5" } { "1" }) }
			{ madd and: { name == "add" } } { values.add(if (outputs) { "0.25" } { "0" }) }
			{ kind == "signal" } {
				names.add("% = %".format(control, signal));
				values.add(control);
			}
			{ kind == "array" } {
				names.add("% = #[%, %]".format(control, signal + 1000, signal + 1000.5));
				values.add(control);
			}
			{ kind == "count" } { values.add("2") }
			{ kind == "symbol" } { values.add("'/abcd'") }
			{ fail.("%.% % has the kind %".format(class, selector, name, kind)) };
		};
		// A variable list of arguments takes the elements of its array.
		if (class.asSymbol.asClass.class.findRespondingMethodFor(selector.asSymbol).varArgs) {
			values[values.size - 1] = "*" ++ values.last;
		};
		call = "%.%(%)".format(class, selector, values.join(", "));
		body = case
			{ outputs.not } { call }
			{ method["rate"] == "audio" } { "Out.ar(0, %)".format(call) }
			{ "Out.kr(0, %)".format(call) };
		build.("m" ++ index, ("{ " ++ if (names.isEmpty) { "" } { "|" ++ names.join(", ") ++ "| " }
			++ body ++ " }").interpret)
	};
	catalog["classes"].do { |class|
		class["methods"].do { |method|
			if (method["addable"] == "true") {
				methods.add(definition.(methods.size, class["name"], method));
			};
		};
	};
	write.("methods", methods);

	writeCases.("expansion", [
		// An array on a count makes one UGen for each count.
		[\expansion_panx, { Out.ar(0, PanX.ar([2, 3], SinOsc.ar, 0.25)) }],
		[\expansion_vbap, { Out.ar(0, VBAP.ar(4, SinOsc.ar, 0, [0, 45])) }],
		[\expansion_grainbufj, {
			Out.ar(0, GrainBufJ.ar([1, 2], Impulse.ar(10), 0.1, 0, 1, 0, 0, 2, 0.5))
		}],
		// An array argument goes into the inputs, and an array among its
		// elements expands the UGen.
		[\expansion_logger, { Logger.kr([SinOsc.kr(1), SinOsc.kr(2)], Impulse.kr(1), 0, 0) }],
		[\expansion_logger_nested, {
			Logger.kr([[SinOsc.kr(1), SinOsc.kr(2)], SinOsc.kr(3)], Impulse.kr(1), 0, 0)
		}],
		[\expansion_ntube, {
			Out.ar(0, NTube.ar(SinOsc.ar, [0.9, 0.8, 0.7], [0.1, 0.2], [0.01, 0.02, 0.03], 0.5))
		}],
		[\expansion_arraymax, { Out.kr(0, ArrayMax.kr([SinOsc.kr(1), SinOsc.kr(2), SinOsc.kr(3)])) }],
		// A variable list of arguments: each element is an argument of
		// multiNewList.
		[\expansion_ladspa, { Out.ar(0, LADSPA.ar(1, 1049, SinOsc.ar, 0.5)) }],
		[\expansion_ladspa_nested, { Out.ar(0, LADSPA.ar(1, 1049, [SinOsc.ar(1), SinOsc.ar(2)], 0.5)) }],
		// The count of the outputs is an input too.
		[\expansion_chromagram, {
			Out.kr(0, Chromagram.kr(FFT(LocalBuf(2048), SinOsc.ar), 2048, [12, 24]))
		}]
	]);
	writeCases.("pv", [
		// Two width-first readers of one chain: a copy for the first.
		[\pv_copies, {
			var chain = FFT(LocalBuf(512), SinOsc.ar);
			Out.ar(0, IFFT(PV_Invert(chain)) + IFFT(PV_EvenBin(chain)))
		}],
		[\pv_magmuladd, { Out.ar(0, IFFT(PV_MagMulAdd(FFT(LocalBuf(512), SinOsc.ar), 2, 0.5))) }]
	]);
	// The cases of the bodies. The sclang code of a value of a case, and the
	// controls that it reads, in order: a number, as parseJSON gives it, an
	// array, a control, a UGen, a symbol or a boolean.
	valueCode = { |value, controls|
		case
		{ value.isString } { value }
		{ value.isArray } { "[" ++ value.collect { |item| valueCode.(item, controls) }.join(", ") ++ "]" }
		{ value["control"].notNil } {
			controls.add(value["control"]);
			value["control"][0]
		}
		{ value["ugen"].notNil } { callCode.(value["ugen"], controls) }
		{ value["symbol"].notNil } { value["symbol"].asSymbol.asCompileString }
		{ value["bool"].notNil } { value["bool"] }
		{ fail.("bodies.json has the value %".format(value)) }
	};
	// sclang drops a keyword for the variable arguments of a method
	// (measured), so a call that gives them goes by position, and spreads
	// them. Such a case gives every argument before them.
	callCode = { |call, controls|
		var method = call[0].asSymbol.asClass.class.findRespondingMethodFor(call[1].asSymbol);
		var rest = method !? { if (method.varArgs) { method.argNames.last } };
		var spread = rest.notNil and: { call[2].any { |pair| pair[0].asSymbol == rest } };
		"%.%(%)".format(call[0], call[1], call[2].collect { |pair|
			case
			{ spread.not } { pair[0] ++ ": " ++ valueCode.(pair[1], controls) }
			{ pair[0].asSymbol == rest } { "*" ++ valueCode.(pair[1], controls) }
			{ valueCode.(pair[1], controls) }
		}.join(", "))
	};
	(fixtures +/+ "bodies.json").parseJSONFile["cases"].do { |case|
		var controls = List.new, call = callCode.(case["call"], controls), func;
		var body = case["out"] !? { |rate| "Out.%(0, %)".format(rate, call) } ? call;
		var code = "{ " ++ if (controls.isEmpty) { "" } {
			"|" ++ controls.collect { |control| control[0] ++ " = " ++ control[1] }.join(", ") ++ "| "
		} ++ body ++ " }";
		func = code.interpret;
		if (func.isNil) { fail.("the case % does not compile: %".format(case["name"], code)) };
		if (case["refused"] == "true") {
			refused.(case["name"], func);
		} {
			bodies.add(build.(case["name"], func));
		};
	};
	write.("bodies", bodies);

	"sc3_plugins/fixtures.scd: wrote % golden files, % definitions of methods, % cases of bodies, to %"
		.format(goldens[\written].value, methods.size, bodies.size, fixtures).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
