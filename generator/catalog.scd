// generator/catalog.json: the catalogue of the core UGens, the tables of the
// operators, and the specs of Spec.specs, from sclang 3.14.1 and the sources
// of SuperCollider 3.14.1.
// Run it from the root of the repository, with absolute paths, since
// ~/.bin/sclang changes the directory:
//
//   sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/catalog.scd"
//
// SC_SOURCE names the clone of SuperCollider, ~/Projects/supercollider by
// default. The script reads its files at the tag Version-3.14.1, whatever is
// checked out. It exits with 1 when a check fails, and the section "The
// catalogue" of the README tells the checks. The probes come from
// generator/probes.scd.
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
// its size and its ASCII codes. A curve is a shape by name, a value, or an
// array of them. A signal entry marks a name that only looks like an array.
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
	[\Env, \adsr, \curve, \curve],
	[\Env, \asr, \curve, \curve],
	[\Env, \circle, \curve, \curve],
	[\Env, \circle, \levels, \array],
	[\Env, \circle, \times, \array],
	[\Env, \cutoff, \curve, \curve],
	[\Env, \dadsr, \curve, \curve],
	[\Env, \linen, \curve, \curve],
	[\Env, \new, \curve, \curve],
	[\Env, \new, \levels, \array],
	[\Env, \new, \times, \array],
	[\Env, \newClear, \numChannels, \count],
	[\Env, \newClear, \numSegments, \count],
	[\Env, \pairs, \curve, \curve],
	[\Env, \pairs, \pairs, \array],
	[\Env, \perc, \curve, \curve],
	[\Env, \step, \levels, \array],
	[\Env, \step, \times, \array],
	[\Env, \xyc, \xyc, \array],
	[\EnvGate, \new, \curve, \curve],
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
	[\NamedControl, \ar, \name, \symbol],
	[\NamedControl, \ar, \spec, \symbol],
	[\NamedControl, \ar, \values, \array],
	[\NamedControl, \ir, \name, \symbol],
	[\NamedControl, \ir, \spec, \symbol],
	[\NamedControl, \ir, \values, \array],
	[\NamedControl, \kr, \fixedLag, \bool],
	[\NamedControl, \kr, \name, \symbol],
	[\NamedControl, \kr, \spec, \symbol],
	[\NamedControl, \kr, \values, \array],
	[\NamedControl, \new, \fixedLag, \bool],
	[\NamedControl, \new, \name, \symbol],
	[\NamedControl, \new, \rate, \symbol],
	[\NamedControl, \new, \spec, \symbol],
	[\NamedControl, \new, \values, \array],
	[\NamedControl, \tr, \name, \symbol],
	[\NamedControl, \tr, \spec, \symbol],
	[\NamedControl, \tr, \values, \array],
	[\NumChannels, \ar, \input, \array],
	[\NumChannels, \ar, \mixdown, \bool],
	[\NumChannels, \ar, \numChannels, \count],
	[\Out, \ar, \channelsArray, \array],
	[\Out, \kr, \channelsArray, \array],
	[\PV_ChainUGen, \pvcalc, \func, \function],
	[\PV_ChainUGen, \pvcalc, \numframes, \signal],
	[\PV_ChainUGen, \pvcalc2, \func, \function],
	[\PV_ChainUGen, \pvcalc2, \numframes, \signal],
	[\PV_ChainUGen, \pvcollect, \func, \function],
	[\PV_ChainUGen, \pvcollect, \numframes, \signal],
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
	[\UGen, \bilin, \clip, \symbol],
	[\UGen, \curvelin, \clip, \symbol],
	[\UGen, \explin, \clip, \symbol],
	[\UGen, \expexp, \clip, \symbol],
	[\UGen, \lincurve, \clip, \symbol],
	[\UGen, \linexp, \clip, \symbol],
	[\UGen, \linlin, \clip, \symbol],
	[\UGen, \dpoll, \label, \symbol],
	[\UGen, \poll, \label, \symbol],
	[\UGen, \prune, \type, \symbol],
	[\UGen, \varlag, \warp, \curve],
	[\VarLag, \ar, \warp, \curve],
	[\VarLag, \kr, \warp, \curve],
	[\Warp1, \ar, \numChannels, \count],
	[\XOut, \ar, \channelsArray, \array],
	[\XOut, \kr, \channelsArray, \array]
];
// The census of the classes whose rate methods do more than new1: the
// pseudo-UGens, whose rate methods build other UGens, the classes of 0.4, and
// the controls, which take their special index from the build. The crate has
// a body for each rate method of a class of bodies, but a method that sclang
// refuses as not valid for its subclass.
var bodies = [\AudioControl, \BeatTrack, \BHiPass4, \BLowPass4, \Changed, \CompanderD,
	\Control, \Dstutter, \DynKlang, \DynKlank, \EnvGate, \HilbertFIR, \Klang, \Klank, \LagControl,
	\LinLin, \LinSelectX, \LocalBuf, \LocalIn, \Mix, \MultiTap, \NamedControl,
	\NumChannels, \PMOsc, \PackFFT, \PingPong, \Poll, \SelectX, \SelectXFocus, \Silent, \SoundIn,
	\Splay, \SplayAz, \Sum3, \Sum4, \TChoose, \TWChoose, \Tap, \TrigControl, \UnpackFFT,
	\VarLag];
// The classes of the census that have no body, and why.
var leftOut = [
	[\AudioIn, "deprecated since 3.10"],
	[\Dunique, "builds its reader in asUGenInput, at each use as an input"],
	[\InBus, "reads a Bus of sclang, for NodeProxy"],
	[\SplayZ, "deprecated since 3.10"],
	[\XIn, "a helper of InBus"],
	[\XInFeedback, "a helper of InBus"]
];
// Classes that the census meets but that are no pseudo-UGens, and why.
var notPseudo = [
	[\LevelComp, "a helper of Splay and SplayAz"],
	[\MulAdd, "folds: madd of the builder"],
	[\Ndef, "a proxy of JITLib"]
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
// The arguments that go to audio rate together before multiNew: when one of
// them is at audio rate, K2A.ar takes each other one, in order. By the class
// that owns the method. testAudioTogether tests each entry.
var audioTogether = [
	[\DemandEnvGen, \ar, [\gate, \reset]]
];

// The class methods of Env that make an envelope, and why the others do not.
var envMethods = [\new, \newClear, \triangle, \sine, \perc, \linen, \xyc, \pairs, \step,
	\cutoff, \dadsr, \adsr, \asr, \circle];
var notEnvMethods = [
	[\initClass, "sets up the class"],
	[\shapeNames, "gives the table of the shapes"],
	[\shapeNumber, "gives the number of a shape"]
];

// The instance methods of Env that map its levels to a range.
var envRanges = [\range, \exprange, \curverange];

// The instance methods of PV_ChainUGen that take a function, and why the
// others do not count.
var pvMethods = [\pvcalc, \pvcalc2, \pvcollect];
var notPvMethods = [
	[\addCopiesIfNeeded, "a step of the build"],
	[\fftSize, "the size of the FFT, which fft_size holds"]
];

// The instance methods of UGen that build graphs, and why the others do
// not count.
var ugenMethods = [\range, \exprange, \curverange, \unipolar, \bipolar, \linlin, \linexp,
	\explin, \expexp, \lincurve, \curvelin, \bilin, \prune, \clip, \fold, \wrap, \lag, \lag2,
	\lag3, \lagud, \lag2ud, \lag3ud, \varlag, \slew, \blend, \minNyquist, \degrad, \raddeg,
	\moddif, \sanitize, \poll, \dpoll, \checkBadValues, \snap, \softRound, \binaryValue,
	\isPositive, \isNegative, \isStrictlyPositive, '|==|', \degreeToKey, \if];
var notUgenMethods = [
	['@', "gives a Point"],
	[\addToSynth, "a step of the build"],
	[\antecedents, "a step of the build"],
	[\antecedents_, "a step of the build"],
	[\argNameForInputAt, "names an input for a message"],
	[\argNamesInputsOffset, "names an input for a message"],
	[\asComplex, "gives a Complex"],
	[\asControlInput, "a step of the build"],
	[\asUGenInput, "a step of the build"],
	[\belaScope, "plays on a Bela board"],
	[\checkInputs, "a step of the build"],
	[\checkNInputs, "a step of the build"],
	[\checkSameRateAsFirstInput, "a step of the build"],
	[\checkValidInputs, "a step of the build"],
	[\collectConstants, "a step of the build"],
	[\composeBinaryOp, "binary_op"],
	[\composeNAryOp, "is not implemented"],
	[\composeUnaryOp, "unary_op"],
	[\copy, "gives the UGen itself"],
	[\descendants, "a step of the build"],
	[\descendants_, "a step of the build"],
	[\dumpArgs, "prints"],
	[\dumpName, "prints"],
	[\init, "a step of the build"],
	[\initTopoSort, "a step of the build"],
	[\inputs, "a step of the build"],
	[\inputs_, "a step of the build"],
	[\isUGen, "a test of the language"],
	[\isValidUGenInput, "a test of the language"],
	[\madd, "madd"],
	[\makeAvailable, "a step of the build"],
	[\methodSelectorForRate, "a step of the build"],
	[\name, "a step of the build"],
	[\numChannels, "a step of the build"],
	[\numInputs, "a step of the build"],
	[\numOutputs, "a step of the build"],
	[\optimizeGraph, "a step of the build"],
	[\outputIndex, "a step of the build"],
	[\performBinaryOpOnComplex, "gives a Complex"],
	[\performDeadCodeElimination, "a step of the build"],
	[\prReverseLazyEquals, "the reverse of |==|"],
	[\rate, "a step of the build"],
	[\rate_, "a step of the build"],
	[\rateNumber, "a step of the build"],
	[\removeAntecedent, "a step of the build"],
	[\reverseComposeBinaryOp, "binary_op"],
	[\schedule, "a step of the build"],
	[\scope, "plays on the server"],
	[\signalRange, "the signal_range of the class"],
	[\specialIndex, "a step of the build"],
	[\specialIndex_, "a step of the build"],
	[\synthDef, "a step of the build"],
	[\synthDef_, "a step of the build"],
	[\synthIndex, "a step of the build"],
	[\synthIndex_, "a step of the build"],
	[\widthFirstAntecedents, "a step of the build"],
	[\widthFirstAntecedents_, "a step of the build"],
	[\writeDef, "a step of the build"],
	[\writeDefOld, "a step of the build"],
	[\writeInputSpec, "a step of the build"],
	[\writeInputSpecOld, "a step of the build"],
	[\writeOutputSpec, "a step of the build"],
	[\writeOutputSpecs, "a step of the build"],
	[\writesToBus, "a step of the build"]
];

var probes, stripComments, toJSON, number, requireEntries, describeMethods, describeClass;
var describePseudoClass, scanCantAlias, stats, cantAlias, enumFrom, selectorsOf, operatorTable;
var opcodes, parseNode, unaryEnum, binaryEnum, unaryNames, binaryNames, unary, binary;
var classes, pseudoClasses, specs, envs, ranges, pvs, ugens;
var helpDir, json;

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

	probes = (root +/+ "generator" +/+ "probes.scd").load.((
		fail: fail, argumentKinds: argumentKinds, bodies: bodies, census: census,
		audioRateInputs: audioRateInputs, levelArguments: levelArguments,
		passthroughs: passthroughs, silentZeros: silentZeros, audioTogether: audioTogether
	));
	#stripComments, toJSON, number, requireEntries, describeMethods, describeClass,
		describePseudoClass, scanCantAlias, stats, cantAlias = probes.atAll([\stripComments,
		\toJSON, \number, \requireEntries, \describeMethods, \describeClass,
		\describePseudoClass, \scanCantAlias, \stats, \cantAlias]);

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

	// The units that take buffers for their outputs before they free those of
	// their inputs, in the plugins at the tag.
	"git -C % ls-tree -r --name-only Version-3.14.1 server/plugins/".format(source.shellQuote)
		.unixCmdGetStdOut.split($\n).reject(_.isEmpty).do { |path|
			scanCantAlias.(path, show.(path));
		};
	if (cantAlias.isEmpty) { fail.("no plugin of % defines a unit that cannot alias".format(source)) };
	cantAlias.do { |name|
		var class = name.asClass;
		if (class.isNil or: { (class.superclasses ? []).includes(UGen).not } or: { core.(class).not }) {
			fail.("the plugins define %, which is no UGen class of the core".format(name));
		};
	};

	classes = UGen.allSubclasses.asArray
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describeClass);
	pseudoClasses = bodies.collect(_.asClass)
		.reject { |class| (class.superclasses ? []).includes(UGen) }
		.sort { |a, b| a.name.asString < b.name.asString }
		.collect(describePseudoClass);
	// The specs of Spec.specs, which give the default of a NamedControl
	// with a spec, and of an argument without a default when the metadata
	// has specs.
	specs = Spec.specs.keys.asArray.sort.collect { |name|
		var spec = Spec.specs[name], warp;
		if (spec.class != ControlSpec) {
			fail.("the spec % is a %, not a ControlSpec".format(name, spec.class));
		};
		warp = spec.warp.asSpecifier;
		if (warp.isKindOf(Symbol).not) {
			fail.("the warp of the spec % is %, not a symbol".format(name, warp));
		};
		[
			\name -> name,
			\minval -> number.(spec.minval),
			\maxval -> number.(spec.maxval),
			\warp -> warp,
			\step -> number.(spec.step),
			\default -> number.(spec.default),
			\units -> spec.units
		]
	};
	// The class methods of Env that make an envelope, the instance methods of
	// PV_ChainUGen that take a function, and the instance methods of UGen
	// that build graphs, with their arguments. For the methods of UGen, also
	// the classes whose methods a Float and an Array answer, and the
	// subclasses of UGen with a method of their own.
	requireEntries.(Env.class.methods, "Env.", envMethods, notEnvMethods, "envMethods", "notEnvMethods");
	requireEntries.(PV_ChainUGen.methods, "PV_ChainUGen:", pvMethods, notPvMethods,
		"pvMethods", "notPvMethods");
	requireEntries.(UGen.methods, "UGen:", ugenMethods, notUgenMethods,
		"ugenMethods", "notUgenMethods");
	pvs = describeMethods.(PV_ChainUGen, pvMethods, PV_ChainUGen.findMethod(_), "method");
	ugens = probes[\describeUGenMethods].(ugenMethods);
	envs = describeMethods.(Env, envMethods, Env.class.findMethod(_), "class method");
	ranges = describeMethods.(Env, envRanges, Env.findMethod(_), "method");

	json = toJSON.([
		\sclang -> Main.version,
		\operators -> [\unary -> unary, \binary -> binary],
		\classes -> classes,
		\pseudo_classes -> pseudoClasses,
		\specs -> specs,
		\env_methods -> envs,
		\env_ranges -> ranges,
		\pv_methods -> pvs,
		\ugen_methods -> ugens
	]) ++ "\n";
	File.use(root +/+ "generator" +/+ "catalog.json", "w", { |file| file.write(json) });
	("catalog.scd: % unary and % binary operators, % classes, % of % methods addable, "
		++ "% pseudo-UGen methods, % classes outside UGen, % specs, % methods of Env, "
		++ "% ranges of Env, % methods of PV_ChainUGen, % units that cannot alias, "
		++ "% methods of UGen")
		.format(unary.size, binary.size, classes.size, stats[\addable], stats[\methods], stats[\pseudo],
			pseudoClasses.size, specs.size, envs.size, ranges.size, pvs.size, cantAlias.size,
			ugens.size).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
