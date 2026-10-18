// generator/catalog.json: the tables of the operators, from the sources of
// SuperCollider 3.14.1 and from sclang 3.14.1 itself. Run it from the root of
// the repository, with absolute paths, since ~/.bin/sclang changes the
// directory:
//
//   sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/catalog.scd"
//
// SC_SOURCE names the clone of SuperCollider, ~/Projects/supercollider by
// default. The script reads its files at the tag Version-3.14.1, whatever is
// checked out. It exits with 1 if sclang is not 3.14.1, if a class or a
// method comes from outside the core class library, or if sclang and the
// sources disagree on an operator.
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
var stripComments, enumFrom, selectorsOf, operatorTable, quote, writeList;
var opcodes, parseNode, unaryEnum, binaryEnum, unaryNames, binaryNames, unary, binary;
var json;

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

	writeList = { |items, indent|
		"[\n" ++ items.collect { |item|
			indent ++ "  " ++ if (item.isNil) { "null" } { quote.(item) }
		}.join(",\n") ++ "\n" ++ indent ++ "]"
	};

	opcodes = stripComments.(show.("lang/LangSource/Opcodes.h"));
	parseNode = stripComments.(show.("lang/LangSource/PyrParseNode.cpp"));
	unaryEnum = enumFrom.(opcodes, "opNeg,");
	binaryEnum = enumFrom.(opcodes, "opAdd,");
	#unaryNames, binaryNames = selectorsOf.(parseNode);
	unary = operatorTable.(unaryEnum, unaryNames);
	binary = operatorTable.(binaryEnum, binaryNames);

	json = "{\n"
		++ "  \"sclang\": " ++ quote.(Main.version) ++ ",\n"
		++ "  \"operators\": {\n"
		++ "    \"unary\": " ++ writeList.(unary, "    ") ++ ",\n"
		++ "    \"binary\": " ++ writeList.(binary, "    ") ++ "\n"
		++ "  }\n"
		++ "}\n";
	File.use(root +/+ "generator" +/+ "catalog.json", "w", { |file| file.write(json) });
	"catalog.scd: % unary and % binary operators".format(unary.size, binary.size).postln;
	0.exit;
} { |error|
	error.reportError;
	1.exit;
};
)
