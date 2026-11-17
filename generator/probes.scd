// generator/probes.scd: the probes that measure the UGen classes in sclang
// 3.14.1, for generator/catalog.scd and generator/sc3_plugins/catalog.scd.
// The file answers a function of the tables of a catalogue, and that
// function answers an Event of the functions that the scripts call. The
// section "The catalogue" of the README tells the checks.
{ |tables|
	var fail = tables[\fail];
	var argumentKinds = tables[\argumentKinds];
	var bodies = tables[\bodies];
	var census = tables[\census];
	var audioRateInputs = tables[\audioRateInputs];
	var levelArguments = tables[\levelArguments];
	var passthroughs = tables[\passthroughs];
	var silentZeros = tables[\silentZeros];
	var audioTogether = tables[\audioTogether];
	// The classes whose methods are not addable, and why, by name.
	var refused = tables[\refused] ? ();

	// An argument whose name holds one of these words needs an entry in argumentKinds.
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

	// What optimizeGraph does with a UGen, by the class that owns the method.
	var optimizeRules = (
		UGen: \nothing,
		DetectSilence: \nothing,
		PureUGen: \dead_code,
		PureMultiOutUGen: \dead_code,
		UnaryOpUGen: \dead_code,
		BinaryOpUGen: \binary_op
	);

	// How fftSize finds the size of the FFT of a PV chain, by the class that
	// owns the method.
	var fftRules = (
		FFT: \buf_frames,
		PackFFT: \input,
		PV_ChainUGen: \chain
	);

	// How convertEnv takes an Env as the envelope of a UGen, by the class that
	// owns the class method: through asArray, or asArrayForInterpolation.
	var convertEnvRules = (
		EnvGen: \array,
		IEnvGen: \interpolation
	);

	// The counts of the methods that the probes describe.
	var stats = (addable: 0, methods: 0, pseudo: 0);
	// The units that cannot alias, which scanCantAlias finds.
	var cantAlias = List.new;
	var stripComments, quote, toJSON, entryFor, number, requireEntries, describeMethods;
	var describeUGenMethods;
	var methodOwner, ugenOwner, kindsOf, probe, signal, element, audio, control, numberValues;
	var controlValues, same, mapInputs, outputsOf, instanceOf, valuesAt, expansionOf;
	var arrayExpansionOf, wiring, testPassthrough, testSilence, testAudioTogether;
	var maddValues, maddOnce;
	var describeArgs, describeMethod, describePseudoClass, describeClass, scanCantAlias;

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

	// The entry of `table` for the method `selector` of `owner`, and for the
	// argument `name` when there is one.
	entryFor = { |table, owner, selector, name|
		table.detect { |entry|
			entry[0] == owner.name and: { entry[1] == selector }
				and: { name.isNil or: { entry[2] == name } }
		}
	};
	// A number of JSON, with nan and the infinities as text.
	number = { |value|
		case
			{ value.isNaN } { "nan" }
			{ value == inf } { "inf" }
			{ value == -inf } { "-inf" }
			{ value }
	};
	// Each of `methods` needs an entry in `table` or in `others`.
	requireEntries = { |methods, prefix, table, others, tableName, othersName|
		methods.do { |method|
			if (table.includes(method.name).not
				and: { others.any { |entry| entry[0] == method.name }.not }) {
				fail.("%% needs an entry in % or %".format(prefix, method.name, tableName, othersName));
			};
		};
	};
	// The methods `selectors` of `owner`, which `find` finds, with their
	// arguments, for a body of the crate.
	describeMethods = { |owner, selectors, find, noun|
		selectors.collect { |selector|
			var method = find.(selector);
			if (method.isNil) { fail.("% has no % %".format(owner.name, noun, selector)) };
			describeArgs.(owner, selector, method, owner)[1] ++ [\addable -> "pseudo"]
		}
	};
	// The instance methods `selectors` of UGen, with the classes whose methods
	// a Float and an Array answer, and the subclasses of UGen with a method of
	// their own.
	describeUGenMethods = { |selectors|
		describeMethods.(UGen, selectors, UGen.findMethod(_), "method").collect { |entry, index|
			var selector = selectors[index];
			var owner = { |class|
				class.findRespondingMethodFor(selector) !? { |found| found.ownerClass.name }
			};
			var overrides = UGen.allSubclasses.collect(owner).reject { |name| name == \UGen };
			entry ++ [
				\number -> owner.(Float),
				\array -> owner.(Array),
				\overrides -> overrides.asSet.asArray.sort
			]
		}
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
			var entry = entryFor.(argumentKinds, owner, selector, name);
			var named = { |table| entryFor.(table, owner, selector, name).notNil };
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
	// inside the build, and `after`, when there is one, takes the result.
	// Answers the error, or nil, the new UGens and the arguments. The call runs in a Routine: when sclang 3.14.1 catches an
	// error thrown while it evaluates the arguments of a send such as
	// `SinOsc.ar(nil.foo)`, it corrupts the frames of the caller (measured).
	probe = { |class, selector, values, after|
		var outcome = Routine {
			var result;
			try {
				result = {
					var def = SynthDef.newCopyArgs(name: \probe, metadata: (), children: Array(64));
					var arguments, before, made;
					def.initBuild;
					arguments = values.value;
					before = def.children.size;
					// A variable list of arguments takes the elements of its
					// array, as sclang passes them.
					made = class.performList(selector,
						if (class.class.findRespondingMethodFor(selector).varArgs) {
							arguments.drop(-1) ++ arguments.last.asArray
						} {
							arguments
						});
					after.value(made);
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

	// The values of numberValues, with `mul` and `add` in place of 1 and 0.
	maddValues = { |kinds, mul, add|
		var values = numberValues.(kinds, [], []);
		{
			var arguments = values.value;
			kinds.do { |kind, position|
				if (kind == \mul) { arguments.put(position, mul) };
				if (kind == \add) { arguments.put(position, add) };
			};
			arguments
		}
	};

	// Whether the method of a UGen with outputs applies madd once: with mul
	// 0.5 and add 0.25, it makes the UGens that one madd of its result makes,
	// at their rates.
	maddOnce = { |class, selector, kinds|
		var shape = { |result| result[1].collect { |ugen| [ugen.class, ugen.rate] } };
		var once = probe.(class, selector, maddValues.(kinds, 1.0, 0.0), _.madd(0.5, 0.25));
		var given = probe.(class, selector, maddValues.(kinds, 0.5, 0.25));
		once[0].isNil and: { given[0].isNil } and: { shape.(once) == shape.(given) }
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
					outputs.notEmpty and: { kinds.includes(\mul) or: { kinds.includes(\add) } }
					and: { maddOnce.(class, selector, kinds).not }
				} {
					"the method applies madd otherwise than once"
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

	// Fails unless the entry of audioTogether holds: with one of its arguments
	// at audio rate, a K2A at audio rate takes each other number, in order,
	// and the instance takes the K2A. With all of them at audio rate, the
	// method makes the instance alone.
	testAudioTogether = { |class, selector, kinds, positions|
		var holds = positions.every { |position|
			var created = probe.(class, selector,
				valuesAt.(kinds, position, { audio.(position) }))[1];
			var others = positions.reject { |other| other == position };
			created.size == (others.size + 1)
				and: { created.last.class === class }
				and: {
					others.every { |other, index|
						var converted = created[index];
						converted.class === K2A and: { converted.rate == \audio }
							and: { converted.inputs[0] == signal.(other) }
							and: { created.last.inputs.includes(converted) }
					}
				}
		} and: {
			var created = probe.(class, selector, {
				var arguments = numberValues.(kinds, [], []).value;
				positions.do { |position| arguments.put(position, audio.(position)) };
				arguments
			})[1];
			created.size == 1 and: { created[0].class === class }
		};
		if (holds.not) {
			fail.("the audio rate of %.% does not go together".format(class.name, selector));
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
						{ value.isKindOf(ArrayedCollection) and: { value.every(_.isNumber) } } {
							[\floats -> value]
						}
						{ value.isNumber } { [\float -> number.(value)] }
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
		var through = entryFor.(passthroughs, owner, selector);
		var silent = entryFor.(silentZeros, owner, selector);
		var together = entryFor.(audioTogether, owner, selector);
		var wired, input, zero, position, positions, refusal;
		stats[\methods] = stats[\methods] + 1;
		case
		{ refused[class.name].notNil } {
			described[1] ++ [\addable -> false, \reason -> refused[class.name]]
		}
		{ bodies.includes(class.name) } {
			refusal = probe.(class, selector, numberValues.(described[0], [], []))[0];
			if (refusal.isString and: { refusal.contains("not valid for this subclass") }) {
				described[1] ++ [\addable -> false, \reason -> refusal]
			} {
				stats[\pseudo] = stats[\pseudo] + 1;
				described[1] ++ [\addable -> "pseudo"]
			}
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
			if (together.notNil and: { wired.isString.not }) {
				positions = together[2].collect { |name| names.indexOf(name) };
				testAudioTogether.(class, selector, described[0], positions);
				wired = wired ++ [\audio_together -> positions];
			};
			if (wired.isString and: { wired.contains("creates no instance") }
				and: { census.includes(class.name).not }) {
				fail.("%.% creates no instance of its class, and the census does not name it"
					.format(class.name, selector));
			};
			described[1] ++ if (wired.isString) {
				[\addable -> false, \reason -> wired]
			} {
				stats[\addable] = stats[\addable] + 1;
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
			\methods -> Array.with(\ar, \kr, \ir, \tr, \new).select { |selector|
				var method = class.class.findRespondingMethodFor(selector);
				method.notNil and: { method.ownerClass != Meta_Object }
			}.collect { |selector|
				var method = class.class.findRespondingMethodFor(selector);
				stats[\pseudo] = stats[\pseudo] + 1;
				describeArgs.(class, selector, method, methodOwner.(method))[1]
					++ [\addable -> "pseudo"]
			}
		]
	};

	describeClass = { |class|
		var supers = class.superclasses;
		var check = class.findRespondingMethodFor(\checkInputs).ownerClass;
		var rule = checkRules[check.name];
		var optimize = optimizeRules[class.findRespondingMethodFor(\optimizeGraph).ownerClass.name];
		var fftSize = class.findRespondingMethodFor(\fftSize);
		var convertEnv = class.class.findRespondingMethodFor(\convertEnv);
		var convertEnvOwner = convertEnv !? { methodOwner.(convertEnv).name };
		var doc = SCDoc.documents["Classes/" ++ class.name];
		var kindOf = { |base| class == base or: { supers.includes(base) } };
		var signalRange = class.newCopyArgs.signalRange;
		if ([\bipolar, \unipolar].includes(signalRange).not) {
			fail.("the signalRange of % is %".format(class, signalRange));
		};
		if (rule.isNil) {
			fail.("checkInputs of % belongs to %, which checkRules does not know"
				.format(class, check.name));
		};
		if (optimize.isNil) {
			fail.("optimizeGraph of % belongs to %, which optimizeRules does not know"
				.format(class, class.findRespondingMethodFor(\optimizeGraph).ownerClass.name));
		};
		if (fftSize.notNil and: { fftRules[fftSize.ownerClass.name].isNil }) {
			fail.("fftSize of % belongs to %, which fftRules does not know"
				.format(class, fftSize.ownerClass.name));
		};
		if (convertEnvOwner.notNil and: { convertEnvRules[convertEnvOwner].isNil }) {
			fail.("convertEnv of % belongs to %, which convertEnvRules does not know"
				.format(class, convertEnvOwner));
		};
		[
			\name -> class.name,
			\superclasses -> supers.copyRange(0, supers.indexOf(UGen) ? -1).collect(_.name),
			\pure -> (kindOf.(PureUGen) or: { kindOf.(PureMultiOutUGen) }),
			\optimize -> optimize,
			\width_first -> kindOf.(WidthFirstUGen),
			\fft_size -> (fftSize !? { fftRules[fftSize.ownerClass.name] }),
			\convert_env -> (convertEnvOwner !? { convertEnvRules[convertEnvOwner] }),
			\cant_alias -> cantAlias.includes(class.name),
			\signal_range -> signalRange,
			\categories -> (doc !? { doc.categories } ? []),
			\check_inputs -> ([\rule -> rule[0]] ++ switch (rule[0])
				{ \n_inputs } { [\count -> rule[1]] }
				{ \input_count } { [\count -> rule[1]] }
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

	// The units of the plugin file at `path` that take buffers for their
	// outputs before they free those of their inputs, as names. A plugin sets
	// kUnitDef_CantAliasInputsToOutputs through the two macros of the
	// interface table, or through a macro of its own file. A FAUST plugin
	// passes the flag to fDefineUnit, under the name of its metadata. The flag
	// anywhere else fails the scan.
	scanCantAlias = { |path, text|
		var flag = "kUnitDef_CantAliasInputsToOutputs";
		var macros = ["DefineSimpleCantAliasUnit(", "DefineDtorCantAliasUnit("];
		var names = List.new, cut, start, stop, name;
		// `text` without the characters from `from` to `to`.
		cut = { |from, to| text.copyRange(0, from - 1) ++ text.copyRange(to + 1, text.size - 1) };
		if (text.find("CantAlias").notNil) {
			text = stripComments.(text);
			start = text.find("#define ");
			while { start.notNil } {
				// A definition goes on through each line that ends with a backslash.
				stop = text.find("\n", offset: start) ? text.size;
				while { stop < text.size and: { text.copyRange(start, stop - 1).stripWhiteSpace.last == $\\ } } {
					stop = text.find("\n", offset: stop + 1) ? text.size;
				};
				if (text.copyRange(start, stop - 1).find(flag).notNil) {
					macros = macros.add(text.copyRange(start + 8, text.find("(", offset: start))
						.stripWhiteSpace);
					text = cut.(start, stop - 1);
					start = text.find("#define ", offset: start);
				} {
					start = text.find("#define ", offset: stop);
				};
			};
			start = text.find("fDefineUnit");
			while { start.notNil } {
				stop = text.find(";", offset: start) ? (text.size - 1);
				if (text.copyRange(start, stop).find(flag).notNil) {
					name = text.find("declare(\"name\", \"");
					if (name.isNil) {
						fail.("% passes % to fDefineUnit, and its metadata has no name".format(path, flag));
					};
					names.add(text.copyRange(name + 17, text.find("\"", offset: name + 17) - 1).asSymbol);
					text = cut.(start, stop);
					start = text.find("fDefineUnit", offset: start);
				} {
					start = text.find("fDefineUnit", offset: stop);
				};
			};
			if (text.find(flag).notNil) {
				fail.("% sets % in another way".format(path, flag));
			};
			macros.do { |macro|
				start = text.find(macro);
				while { start.notNil } {
					stop = text.find(")", offset: start);
					names.add(text.copyRange(start + macro.size, stop - 1).stripWhiteSpace.asSymbol);
					start = text.find(macro, offset: stop);
				};
			};
		};
		cantAlias.addAll(names);
		names
	};

	(
		stats: stats, cantAlias: cantAlias, checkRules: checkRules, stripComments: stripComments,
		toJSON: toJSON, number: number, requireEntries: requireEntries,
		describeMethods: describeMethods, describeUGenMethods: describeUGenMethods,
		describeClass: describeClass,
		describePseudoClass: describePseudoClass, scanCantAlias: scanCantAlias
	)
}
