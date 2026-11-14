// generator/catalog.json: the catalogue of the core UGens and the tables of
// the operators, from sclang 3.14.1 and the sources of SuperCollider 3.14.1.
// Run it from the root of the repository, with absolute paths, since
// ~/.bin/sclang changes the directory:
//
//   sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/catalog.scd"
//
// SC_SOURCE names the clone of SuperCollider, ~/Projects/supercollider by
// default. The script reads its files at the tag Version-3.14.1, whatever is
// checked out. It exits with 1 if sclang is not 3.14.1, if a class or a
// method comes from outside the core class library, if sclang and the
// sources disagree on an operator, or if a table below misses an entry.
(
var root = thisProcess.nowExecutingPath.dirname.dirname;
var source = "SC_SOURCE".getenv ? "~/Projects/supercollider".standardizePath;
var library = Platform.classLibraryDir;
// sclang 3.14.1 runs the code after 1.exit (measured), so the throw stops it.
var fail = { |message|
	("catalog.scd: " ++ message).postln;
	1.exit;
	Error(message).throw;
};
var core = { |item| item.filenameSymbol.asString.beginsWith(library) };
var show = { |path|
	var text = "git -C % show Version-3.14.1:%".format(source.shellQuote, path).unixCmdGetStdOut;
	if (text.isEmpty) { fail.("git cannot show % in %".format(path, source)) };
	text
};

// The kind of each argument that is not a signal, by the class that owns
// the method, the selector and the argument. A count is an integer number
// of channels. An array is flattened into the inputs. A symbol is written as
// its size and its ASCII codes. A signal entry marks a name that only looks
// like an array.
var argumentKinds = [
	[\AudioControl, \ar, \values, \array],
	[\BelaScopeOut, \ar, \channelsArray, \array],
	[\BufRd, \ar, \numChannels, \count],
	[\BufRd, \kr, \numChannels, \count],
	[\BufWr, \ar, \inputArray, \array],
	[\BufWr, \kr, \inputArray, \array],
	[\Control, \ir, \values, \array],
	[\Control, \kr, \values, \array],
	[\DecodeB2, \ar, \numChans, \count],
	[\DecodeB2, \kr, \numChans, \count],
	[\Demand, \ar, \demandUGens, \array],
	[\Demand, \kr, \demandUGens, \array],
	[\DemandEnvGen, \ar, \levelScale, \signal],
	[\DemandEnvGen, \ar, \timeScale, \signal],
	[\DemandEnvGen, \kr, \levelScale, \signal],
	[\DemandEnvGen, \kr, \timeScale, \signal],
	[\DiskIn, \ar, \numChannels, \count],
	[\DiskOut, \ar, \channelsArray, \array],
	[\Dpoll, \new, \label, \symbol],
	[\Dswitch1, \new, \list, \array],
	[\Dwrand, \new, \list, \array],
	[\Dwrand, \new, \weights, \array],
	[\DynKlang, \ar, \specificationsArrayRef, \array],
	[\DynKlang, \kr, \specificationsArrayRef, \array],
	[\DynKlank, \ar, \specificationsArrayRef, \array],
	[\DynKlank, \kr, \specificationsArrayRef, \array],
	[\EnvGen, \ar, \envelope, \array],
	[\EnvGen, \ar, \levelScale, \signal],
	[\EnvGen, \ar, \timeScale, \signal],
	[\EnvGen, \kr, \envelope, \array],
	[\EnvGen, \kr, \levelScale, \signal],
	[\EnvGen, \kr, \timeScale, \signal],
	[\GrainBuf, \ar, \numChannels, \count],
	[\GrainFM, \ar, \numChannels, \count],
	[\GrainIn, \ar, \numChannels, \count],
	[\GrainSin, \ar, \numChannels, \count],
	[\IEnvGen, \ar, \envelope, \array],
	[\IEnvGen, \kr, \envelope, \array],
	[\In, \ar, \numChannels, \count],
	[\In, \kr, \numChannels, \count],
	[\InBus, \ar, \numChannels, \count],
	[\InBus, \kr, \numChannels, \count],
	[\InFeedback, \ar, \numChannels, \count],
	[\InTrig, \kr, \numChannels, \count],
	[\Klang, \ar, \specificationsArrayRef, \array],
	[\Klank, \ar, \specificationsArrayRef, \array],
	[\LagControl, \ar, \values, \array],
	[\LagControl, \kr, \values, \array],
	[\LagIn, \kr, \numChannels, \count],
	[\ListDUGen, \new, \list, \array],
	[\LocalBuf, \new, \numChannels, \signal],
	[\LocalBuf, \new, \numFrames, \signal],
	[\LocalIn, \ar, \default, \array],
	[\LocalIn, \ar, \numChannels, \count],
	[\LocalIn, \kr, \default, \array],
	[\LocalIn, \kr, \numChannels, \count],
	[\LocalOut, \ar, \channelsArray, \array],
	[\LocalOut, \kr, \channelsArray, \array],
	[\MFCC, \kr, \numcoeff, \count],
	[\Mix, \ar, \array, \array],
	[\Mix, \kr, \array, \array],
	[\Mix, \new, \array, \array],
	[\MultiTap, \ar, \levelsArray, \array],
	[\MultiTap, \ar, \timesArray, \array],
	[\NumChannels, \ar, \input, \array],
	[\NumChannels, \ar, \mixdown, \bool],
	[\NumChannels, \ar, \numChannels, \count],
	[\Out, \ar, \channelsArray, \array],
	[\Out, \kr, \channelsArray, \array],
	[\PackFFT, \new, \magsphases, \array],
	[\PanAz, \ar, \numChans, \count],
	[\PanAz, \kr, \numChans, \count],
	[\PlayBuf, \ar, \numChannels, \count],
	[\PlayBuf, \kr, \numChannels, \count],
	[\PingPong, \ar, \inputs, \array],
	[\Poll, \ar, \label, \symbol],
	[\Poll, \kr, \label, \symbol],
	[\Poll, \new, \label, \symbol],
	[\RecordBuf, \ar, \inputArray, \array],
	[\RecordBuf, \kr, \inputArray, \array],
	[\ScopeOut, \ar, \inputArray, \array],
	[\ScopeOut, \kr, \inputArray, \array],
	[\ScopeOut2, \ar, \inputArray, \array],
	[\ScopeOut2, \kr, \inputArray, \array],
	[\Select, \ar, \array, \array],
	[\Select, \kr, \array, \array],
	[\SelectX, \ar, \array, \array],
	[\SelectX, \kr, \array, \array],
	[\SelectXFocus, \ar, \array, \array],
	[\SelectXFocus, \ar, \wrap, \bool],
	[\SelectXFocus, \kr, \array, \array],
	[\SelectXFocus, \kr, \wrap, \bool],
	[\SelectXFocus, \new, \array, \array],
	[\SelectXFocus, \new, \wrap, \bool],
	[\SendPeakRMS, \ar, \cmdName, \symbol],
	[\SendPeakRMS, \kr, \cmdName, \symbol],
	[\SendReply, \ar, \cmdName, \symbol],
	[\SendReply, \ar, \values, \array],
	[\SendReply, \kr, \cmdName, \symbol],
	[\SendReply, \kr, \values, \array],
	[\SetBuf, \new, \values, \array],
	[\SharedIn, \kr, \numChannels, \count],
	[\SharedOut, \kr, \channelsArray, \array],
	[\Silent, \ar, \numChannels, \count],
	[\Splay, \ar, \inArray, \array],
	[\Splay, \kr, \inArray, \array],
	[\SplayAz, \ar, \inArray, \array],
	[\SplayAz, \ar, \numChans, \count],
	[\SplayAz, \kr, \inArray, \array],
	[\SplayAz, \kr, \numChans, \count],
	[\TChoose, \ar, \array, \array],
	[\TChoose, \kr, \array, \array],
	[\TGrains, \ar, \numChannels, \count],
	[\TWChoose, \ar, \array, \array],
	[\TWChoose, \ar, \weights, \array],
	[\TWChoose, \kr, \array, \array],
	[\TWChoose, \kr, \weights, \array],
	[\TWindex, \ar, \array, \array],
	[\TWindex, \kr, \array, \array],
	[\Tap, \ar, \numChannels, \count],
	[\VDiskIn, \ar, \numChannels, \count],
	[\Warp1, \ar, \numChannels, \count],
	[\XOut, \ar, \channelsArray, \array],
	[\XOut, \kr, \channelsArray, \array]
];
// The census of the classes whose rate methods do more than new1: the
// pseudo-UGens, whose rate methods build other UGens, and the classes of 0.4.
// The crate has a body for each rate method of a class of bodies.
var bodies = [\BeatTrack, \BHiPass4, \BLowPass4, \Changed, \CompanderD, \Dstutter,
	\DynKlang, \DynKlank, \HilbertFIR, \Klang, \Klank, \LinLin, \LinSelectX, \LocalBuf,
	\LocalIn, \Mix, \MultiTap, \NumChannels, \PMOsc, \PingPong, \Poll, \SelectX,
	\SelectXFocus, \Silent, \SoundIn, \Splay, \SplayAz, \Sum3, \Sum4, \TChoose,
	\TWChoose, \Tap, \UnpackFFT];
// The classes of the census that have no body, and why.
var leftOut = [
	[\AudioIn, "deprecated since 3.10"],
	[\Dunique, "builds its reader in asUGenInput, at each use as an input"],
	[\EnvGate, "needs NamedControl and Env: items 4 and 5 of Left out"],
	[\InBus, "reads a Bus of sclang, for NodeProxy"],
	[\NamedControl, "a control: item 4 of Left out"],
	[\SplayZ, "deprecated since 3.10"],
	[\XIn, "a helper of InBus"],
	[\XInFeedback, "a helper of InBus"]
];
// Classes that the census meets but that are no pseudo-UGens, and why.
var notPseudo = [
	[\LagControl, "a control, which waits for NamedControl"],
	[\LevelComp, "a helper of Splay and SplayAz"],
	[\MulAdd, "folds: madd of the builder"],
	[\Ndef, "a proxy of JITLib"],
	[\VarLag, "builds an Env with its default warp, which waits for Env"]
];
var census = bodies ++ leftOut.collect(_.first) ++ notPseudo.collect(_.first);

// The arguments that pass through asAudioRateInput before multiNew, by the
// class that owns the method. The probe gives them audio-rate outputs.
var audioRateInputs = [
	[\BufCombN, \ar, \in],
	[\BufDelayN, \ar, \in],
	[\CombN, \ar, \in],
	[\DelTapWr, \ar, \in],
	[\DelayN, \ar, \in]
];
// The arguments that multiply the result of multiNew, as `* level`. The
// probe gives them 1, which the fold of `*` drops.
var levelArguments = [
	[\LinXFade2, \ar, \level],
	[\LinXFade2, \kr, \level]
];
// The methods that return `in.madd(mul, add)` in place of an instance, by
// the class that owns the method: the input, and the argument whose 0 also
// returns it, or nil. The probe gives the input a control output, and
// testPassthrough tests each entry.
var passthroughs = [
	[\Lag, \ar, \in, \lagTime],
	[\Lag, \kr, \in, \lagTime],
	[\LagUD, \ar, \in, nil],
	[\LagUD, \kr, \in, nil],
	[\Slew, \ar, \in, nil],
	[\Slew, \kr, \in, nil]
];
// The arguments whose zeros become the channels of a Silent.ar before
// multiNewList, through replaceZeroesWithSilence, by the class that owns the
// method. testSilence tests each entry.
var silentZeros = [
	[\Out, \ar, \channelsArray],
	[\LocalOut, \ar, \channelsArray],
	[\XOut, \ar, \channelsArray]
];

// An argument whose name holds one of these words needs an entry above.
var arrayWords = ["array", "list", "values", "ugens", "envelope", "weights",
	"magsphases", "spec", "chans", "levels", "times", "numchannels", "numcoeff",
	"numframes"];

// The rule of checkInputs, by the class that owns the method. A custom rule
// makes the class not addable by name.
var checkRules = (
	UGen: [\valid],
	Filter: [\same_rate_as_first_input],
	Demand: [\same_rate_as_first_input],
	LinExp: [\same_rate_as_first_input],
	PitchShift: [\same_rate_as_first_input],
	Poll: [\same_rate_as_first_input],
	PulseCount: [\same_rate_as_first_input],
	SendTrig: [\same_rate_as_first_input],
	Stepper: [\same_rate_as_first_input],
	TDelay: [\same_rate_as_first_input],
	Timer: [\same_rate_as_first_input],
	ZeroCrossing: [\same_rate_as_first_input],
	AmpComp: [\same_rate_as_first_input_if_audio],
	CheckBadValues: [\same_rate_as_first_input_if_audio],
	Hasher: [\same_rate_as_first_input_if_audio],
	InRange: [\same_rate_as_first_input_if_audio],
	Sanitize: [\same_rate_as_first_input_if_audio],
	GVerb: [\n_inputs, 1],
	Pan2: [\n_inputs, 1],
	Pan4: [\n_inputs, 1],
	PanAz: [\n_inputs, 1],
	PanB: [\n_inputs, 1],
	PanB2: [\n_inputs, 1],
	Balance2: [\n_inputs, 2],
	BiPanB2: [\n_inputs, 2],
	FreeVerb2: [\n_inputs, 2],
	InRect: [\n_inputs, 2],
	LinXFade2: [\n_inputs, 2],
	Rotate2: [\n_inputs, 2],
	XFade2: [\n_inputs, 2],
	DecodeB2: [\n_inputs, 3],
	BufRd: [\audio_at, 1],
	Median: [\audio_at, 1],
	DiskOut: [\audio_from, 1],
	Select: [\audio_from, 1],
	AbstractOut: [\abstract_out],
	BufWr: [\buf_wr],
	Duty: [\duty],
	Peak: [\peak],
	T2K: [\t2k]
);

var stripComments, enumFrom, selectorsOf, operatorTable, quote, toJSON;
var ugenOwner, kindsOf, probe, signal, element, audio, control, numberValues, controlValues;
var same, mapInputs, outputsOf, instanceOf, valuesAt, expansionOf, arrayExpansionOf, wiring;
var testPassthrough, testSilence;
var describeArgs, describeMethod, describeClass, methodOwner, describePseudoClass;
var opcodes, parseNode, unaryEnum, binaryEnum, unaryNames, binaryNames, unary, binary;
var classes, pseudoClasses, addable = 0, methods = 0, pseudo = 0, helpDir, json;

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

	// C and C++ without their comments. A string literal stays as it is.
	stripComments = { |text|
		var kept = List.new, i = 0, char, next, inString = false;
		while { i < text.size } {
			char = text[i];
			next = if (i + 1 < text.size) { text[i + 1] };
			case
			{ inString } {
				kept.add(char);
				if (char == $\\) {
					kept.add(next);
					i = i + 1;
				} {
					if (char == $") { inString = false };
				};
			}
			{ char == $" } {
				inString = true;
				kept.add(char);
			}
			{ char == $/ and: { next == $* } } {
				i = i + 2;
				while { i + 1 < text.size and: { (text[i] == $* and: { text[i + 1] == $/ }).not } } {
					i = i + 1;
				};
				i = i + 1;
			}
			{ char == $/ and: { next == $/ } } {
				while { i < text.size and: { text[i] != $\n } } { i = i + 1 };
				i = i - 1;
			}
			{ kept.add(char) };
			i = i + 1;
		};
		String.newFrom(kept)
	};

	// The names of the enum that starts with `first`, up to its count.
	enumFrom = { |text, first|
		var start = text.find(first);
		var names = text.copyRange(start, text.find("}", offset: start) - 1)
			.split($,).collect(_.stripWhiteSpace).reject(_.isEmpty);
		names.drop(-1)
	};

	// The names that initSpecialSelectors gives the unary and the binary
	// operators, by their enum constants.
	selectorsOf = { |text|
		var unary = IdentityDictionary.new, binary = IdentityDictionary.new, table;
		text.split($\n).do { |line|
			case
			{ line.contains("sel = gSpecialUnarySelectors") } { table = unary }
			{ line.contains("sel = gSpecialBinarySelectors") } { table = binary }
			{ line.contains("sel = gSpecialSelectors") } { table = nil }
			{ table.notNil and: { line.contains("getsym(") } } {
				var constant = line.copyRange(line.find("[") + 1, line.find("]") - 1);
				var name = line.copyRange(line.find("getsym(\"") + 8, line.find("\")") - 1);
				table[constant.asSymbol] = name;
			};
		};
		[unary, binary]
	};

	// The name of each special index, or nil. sclang must give each name the
	// index of its enum constant.
	operatorTable = { |enum, names|
		enum.collect { |constant, index|
			var name = names[constant.asSymbol];
			if (name.notNil and: { name.asSymbol.specialIndex != index }) {
				fail.("sclang gives % the index %, and Opcodes.h gives %"
					.format(name, name.asSymbol.specialIndex, index));
			};
			name
		}
	};

	quote = { |string|
		"\"" ++ string.asString.replace("\\", "\\\\").replace("\"", "\\\"") ++ "\""
	};

	// JSON of nil, booleans, numbers, strings, symbols, lists, and objects:
	// arrays of associations, in order.
	toJSON = { |value, indent = ""|
		var inner = indent ++ "  ";
		case
		{ value.isNil } { "null" }
		{ value === true } { "true" }
		{ value === false } { "false" }
		{ value.isKindOf(Symbol) or: { value.isString } } { quote.(value) }
		{ value.isKindOf(Integer) } { value.asString }
		{ value.isKindOf(Float) } {
			if (value.isNaN or: { value.abs == inf }) {
				fail.("JSON has no number %".format(value));
			};
			value.asStringPrec(17)
		}
		{ value.isKindOf(SequenceableCollection) and: { value.isEmpty } } { "[]" }
		{ value.isKindOf(SequenceableCollection) and: { value.first.isKindOf(Association) } } {
			"{\n" ++ value.collect { |pair|
				inner ++ quote.(pair.key) ++ ": " ++ toJSON.(pair.value, inner)
			}.join(",\n") ++ "\n" ++ indent ++ "}"
		}
		{ value.isKindOf(SequenceableCollection) } {
			"[\n" ++ value.collect { |item| inner ++ toJSON.(item, inner) }.join(",\n")
				++ "\n" ++ indent ++ "]"
		}
		{ fail.("no JSON for %".format(value)) }
	};

	// The class that owns a class method.
	methodOwner = { |method| method.ownerClass.name.asString.drop(5).asSymbol.asClass };

	// The UGen class that owns a class method, or nil for Object and the like.
	ugenOwner = { |method|
		var owner = methodOwner.(method);
		if (owner.notNil and: { owner == UGen or: { (owner.superclasses ? []).includes(UGen) } }) {
			owner
		}
	};

	kindsOf = { |owner, selector, names|
		names.collect { |name|
			var entry = argumentKinds.detect { |entry|
				entry[0] == owner.name and: { entry[1] == selector } and: { entry[2] == name }
			};
			var named = { |table|
				table.any { |entry|
					entry[0] == owner.name and: { entry[1] == selector } and: { entry[2] == name }
				}
			};
			case
			{ entry.notNil } { entry[3] }
			{ named.(audioRateInputs) } { \audio }
			{ named.(levelArguments) } { \level }
			{ named.(passthroughs) } { \through }
			{ name == \mul } { \mul }
			{ name == \add } { \add }
			{ arrayWords.any { |word| name.asString.toLower.contains(word) } } {
				fail.("%.% % needs an entry in argumentKinds".format(owner.name, selector, name));
			}
			{ \signal }
		}
	};

	// Calls a method in a build of its own. `values` makes the arguments
	// inside the build. Answers the error, or nil, the new UGens and the
	// arguments. The call runs in a Routine: when sclang 3.14.1 catches an
	// error thrown while it evaluates the arguments of a send such as
	// `SinOsc.ar(nil.foo)`, it corrupts the frames of the caller (measured).
	probe = { |class, selector, values|
		var outcome = Routine {
			var result;
			try {
				result = {
					var def = SynthDef.newCopyArgs(name: \probe, metadata: (), children: Array(64));
					var arguments, before;
					def.initBuild;
					arguments = values.value;
					before = def.children.size;
					class.performList(selector, arguments);
					[nil, def.children.drop(before), arguments]
				}.value;
			} { |exception|
				result = [exception.errorString.replace("\n", " ").keep(160), [], nil];
			};
			result.yield;
		}.next;
		UGen.buildSynthDef = nil;
		outcome
	};

	// The sentinels: distinct numbers, far from 0, 1 and the counts.
	signal = { |position| 1000.25 + (17 * position) };
	element = { |position, index| 2000.25 + (17 * position) + (0.5 * index) };
	// An audio-rate output, made inside the build of a probe.
	audio = { |position, index = 0|
		AudioControl.ar([signal.(position) + (0.125 * index)]).asArray.first
	};
	// A control-rate output, made inside the build of a probe.
	control = { |position, index = 0|
		Control.kr([signal.(position) + (0.125 * index)]).asArray.first
	};
	numberValues = { |kinds, counts, sizes|
		{
			kinds.collect { |kind, position|
				switch (kind)
				{ \count } { counts[position] ? 2 }
				{ \array } { (0..(sizes[position] ? 2) - 1).collect { |index| element.(position, index) } }
				{ \symbol } { '/abcd' }
				{ \mul } { 1.0 }
				{ \add } { 0.0 }
				{ \level } { 1.0 }
				{ \audio } { audio.(position) }
				{ \through } { control.(position) }
				{ signal.(position) }
			}
		}
	};
	// The same, with distinct outputs of one Control.kr in place of numbers.
	controlValues = { |kinds|
		{
			var numbers = List.new;
			var places, proxies;
			places = kinds.collect { |kind, position|
				switch (kind)
				{ \signal } { numbers.add(signal.(position)); numbers.size - 1 }
				{ \array } {
					numbers.addAll([element.(position, 0), element.(position, 1)]);
					numbers.size - 2
				}
			};
			proxies = if (numbers.isEmpty) { [] } { Control.kr(numbers.asArray).asArray };
			kinds.collect { |kind, position|
				switch (kind)
				{ \signal } { proxies[places[position]] }
				{ \array } { proxies.copyRange(places[position], places[position] + 1) }
				{ \count } { 2 }
				{ \symbol } { '/abcd' }
				{ \mul } { 1.0 }
				{ \add } { 0.0 }
				{ \level } { 1.0 }
				{ \audio } { audio.(position) }
				{ \through } { control.(position) }
			}
		}
	};

	same = { |a, b| if (a.isNumber and: { b.isNumber }) { a == b } { a === b } };

	// The slot of each input: the argument it comes from. Answers the slots,
	// or a String that says why an input maps to none.
	mapInputs = { |inputs, arguments, kinds|
		var slots = List.new, i = 0, failure;
		while { failure.isNil and: { i < inputs.size } } {
			var found;
			arguments.do { |value, position|
				if (found.isNil) {
					switch (kinds[position])
					{ \mul } {}
					{ \add } {}
					{ \level } {}
					{ \symbol } {
						var codes = value.asString.ascii;
						if (inputs[i] == codes.size
							and: { inputs.copyRange(i + 1, i + codes.size) == codes }) {
							found = [[\string, position], codes.size + 1];
						};
					}
					{ \array } {
						if (value.size > 0 and: {
							value.every { |item, k| inputs[i + k].notNil and: { same.(inputs[i + k], item) } }
						}) {
							found = [[\array, position], value.size];
						};
					}
					{
						if (same.(inputs[i], value)) { found = [[\arg, position], 1] };
					};
				};
			};
			if (found.isNil) {
				failure = "input % maps to no argument".format(i);
			} {
				slots.add(found[0]);
				i = i + found[1];
			};
		};
		failure ? slots.asArray
	};

	outputsOf = { |ugen|
		case
		{ ugen.isKindOf(MultiOutUGen) } { ugen.channels.collect(_.rate) }
		{ ugen.numOutputs == 0 } { [] }
		{ [ugen.rate] }
	};

	// The one instance of `class` among the new UGens, or why there is none.
	instanceOf = { |class, result|
		var error = result[0], created = result[1];
		var instances = created.select { |ugen| ugen.class === class };
		case
		{ error.notNil } { error }
		{ instances.isEmpty } { "the method creates no instance of the class" }
		{ instances.size > 1 } { "the method creates % instances of the class".format(instances.size) }
		{ created.size > 1 } {
			"the method also creates %".format(created.reject { |ugen| ugen.class === class }
				.collect { |ugen| ugen.class.name })
		}
		{ instances[0] }
	};

	// The values of numberValues, with `value` at `position`. A function makes
	// the value inside the build.
	valuesAt = { |kinds, position, value|
		var values = numberValues.(kinds, [], []);
		{
			var arguments = values.value;
			arguments.put(position, if (value.isFunction) { value.value } { value });
			arguments
		}
	};

	// How the method expands, as associations, or why it does not expand as
	// multiNewList does. The arguments that expand are those of the wire and
	// the count. On each of them, an array of two elements must make two
	// instances and nothing else, instance i with element i. Then an array
	// that holds an array must make three instances, in order: else the
	// expansion stops at one level, as the flop of SendReply does.
	expansionOf = { |class, selector, kinds, slots, dependence|
		var positions = slots.select { |slot| slot[0] == \arg }.collect { |slot| slot[1] };
		var failure, nested = true;
		var matches = { |result, position|
			var instances = result[1].select { |ugen| ugen.class === class };
			var elements = result[2] !? { result[2][position].flat } ? [];
			result[0].isNil
			and: { elements.notEmpty }
			and: { result[1].size == elements.size }
			and: { instances.size == elements.size }
			and: {
				instances.every { |instance, i|
					var arguments = result[2].copy.put(position, elements[i]);
					mapInputs.(instance.inputs ? [], arguments, kinds) == slots
					and: { kinds[position] != \count or: { outputsOf.(instance).size == elements[i] } }
				}
			}
		};
		if (dependence.notEmpty and: { dependence[0][0] == \count }) {
			positions = positions.add(dependence[0][1]);
		};
		positions.asSet.asArray.sort.do { |position|
			var flat, deep;
			if (failure.isNil) {
				case
				{ kinds[position] == \count } {
					flat = [2, 3];
					deep = [[2, 3], 4];
				}
				{ kinds[position] == \audio } {
					flat = { [audio.(position, 0), audio.(position, 1)] };
					deep = { [[audio.(position, 0), audio.(position, 1)], audio.(position, 2)] };
				}
				{ kinds[position] == \through } {
					flat = { [control.(position, 0), control.(position, 1)] };
					deep = { [[control.(position, 0), control.(position, 1)], control.(position, 2)] };
				}
				{
					flat = [signal.(position), signal.(position) + 0.125];
					deep = [flat, signal.(position) + 0.375];
				};
				case
				{ matches.(probe.(class, selector, valuesAt.(kinds, position, flat)), position).not } {
					failure = "an array on argument % does not make one instance per element"
						.format(position);
				}
				{ matches.(probe.(class, selector, valuesAt.(kinds, position, deep)), position).not } {
					nested = false;
				};
			};
		};
		failure ?? { [\nested_expansion -> nested] }
	};

	// How the array argument of a method takes an array among its elements,
	// as associations: none without an array argument, or when the array
	// stays among the inputs of the one instance. Each call must make the
	// instances and nothing else, each with the array of its own.
	arrayExpansionOf = { |class, selector, kinds, slots|
		var position = slots.detect { |slot| slot[0] == \array } !? { |slot| slot[1] };
		var e = { |index| element.(position, index) };
		var instancesFor = { |value|
			var result = probe.(class, selector, valuesAt.(kinds, position, value));
			var instances = result[1].select { |ugen| ugen.class === class };
			if (result[0].isNil and: { instances.size == result[1].size }) {
				[instances, result[2]]
			}
		};
		var holds = { |found, arrays|
			found.notNil and: { found[0].size == arrays.size } and: {
				found[0].every { |instance, i|
					var arguments = found[1].copy.put(position, arrays[i]);
					mapInputs.(instance.inputs ? [], arguments, kinds) == slots
				}
			}
		};
		var keeps = { |found|
			found.notNil and: { found[0].size == 1 }
			and: { found[0][0].inputs.any { |input| input.class == Array } }
		};
		var first, second;
		if (position.isNil) { [] } {
			first = instancesFor.([[e.(0), e.(1)], e.(2)]);
			second = instancesFor.([e.(2), [e.(0), e.(1)]]);
			case
			{
				holds.(first, [[e.(0), e.(2)], [e.(1), e.(2)]])
				and: { holds.(second, [[e.(2), e.(0)], [e.(2), e.(1)]]) }
				and: {
					holds.(instancesFor.([[[e.(0), e.(1)], e.(2)], e.(3)]),
						[[e.(0), e.(3)], [e.(1), e.(3)], [e.(2), e.(3)]])
				}
			} { [\array_expansion -> \elements] }
			{ holds.(first, [[e.(0), e.(1)], [e.(2)]]) and: { holds.(second, [[e.(2)], [e.(0), e.(1)]]) } } {
				[\array_expansion -> \flop]
			}
			{ holds.(first, [[e.(0), e.(1)], [e.(2)]]) and: { keeps.(second) } } {
				[\array_expansion -> \refs]
			}
			{ keeps.(first) and: { keeps.(second) } } { [] }
			{ fail.("the array of %.% takes an array in no known way".format(class.name, selector)) }
		}
	};

	// The wiring of a method, as associations, or the reason why `ugen`
	// cannot add it.
	wiring = { |class, selector, kinds|
		var numbers = probe.(class, selector, numberValues.(kinds, [], []));
		var controls = probe.(class, selector, controlValues.(kinds));
		var first = instanceOf.(class, numbers), second = instanceOf.(class, controls);
		var slots, outputs, dependence = [], failure, expansion;
		case
		{ first.isString } { first }
		{ second.isString } { "with control-rate inputs, " ++ second }
		{
			slots = mapInputs.(first.inputs ? [], numbers[2], kinds);
			case
			{ slots.isString } { slots }
			{ slots != mapInputs.(second.inputs ? [], controls[2], kinds) } {
				"the inputs map otherwise with control-rate inputs"
			}
			{ first.rate != second.rate } { "the rate depends on the inputs" }
			{ first.specialIndex != second.specialIndex } { "the special index depends on the inputs" }
			{
				outputs = outputsOf.(first);
				kinds.do { |kind, position|
					var counts = [], sizes = [], result, instance, size;
					if (failure.isNil and: { kind == \count or: { kind == \array } }) {
						if (kind == \count) { counts = counts.extend(position + 1, nil).put(position, 3) };
						if (kind == \array) { sizes = sizes.extend(position + 1, nil).put(position, 3) };
						result = probe.(class, selector, numberValues.(kinds, counts, sizes));
						instance = instanceOf.(class, result);
						case
						{ instance.isString } { failure = "with % at 3, %".format(position, instance) }
						{ mapInputs.(instance.inputs ? [], result[2], kinds) != slots } {
							failure = "the inputs depend on argument %".format(position)
						}
						{
							size = outputsOf.(instance).size;
							if (size != outputs.size) {
								if (size == 3 and: { outputs.size == 2 }) {
									dependence = dependence.add([kind, position]);
								} {
									failure = "the outputs depend on argument % otherwise".format(position);
								};
							};
						};
					};
				};
				case
				{ failure.notNil } { failure }
				{ dependence.size > 1 } { "the outputs depend on several arguments" }
				{ outputs.asSet.size > 1 and: { dependence.notEmpty } } { "the outputs have several rates" }
				{ (expansion = expansionOf.(class, selector, kinds, slots, dependence)).isString } {
					expansion
				}
				{
					[
						\rate -> first.rate,
						\inputs -> slots.collect { |slot| [slot[0] -> slot[1]] },
						\outputs -> case
							{ dependence.isEmpty } { [\fixed -> outputs] }
							{ dependence[0][0] == \count } {
								[\arg -> dependence[0][1], \rate -> outputs[0]]
							}
							{ [\array_size -> dependence[0][1], \rate -> outputs[0]] },
						\special_index -> first.specialIndex,
						\madd -> (kinds.includes(\mul) or: { kinds.includes(\add) })
					] ++ expansion
					++ arrayExpansionOf.(class, selector, kinds, slots)
					++ if (kinds.includes(\audio)) { [\audio_input -> kinds.indexOf(\audio)] } { [] }
					++ if (kinds.includes(\level)) { [\level -> true] } { [] }
				}
			}
		}
	};

	// Fails unless the entry of passthroughs holds: a number input makes no
	// UGen. With a zero argument, a control input and 0 there make no UGen.
	// Without one, a control input and 0 in each other signal make one
	// instance.
	testPassthrough = { |class, selector, kinds, input, zero|
		var none = { |values|
			var result = probe.(class, selector, values);
			result[0].isNil and: { result[1].isEmpty }
		};
		var holds = none.(valuesAt.(kinds, input, signal.(input))) and: {
			if (zero.notNil) {
				none.(valuesAt.(kinds, zero, 0))
			} {
				instanceOf.(class, probe.(class, selector, {
					var arguments = numberValues.(kinds, [], []).value;
					kinds.do { |kind, position|
						if (kind == \signal) { arguments.put(position, 0) };
					};
					arguments
				})).isString.not
			}
		};
		if (holds.not) {
			fail.("the passthrough of %.% does not hold".format(class.name, selector));
		};
	};

	// Fails unless the entry of silentZeros holds: a 0 in the array makes one
	// DC at audio rate before the instance, which takes its output.
	testSilence = { |class, selector, kinds, position|
		var created = probe.(class, selector,
			valuesAt.(kinds, position, [element.(position, 0), 0]))[1];
		var holds = created.size == 2
			and: { created[0].class === DC and: { created[0].rate == \audio } }
			and: { created[1].class === class }
			and: {
				created[1].inputs.any { |input|
					input.isKindOf(OutputProxy) and: { input.source === created[0] }
				}
			};
		if (holds.not) {
			fail.("the silence of %.% does not hold".format(class.name, selector));
		};
	};

	// The kinds of the arguments of a method, with the kinds of `owner`, and
	// the method as associations: its selector and its arguments.
	describeArgs = { |class, selector, method, owner|
		var names = (method.argNames ? []).drop(1).asArray;
		var defaults = (method.prototypeFrame ? []).drop(1);
		var kinds = kindsOf.(owner, selector, names);
		[kinds, [
			\selector -> selector,
			\args -> names.collect { |name, position|
				var value = defaults[position];
				[
					\name -> name,
					\kind -> if ([\mul, \add, \level, \audio, \through].includes(kinds[position])) {
						\signal
					} {
						kinds[position]
					},
					\default -> case
						{ value.isNil } { nil }
						{ value === true or: { value === false } } { [\bool -> value] }
						{ value.isKindOf(Symbol) } { [\symbol -> value] }
						{ value.isString } { [\string -> value] }
						{ value.isNumber } {
							[\float -> case
								{ value.isNaN } { "nan" }
								{ value == inf } { "inf" }
								{ value == -inf } { "-inf" }
								{ value }]
						}
						{ fail.("%.% % has the default %".format(class, selector, name, value)) }
				]
			}
		]]
	};

	// A method of a UGen class: its body, or its wiring, or why it is not
	// addable.
	describeMethod = { |class, selector, method, customCheck|
		var owner = ugenOwner.(method);
		var described = describeArgs.(class, selector, method, owner);
		var names = (method.argNames ? []).drop(1).asArray;
		var through = passthroughs.detect { |entry| entry[0] == owner.name and: { entry[1] == selector } };
		var silent = silentZeros.detect { |entry| entry[0] == owner.name and: { entry[1] == selector } };
		var wired, input, zero, position;
		methods = methods + 1;
		if (bodies.includes(class.name)) {
			pseudo = pseudo + 1;
			described[1] ++ [\addable -> "pseudo"]
		} {
			wired = wiring.(class, selector, described[0]);
			if (customCheck.notNil and: { wired.isString.not }) {
				wired = "the checkInputs of % is a rule of its own".format(customCheck);
			};
			if (through.notNil and: { wired.isString.not }) {
				input = names.indexOf(through[2]);
				zero = through[3] !? { |name| names.indexOf(name) };
				testPassthrough.(class, selector, described[0], input, zero);
				wired = wired ++ [\passthrough -> [\input -> input, \zero -> zero]];
			};
			if (silent.notNil and: { wired.isString.not }) {
				position = names.indexOf(silent[2]);
				testSilence.(class, selector, described[0], position);
				wired = wired ++ [\silence -> position];
			};
			if (wired.isString and: { wired.contains("creates no instance") }
				and: { census.includes(class.name).not }) {
				fail.("%.% creates no instance of its class, and the census does not name it"
					.format(class.name, selector));
			};
			described[1] ++ if (wired.isString) {
				[\addable -> false, \reason -> wired]
			} {
				addable = addable + 1;
				[\addable -> true] ++ wired
			}
		}
	};

	// A class of bodies outside UGen, with its rate methods but the new of
	// Object.
	describePseudoClass = { |class|
		var doc = SCDoc.documents["Classes/" ++ class.name];
		[
			\name -> class.name,
			\categories -> (doc !? { doc.categories } ? []),
			\methods -> Array.with(\ar, \kr, \ir, \new).select { |selector|
				var method = class.class.findRespondingMethodFor(selector);
				method.notNil and: { method.ownerClass != Meta_Object }
			}.collect { |selector|
				var method = class.class.findRespondingMethodFor(selector);
				pseudo = pseudo + 1;
				describeArgs.(class, selector, method, methodOwner.(method))[1]
					++ [\addable -> "pseudo"]
			}
		]
	};

	describeClass = { |class|
		var supers = class.superclasses;
		var check = class.findRespondingMethodFor(\checkInputs).ownerClass;
		var rule = checkRules[check.name];
		var doc = SCDoc.documents["Classes/" ++ class.name];
		var kindOf = { |base| class == base or: { supers.includes(base) } };
		if (rule.isNil) {
			fail.("checkInputs of % belongs to %, which checkRules does not know"
				.format(class, check.name));
		};
		[
			\name -> class.name,
			\superclasses -> supers.copyRange(0, supers.indexOf(UGen) ? -1).collect(_.name),
			\pure -> (kindOf.(PureUGen) or: { kindOf.(PureMultiOutUGen) }),
			\width_first -> kindOf.(WidthFirstUGen),
			\categories -> (doc !? { doc.categories } ? []),
			\check_inputs -> ([\rule -> rule[0]] ++ switch (rule[0])
				{ \n_inputs } { [\count -> rule[1]] }
				{ \audio_at } { [\index -> rule[1]] }
				{ \audio_from } { [\index -> rule[1]] }
				{ \abstract_out } { [\fixed -> (try { class.numFixedArgs } ? 0)] }
				{ \custom } { [\owner -> check.name] }
				{ [] }),
			\methods -> Array.with(\ar, \kr, \ir, \new).select { |selector|
				var method = class.class.findRespondingMethodFor(selector);
				method.notNil and: { ugenOwner.(method).notNil }
			}.collect { |selector|
				describeMethod.(class, selector, class.class.findRespondingMethodFor(selector),
					if (rule[0] == \custom) { check.name })
			}
		]
	};

	opcodes = stripComments.(show.("lang/LangSource/Opcodes.h"));
	parseNode = stripComments.(show.("lang/LangSource/PyrParseNode.cpp"));
	unaryEnum = enumFrom.(opcodes, "opNeg,");
	binaryEnum = enumFrom.(opcodes, "opAdd,");
	#unaryNames, binaryNames = selectorsOf.(parseNode);
	unary = operatorTable.(unaryEnum, unaryNames);
	binary = operatorTable.(binaryEnum, binaryNames);

	// SCDoc writes docmap.js into its target folder, by default the help
	// cache of the IDE.
	helpDir = Platform.defaultTempDir +/+ "scsynth-synthdef-scdoc";
	helpDir.mkdir;
	SCDoc.helpTargetDir_(helpDir);
	SCDoc.indexAllDocuments;

	// The census: each class outside UGen that answers ar or kr, and each
	// class that it names, must exist and be named.
	census.do { |name|
		if (name.asClass.isNil) { fail.("the census names %, which is no class".format(name)) };
	};
	(passthroughs ++ silentZeros).do { |entry|
		if (entry[0].asClass.isNil) { fail.("a table names %, which is no class".format(entry[0])) };
	};
	Class.allClasses.do { |class|
		if (class.isMetaClass.not
			and: { class != UGen and: { (class.superclasses ? []).includes(UGen).not } }
			and: { [\ar, \kr].any { |selector| class.class.findRespondingMethodFor(selector).notNil } }
			and: { census.includes(class.name).not }) {
			fail.("% answers ar or kr, and the census does not name it".format(class.name));
		};
	};

	classes = UGen.allSubclasses.asArray
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describeClass);
	pseudoClasses = bodies.collect(_.asClass)
		.reject { |class| (class.superclasses ? []).includes(UGen) }
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describePseudoClass);

	json = toJSON.([
		\sclang -> Main.version,
		\operators -> [\unary -> unary, \binary -> binary],
		\classes -> classes,
		\pseudo_classes -> pseudoClasses
	]) ++ "\n";
	File.use(root +/+ "generator" +/+ "catalog.json", "w", { |file| file.write(json) });
	("catalog.scd: % unary and % binary operators, % classes, % of % methods addable, "
		++ "% pseudo-UGen methods, % classes outside UGen")
		.format(unary.size, binary.size, classes.size, addable, methods, pseudo,
			pseudoClasses.size).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
