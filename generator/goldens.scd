// generator/goldens.scd: the writers of the golden files, for
// generator/fixtures.scd and generator/sc3_plugins/fixtures.scd. The file
// answers a function of the folder of the golden files and of `fail`, which
// answers an Event of the writers and of the count of the written files.
{ |fixtures, fail|
	var build, write, writeCases, refused, written = 0;

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

	(build: build, write: write, writeCases: writeCases, refused: refused, written: { written })
}
