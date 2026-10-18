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
var fail = { |message|
	("catalog.scd: " ++ message).postln;
	1.exit;
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
	[\LocalBuf, \new, \numChannels, \count],
	[\LocalBuf, \new, \numFrames, \count],
	[\LocalIn, \ar, \numChannels, \count],
	[\LocalIn, \kr, \numChannels, \count],
	[\LocalOut, \ar, \channelsArray, \array],
	[\LocalOut, \kr, \channelsArray, \array],
	[\MFCC, \kr, \numcoeff, \count],
	[\Out, \ar, \channelsArray, \array],
	[\Out, \kr, \channelsArray, \array],
	[\PackFFT, \new, \magsphases, \array],
	[\PanAz, \ar, \numChans, \count],
	[\PanAz, \kr, \numChans, \count],
	[\PlayBuf, \ar, \numChannels, \count],
	[\PlayBuf, \kr, \numChannels, \count],
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
	[\SendPeakRMS, \ar, \cmdName, \symbol],
	[\SendPeakRMS, \kr, \cmdName, \symbol],
	[\SendReply, \ar, \cmdName, \symbol],
	[\SendReply, \ar, \values, \array],
	[\SendReply, \kr, \cmdName, \symbol],
	[\SendReply, \kr, \values, \array],
	[\SetBuf, \new, \values, \array],
	[\SharedIn, \kr, \numChannels, \count],
	[\SharedOut, \kr, \channelsArray, \array],
	[\Splay, \ar, \inArray, \array],
	[\Splay, \kr, \inArray, \array],
	[\SplayAz, \ar, \inArray, \array],
	[\SplayAz, \ar, \numChans, \count],
	[\SplayAz, \kr, \inArray, \array],
	[\SplayAz, \kr, \numChans, \count],
	[\TGrains, \ar, \numChannels, \count],
	[\TWindex, \ar, \array, \array],
	[\TWindex, \kr, \array, \array],
	[\Tap, \ar, \numChannels, \count],
	[\VDiskIn, \ar, \numChannels, \count],
	[\Warp1, \ar, \numChannels, \count],
	[\XOut, \ar, \channelsArray, \array],
	[\XOut, \kr, \channelsArray, \array]
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
	BufWr: [\custom],
	Duty: [\custom],
	Peak: [\custom],
	T2K: [\custom]
);

var stripComments, enumFrom, selectorsOf, operatorTable, quote, toJSON;
var ugenOwner, kindsOf, probe, signal, element, numberValues, controlValues;
var same, mapInputs, outputsOf, instanceOf, wiring, describeMethod, describeClass;
var opcodes, parseNode, unaryEnum, binaryEnum, unaryNames, binaryNames, unary, binary;
var classes, addable = 0, methods = 0, helpDir, json;

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

	// The UGen class that owns a class method, or nil for Object and the like.
	ugenOwner = { |method|
		var owner = method.ownerClass.name.asString.drop(5).asSymbol.asClass;
		if (owner.notNil and: { owner == UGen or: { (owner.superclasses ? []).includes(UGen) } }) {
			owner
		}
	};

	kindsOf = { |owner, selector, names|
		names.collect { |name|
			var entry = argumentKinds.detect { |entry|
				entry[0] == owner.name and: { entry[1] == selector } and: { entry[2] == name }
			};
			case
			{ entry.notNil } { entry[3] }
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
	numberValues = { |kinds, counts, sizes|
		{
			kinds.collect { |kind, position|
				switch (kind)
				{ \count } { counts[position] ? 2 }
				{ \array } { (0..(sizes[position] ? 2) - 1).collect { |index| element.(position, index) } }
				{ \symbol } { '/abcd' }
				{ \mul } { 1.0 }
				{ \add } { 0.0 }
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

	// The wiring of a method, as associations, or the reason why `ugen`
	// cannot add it.
	wiring = { |class, selector, kinds|
		var numbers = probe.(class, selector, numberValues.(kinds, [], []));
		var controls = probe.(class, selector, controlValues.(kinds));
		var first = instanceOf.(class, numbers), second = instanceOf.(class, controls);
		var slots, outputs, dependence = [], failure;
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
					]
				}
			}
		}
	};

	describeMethod = { |class, selector, method|
		var owner = ugenOwner.(method);
		var names = (method.argNames ? []).drop(1).asArray;
		var defaults = (method.prototypeFrame ? []).drop(1);
		var kinds = kindsOf.(owner, selector, names);
		var wired = wiring.(class, selector, kinds);
		methods = methods + 1;
		[
			\selector -> selector,
			\args -> names.collect { |name, position|
				var value = defaults[position];
				[
					\name -> name,
					\kind -> if ([\mul, \add].includes(kinds[position])) { \signal } { kinds[position] },
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
		] ++ if (wired.isString) {
			[\addable -> false, \reason -> wired]
		} {
			addable = addable + 1;
			[\addable -> true] ++ wired
		}
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
				describeMethod.(class, selector, class.class.findRespondingMethodFor(selector))
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

	classes = UGen.allSubclasses.asArray
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describeClass);

	json = toJSON.([
		\sclang -> Main.version,
		\operators -> [\unary -> unary, \binary -> binary],
		\classes -> classes
	]) ++ "\n";
	File.use(root +/+ "generator" +/+ "catalog.json", "w", { |file| file.write(json) });
	"catalog.scd: % unary and % binary operators, % classes, % of % methods addable"
		.format(unary.size, binary.size, classes.size, addable, methods).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
