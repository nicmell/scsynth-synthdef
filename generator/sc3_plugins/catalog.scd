// generator/sc3_plugins/catalog.json: the catalogue of the UGens of
// sc3-plugins 3.14.0, from sclang 3.14.1 and the sources of sc3-plugins.
// Run it from the root of the repository, with absolute paths, since
// ~/.bin/sclang changes the directory:
//
//   sclang -l "$PWD/generator/sc3_plugins/sclang_conf.yaml" \
//     "$PWD/generator/sc3_plugins/catalog.scd"
//
// SC3_PLUGINS_SOURCE names the clone of sc3-plugins, ~/Projects/sc3-plugins
// by default. The script reads its files at the tag Version-3.14.0, whatever
// is checked out. The conf names the installed folder SC3plugins, whose class
// files must equal those of the tag. The script exits with 1 when a check
// fails, and the section "sc3-plugins" of the README tells the checks. The
// probes come from generator/probes.scd.
(
var root = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var source = "SC3_PLUGINS_SOURCE".getenv ? "~/Projects/sc3-plugins".standardizePath;
var tag = "Version-3.14.0";
var library = Platform.classLibraryDir;
var folder = LanguageConfig.includePaths.detect { |path| path.endsWith("SC3plugins") };
var scratch = Platform.defaultTempDir +/+ "scsynth-synthdef-sc3-plugins";
// sclang 3.14.1 runs the code after 1.exit (measured), so the throw stops it.
var fail = { |message|
	("sc3_plugins/catalog.scd: " ++ message).postln;
	1.exit;
	Error(message).throw;
};
var core = { |item| item.filenameSymbol.asString.beginsWith(library) };
var plugin = { |item| folder.notNil and: { item.filenameSymbol.asString.beginsWith(folder) } };
// A file of the sources at the tag. It goes through a file of its own, as
// unixCmdGetStdOut splits a line of more than 1024 characters.
var show = { |path|
	var copy = scratch +/+ "show.txt";
	if ("git -C % show % > %".format(source.shellQuote, (tag ++ ":" ++ path).shellQuote,
		copy.shellQuote).systemCmd != 0) {
		fail.("git cannot show % in %".format(path, source));
	};
	File.readAllString(copy)
};

// The kind of each argument that is not a signal, by the class that owns
// the method, the selector and the argument, as in generator/catalog.scd.
var argumentKinds = [
	[\ArrayMax, \ar, \array, \array],
	[\ArrayMax, \kr, \array, \array],
	[\Chromagram, \kr, \n, \count],
	[\Dneuromodule, \ar, \numChannels, \count],
	[\Dneuromodule, \ar, \weights, \array],
	[\Dneuromodule, \kr, \numChannels, \count],
	[\Dneuromodule, \kr, \weights, \array],
	[\Dneuromodule, \new, \numChannels, \count],
	[\Dneuromodule, \new, \weights, \array],
	[\FFTSubbandFlatness, \kr, \cutfreqs, \array],
	[\FFTSubbandFlux, \kr, \cutfreqs, \array],
	[\FFTSubbandPower, \kr, \cutfreqs, \array],
	[\FeatureSave, \kr, \features, \array],
	[\GaussClass, \kr, \in, \array],
	[\GrainBufJ, \ar, \numChannels, \count],
	[\GrainFMJ, \ar, \numChannels, \count],
	[\GrainInJ, \ar, \numChannels, \count],
	[\GrainSinJ, \ar, \numChannels, \count],
	[\Greyhole, \ar, \in, \array],
	[\JPverb, \ar, \in, \array],
	[\KMeansRT, \kr, \inputdata, \array],
	[\LADSPA, \ar, \args, \array],
	[\LADSPA, \ar, \nChans, \count],
	[\ListTrig, \kr, \numframes, \signal],
	[\ListTrig2, \kr, \numframes, \signal],
	[\Logger, \kr, \inputArray, \array],
	[\LoopBuf, \ar, \numChannels, \count],
	[\MatchingPResynth, \ar, \activs, \array],
	[\MatchingPResynth, \kr, \activs, \array],
	[\NHHall, \ar, \in, \array],
	[\NTube, \ar, \delaylengtharray, \array],
	[\NTube, \ar, \karray, \array],
	[\NTube, \ar, \lossarray, \array],
	[\NearestN, \kr, \in, \array],
	[\PV_MagMulAdd, \new, \add, \signal],
	[\PV_MagMulAdd, \new, \mul, \signal],
	[\PV_NoiseSynthP, \new, \numFrames, \signal],
	[\PV_SpectralMap, \new, \specBuffer, \signal],
	[\PanX, \ar, \numChans, \count],
	[\PanX, \kr, \numChans, \count],
	[\PanX2D, \ar, \numChansX, \count],
	[\PanX2D, \ar, \numChansY, \count],
	[\PanX2D, \kr, \numChansX, \count],
	[\PanX2D, \kr, \numChansY, \count],
	[\PermModArray, \ar, \pattern, \array],
	[\PlaneTree, \kr, \in, \array],
	[\SOMAreaWr, \kr, \coords, \array],
	[\SOMAreaWr, \kr, \inputdata, \array],
	[\SOMRd, \ar, \inputdata, \array],
	[\SOMRd, \ar, \numdims, \count],
	[\SOMRd, \kr, \inputdata, \array],
	[\SOMRd, \kr, \numdims, \count],
	[\SOMTrain, \kr, \inputdata, \array],
	[\SpectralEntropy, \kr, \numbands, \count],
	[\StkInst, \ar, \args, \array],
	[\TGrains2, \ar, \numChannels, \count],
	[\TGrains3, \ar, \numChannels, \count],
	[\TextVU, \ar, \label, \symbol],
	[\TextVU, \kr, \label, \symbol],
	[\TextVU, \new, \label, \symbol],
	[\VBAP, \ar, \numChans, \count],
	[\VBAP, \kr, \numChans, \count],
	[\WarpZ, \ar, \numChannels, \count],
	[\ZHPF, \ar, \damping, \array],
	[\ZLPF, \ar, \damping, \array]
];

// The rules of checkInputs of the owners of the folder outside HOA. The rule
// of an owner that this table does not name must loop over the first n
// inputs at audio rate, as the 96 HOA classes do. The script measures n.
// BFGrainPanner checks input 2 in its own checkNInputs.
var checkRules = (
	A2B: [\n_inputs, 4],
	B2A: [\n_inputs, 4],
	B2Ster: [\n_inputs, 3],
	B2UHJ: [\n_inputs, 3],
	BFDecode1: [\n_inputs, 4],
	BFEncode1: [\n_inputs, 1],
	BFEncodeSter: [\n_inputs, 2],
	BFGrainPanner: [\audio_at, 2],
	BFManipulate: [\n_inputs, 4],
	Dneuromodule: [\n_inputs, 1],
	EnvDetect: [\same_rate_as_first_input_if_audio],
	FM7: [\custom],
	FMHDecode1: [\n_inputs, 9],
	GreyholeRaw: [\n_inputs, 2],
	JPverbRaw: [\n_inputs, 2],
	MembraneCircle: [\same_rate_as_first_input],
	NHHall: [\n_inputs, 2],
	PermMod: [\same_rate_as_first_input_if_audio],
	PermModArray: [\same_rate_as_first_input_if_audio],
	PermModT: [\same_rate_as_first_input_if_audio],
	Rotate: [\n_inputs, 4],
	Summer: [\same_rate_as_first_input],
	Tilt: [\n_inputs, 4],
	Tumble: [\n_inputs, 4],
	UHJ2B: [\n_inputs, 2],
	WrapSummer: [\same_rate_as_first_input]
);

// The census of the classes whose rate methods do more than new1: each class
// outside UGen that answers ar or kr, and each UGen class whose methods
// create no instance of the class. The crate has a body for each method of a
// class of bodies.
var bodies = [\BFFreeVerb, \BFGVerb, \BHPF, \BLPF, \BhobHiShelf, \BhobLoShelf, \BtoUHJ, \Greyhole,
	\HighShelf, \JPverb, \LRHPF, \LRLPF, \LowShelf, \NHHall, \Notch, \RBJHighShelf, \RBJLowShelf,
	\RBJNotch, \Rotate, \Tilt, \Tumble, \UHJtoB, \ZHPF, \ZLPF];
// The classes of the census without a body, and why.
var leftOut = [
	[\AutoTrack, "no body yet"],
	[\BF, "computes with objects of the language, such as ZPoint"],
	[\BFDecode, "computes with objects of the language, such as ZPoint"],
	[\BFEncode, "computes with objects of the language, such as ZPoint"],
	[\BhobTone, "calls HiShelf, which no class defines"],
	[\CheapVerb, "draws rand of the language at each build"],
	[\DelayWr, "no body yet"],
	[\FFTCentroid, "no body yet"],
	[\Feedback, "no body yet"],
	[\FreeVerb1, "no body yet"],
	[\OnsetsDS, "deprecated, and gives a Function"],
	[\PV_DiffMags, "no body yet"],
	[\PanX2D, "no body yet"],
	[\Phaser, "no body yet"],
	[\PulseDPW, "no body yet"],
	[\Reflections, "computes with objects of the language, such as ZPoint"],
	[\SoftClipAmp, "no body yet"],
	[\StkBowedI, "no body yet"],
	[\TapN, "no body yet"],
	[\Vocode, "no body yet"],
	[\VocodeBand, "no body yet"],
	[\Vocoder, "no body yet"],
	[\WarpOverlap, "no body yet"]
];
var census = bodies ++ leftOut.collect(_.first);

// The classes whose unit no plugin of the folder defines. sclang writes them,
// and scsynth refuses them with "UGen '<name>' not installed.".
var refused = ();
var noUnit = [\BlitB3D, \CQ_Diff, \Clipper32, \FFTSubbandFlux, \RMAFoodChainL, \RegaliaMitraEQ,
	\RosslerResL];

// The units that cannot alias and that no class builds, but those of the
// folder ATK, whose classes come with the quark atk-sc3.
var classless = [\CubicDelay, \TALReverb, \Xover2];

// The methods that the folder adds to classes of the core. Only
// UGen:circleRamp builds a graph, and it has no body yet.
var extensions = [
	[\EnvelopeView, \asEnv],
	[\Function, \benchmark],
	[\Integer, \asPaddedString],
	[\Meta_SequenceableCollection, \newFromFile],
	[\Meta_SequenceableCollection, \tuningScale],
	[\SequenceableCollection, \asAddendsOf],
	[\SequenceableCollection, \asPaddedString],
	[\SequenceableCollection, \aswAddendsOf],
	[\SequenceableCollection, \cauchy],
	[\SequenceableCollection, \gaussian],
	[\SequenceableCollection, \logistic],
	[\SequenceableCollection, \midinote],
	[\SequenceableCollection, \nearestTo],
	[\SequenceableCollection, \nearestToList],
	[\SequenceableCollection, \notemidi],
	[\SequenceableCollection, \pareto],
	[\SequenceableCollection, \poisson],
	[\SequenceableCollection, \writeFile],
	[\SimpleNumber, \asTimeStringLJP],
	[\SimpleNumber, \betarand],
	[\SimpleNumber, \calcPVRecSize],
	[\SimpleNumber, \cauchy],
	[\SimpleNumber, \gaussian],
	[\SimpleNumber, \linrrand],
	[\SimpleNumber, \logistic],
	[\SimpleNumber, \midinote],
	[\SimpleNumber, \pareto],
	[\SimpleNumber, \poisson],
	[\SimpleNumber, \weibull],
	[\String, \notemidi],
	[\UGen, \circleRamp]
];

var probes, toJSON, describeClass, describePseudoClass, scanCantAlias, stats, cantAlias;
var inputs, verdict, bare, first, ruleHolds, blobs, classes, pseudoClasses, helpDir, json;
var measured = 0;

try {
	if (Main.version != "3.14.1") {
		fail.("sclang % is not 3.14.1".format(Main.version));
	};
	if (folder.isNil) { fail.("the conf names no folder SC3plugins") };
	scratch.mkdir;
	Class.allClasses.do { |class|
		if ((core.(class) or: { plugin.(class) }).not) {
			fail.("% comes from %".format(class, class.filenameSymbol));
		};
		(class.methods ? []).do { |method|
			if ((core.(method) or: { plugin.(method) }).not) {
				fail.("% comes from %".format(method, method.filenameSymbol));
			};
			if (plugin.(class).not and: { plugin.(method) }
				and: { extensions.any { |entry| entry == [class.name, method.name] }.not }) {
				fail.("the folder adds %, which extensions does not name".format(method));
			};
		};
	};
	extensions.do { |entry|
		if ((entry[0].asClass !? { |class| class.findMethod(entry[1]) }).isNil) {
			fail.("extensions names %:%, which the folder does not add".format(*entry));
		};
	};

	// Each class file of the folder is the blob of its plugin and its name at
	// the tag.
	blobs = Dictionary.new;
	"git -C % ls-tree -r % source/".format(source.shellQuote, tag).unixCmdGetStdOut.split($\n)
		.do { |line|
			var fields = line.split($\t), parts, key;
			if (fields.size == 2 and: { fields[1].endsWith(".sc") }) {
				parts = fields[1].split($/);
				key = parts[1] ++ "/" ++ parts.last;
				blobs[key] = (blobs[key] ? []).add(fields[0].split($ ).last);
			};
		};
	if (blobs.isEmpty) { fail.("git lists no class file at % in %".format(tag, source)) };
	PathName(folder).deepFiles.select { |file| file.extension == "sc" }.do { |file|
		var key = file.fullPath.drop(folder.size + 1).split($/).first ++ "/" ++ file.fileName;
		var id = "git hash-object %".format(file.fullPath.shellQuote).unixCmdGetStdOut.stripWhiteSpace;
		if ((blobs[key] ? []).includesEqual(id).not) {
			fail.("% differs from its file at %".format(file.fullPath, tag));
		};
	};

	noUnit.do { |name| refused[name] = "no plugin of sc3-plugins 3.14.0 defines the unit" };
	probes = (root +/+ "generator" +/+ "probes.scd").load.((
		fail: fail, argumentKinds: argumentKinds, bodies: bodies, census: census,
		audioRateInputs: [], levelArguments: [], passthroughs: [], silentZeros: [],
		audioTogether: [], refused: refused
	));
	#toJSON, describeClass, describePseudoClass, scanCantAlias, stats, cantAlias =
		probes.atAll([\toJSON, \describeClass, \describePseudoClass, \scanCantAlias, \stats,
			\cantAlias]);
	probes[\checkRules].putAll(checkRules);
	argumentKinds.do { |entry|
		var method = entry[0].asClass !? { |class| class.class.findMethod(entry[1]) };
		if (method.isNil or: { (method.argNames ? []).includes(entry[2]).not }) {
			fail.("argumentKinds names %.% %, which no method has".format(*entry));
		};
	};

	// SCDoc writes docmap.js into its target folder, by default the help
	// cache of the IDE.
	helpDir = Platform.defaultTempDir +/+ "scsynth-synthdef-scdoc";
	helpDir.mkdir;
	SCDoc.helpTargetDir_(helpDir);
	SCDoc.indexAllDocuments;

	// The census: each class of the folder outside UGen that answers ar or kr,
	// and each class that it names, must exist and be named.
	(census ++ noUnit).do { |name|
		if (name.asClass.isNil) { fail.("a table names %, which is no class".format(name)) };
	};
	Class.allClasses.do { |class|
		if (plugin.(class) and: { class.isMetaClass.not }
			and: { (class.superclasses ? []).includes(UGen).not }
			and: { [\ar, \kr].any { |selector| class.class.findRespondingMethodFor(selector).notNil } }
			and: { census.includes(class.name).not }) {
			fail.("% answers ar or kr, and the census does not name it".format(class.name));
		};
	};

	// The verdict that the crate gives a UGen at `rate` whose inputs run at
	// the rates `rates`, under `rule`: true when it passes, or nil for a rule
	// that this test does not cover.
	verdict = { |rule, rate, rates|
		switch (rule[0])
		{ \valid } { true }
		{ \same_rate_as_first_input } { rates[0] == rate }
		{ \same_rate_as_first_input_if_audio } { rate != \audio or: { rates[0] == \audio } }
		{ \n_inputs } { rate != \audio or: { rates.keep(rule[1]).every { |input| input == \audio } } }
		{ \audio_at } { rate != \audio or: { rates[rule[1]] == \audio } }
	};
	// A bare instance of `class` at `rate`, with inputs at the rates `rates`.
	inputs = (audio: SinOsc.newCopyArgs.rate_(\audio), control: SinOsc.newCopyArgs.rate_(\control));
	bare = { |class, rate, rates| class.newCopyArgs.rate_(rate).inputs_(rates.collect(inputs[_])) };
	// The rates of 128 inputs, the first `count` at audio rate.
	first = { |count| 128.collect { |i| if (i < count) { \audio } { \control } } };
	// Whether checkInputs of `class` gives the verdict of `rule`, at audio rate
	// and at control rate, for each pattern of rates.
	ruleHolds = { |class, rule|
		var patterns = [first.(0), first.(1), first.(128), first.(128).put(0, \control)];
		if (rule[0] == \n_inputs) { patterns = patterns ++ [first.(rule[1] - 1), first.(rule[1])] };
		if (rule[0] == \audio_at) {
			patterns = patterns ++ [first.(0).put(rule[1], \audio), first.(128).put(rule[1], \control)];
		};
		[\audio, \control].every { |rate|
			patterns.every { |rates|
				var expected = verdict.(rule, rate, rates);
				expected.isNil or: { bare.(class, rate, rates).checkInputs.isNil == expected }
			}
		}
	};
	UGen.allSubclasses.select(plugin).do { |class|
		var owner = class.findRespondingMethodFor(\checkInputs).ownerClass;
		var rule = probes[\checkRules][owner.name], count;
		if (rule.isNil and: { plugin.(owner) }) {
			count = (1..128).detect { |n| bare.(class, \audio, first.(n)).checkInputs.isNil };
			if (count.isNil) {
				fail.("the checkInputs of % is a rule that no table names".format(owner.name));
			};
			rule = [\n_inputs, count];
			probes[\checkRules][owner.name] = rule;
			measured = measured + 1;
		};
		if (rule.notNil and: { rule[0] != \custom } and: { ruleHolds.(class, rule).not }) {
			fail.("the checkInputs of % does not hold %".format(class.name, rule));
		};
	};

	// The units that take buffers for their outputs before they free those of
	// their inputs, in the C++ files of each plugin, which the build compiles.
	"git -C % ls-tree -r --name-only % source/".format(source.shellQuote, tag).unixCmdGetStdOut
		.split($\n).select { |path| path.split($/).size == 3 and: { path.endsWith(".cpp") } }
		.do { |path|
			scanCantAlias.(path, show.(path)).do { |name|
				var class = name.asClass;
				case
				{ class.notNil } {
					if ((class.superclasses ? []).includes(UGen).not or: { plugin.(class).not }) {
						fail.("% defines %, which is no UGen class of the folder".format(path, name));
					};
				}
				{ path.beginsWith("source/ATK/").not and: { classless.includes(name).not } } {
					fail.("% defines %, which no class builds".format(path, name));
				};
			};
		};
	if (cantAlias.isEmpty) { fail.("no plugin of % defines a unit that cannot alias".format(source)) };

	classes = UGen.allSubclasses.select(plugin).asArray
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describeClass);
	pseudoClasses = bodies.collect(_.asClass)
		.reject { |class| (class.superclasses ? []).includes(UGen) }
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describePseudoClass);

	json = toJSON.([
		\sclang -> Main.version,
		\sc3_plugins -> tag.drop(8),
		\classes -> classes,
		\pseudo_classes -> pseudoClasses
	]) ++ "\n";
	File.use(root +/+ "generator" +/+ "sc3_plugins" +/+ "catalog.json", "w", { |file| file.write(json) });
	("sc3_plugins/catalog.scd: % classes, % of % methods addable, % pseudo-UGen methods, "
		++ "% classes outside UGen, % rules of checkInputs measured, % units that cannot alias")
		.format(classes.size, stats[\addable], stats[\methods], stats[\pseudo], pseudoClasses.size,
			measured, cantAlias.select { |name| name.asClass.notNil }.asSet.size).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
