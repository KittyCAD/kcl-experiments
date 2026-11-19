# Showing and hiding geometry (construction geometry, assemblies)

I propose an 'implicit show' syntax (with an explicit version as an alternative), where geometry which is consumed (by assigning into a variable, passing to a function, etc.) is treated as construction geometry and geometry which escapes to the top-level of a module is rendered (or becomes part of an assembly).

## Background and motivation

Most CAD software has the concept of *construction geometry* which is geometry used for constructing other geometry but which is not itself rendered. We currently don't have explicit support for this in KittyCAD, but the engine does make some choices about which geometry is rendered. In KCL, non-rendered geometry might take other forms too, e.g., geometry in functions or in imported modules. It's also likely to be used in many places, e.g., creating a single instance of an object and then using a pattern to create multiple instances, where we don't want to render the original instance.

One particular question to answer is how geometry should work with imported modules (which I envisage to be the way that KCL supports assemblies). To recap the current and [proposed](https://github.com/KittyCAD/kcl-experiments/pull/12) syntax:

- modules are declared in a separate file
- exported functions of a file can be imported using `import foo from 'bar.kcl'`
- the module itself (i.e., the geometry defined at the top-level of the file) can be imported using `import 'bar.kcl'` (creates a single object bound to the variable `bar`, which should be able to be used as an assembly)

Currently, objects are effectively rendered by virtue of the side effects of execution (issuing API calls to the engine). Therefore, exactly how all the above might work is a bit vague. I'll spell some of it out here, focussing on the user-facing syntax. More of the fundamentals are discussed in the [foundations design doc](foundations.md), including side-effects.


## Proposal: implicit `show`

The basics of the proposal is that an object that is defined at the top level is rendered by the engine unless it is passed into a function, used in a pipeline, or assigned into a variable (or used in some other way we add to KCL in the future). Using the result of a function or a variable at the top level will render it. E.g.,

```
// Result of function call, rendered
makeSphere()

// Result of pipeline, rendered (nothing intermediate in the pipeline is rendered)
startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// As above, but assigned into a variable, nothing is rendered
object = startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Variable is used, result of pipeline is rendered
object
```

Exactly how this is implemented with respect to the engine is left to the [foundations design doc](foundations.md), however, whether something is rendered or not must (I think) be explicit in the API calls.

When a function is declared, nothing is rendered (if an entity escapes usage into the top-level of the function, then there should be an error or warning). When a function is called, the result returned by the function follows the usage rules as described above.

When a file is imported as a module, all geometry which would be rendered is collected into the imported assembly (that is `bar` in `import 'bar.kcl'`). Anything else is not directly imported into the assembly, though it may be used indirectly, or if it is `export`ed, then it may be individually imported. E.g., if the above example code were imported, the resulting assembly would contain the result of `makeSphere()`, the first pipeline, and via `object`, the second pipeline.

Since imported code is treated as immutable, geometry in modules cannot be changed from shown/hidden without editing the file directly.

Naming of objects within assemblies is covered in the [modules design doc](modules.md).

Implicit `show` would be mostly backwards compatible, I believe. Where there are changes, they would be a bit confusing, but I believe that we would require these changes in any case to have a better model of side-effects in the language.

### GUI

This proposal focusses on KCL, not the UI, so I won't get too deep into things, but I have the following recommendations/expectations for the UI:

- By default non-rendered geometry is not rendered (surprise!)
- There is some UI to show all construction geometry or a specific construction object rendered as a dashed wireframe, this does not affect the KCL program
  - this might be implemented as a check box in the margin of the KCL code and/or in some object summary, as well as in a context menu, command, etc.
- There is some UI to properly show/hide geometry, which is a simple code mod:
  - To hide geometry, insert a dummy variable, e.g., `makeSphere()` becomes `object003 = makeSphere()` or remove (or comment out) the use of a variable,
  - To show geometry, remove a variable if the variable is not used elsewhere, e.g., `object003 = makeSphere()` becomes `makeSphere()`, or use the variable at the top-level if it is, e.g., `object003 = makeSphere()` becomes `object003 = makeSphere(); object003`
- I expect that construction geometry in assemblies is handled differently, future work...

A note on a previous proposal: checkboxes in the source code editor should not be the primary mechanism for showing/hiding geometry in a permanent way, since it blurs the code/UI distinction and would not have a permanent representation in the source code.

## Alternatives

### Explicit `show`

An alternative is to use a `show` keyword (or some other keyword or sigil). The rules as above remain basically the same, in particular with respect to modules and assemblies. However, rendering an object requires `show`. We have a 'not used, but not shown' state which is a little ambiguous and should probably be an error or warning. E.g.,

```
// Result of function call, not rendered, probably should be a warning
makeSphere()
// Rendered
show makeSphere()

// Result of pipeline, rendered (nothing intermediate in the pipeline is rendered)
show startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Also rendered
show object1 = startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Not rendered
object2 = startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Variable is shown, result of pipeline is rendered
show object2
```

This is more explicit which I think is both better (self-explanatory) and worse (more boilerplate for an extremely common concept).

`show` in functions would be an error.

`show` could be permitted in sub-expressions where `show expr` produces the result of `expr` and has the side effect of rendering it. E.g., `foo(show bar)` would render `bar` and pass it to `foo`. This is not possible with the implicit syntax (the work around is to use the variable either before or after as well as in the function call, easy enough here, but less ergonomic in the middle of a pipeline), but I'm not sure if this is a common use case.

We could support both implicit and explicit `show` but I think that is a bad idea since it would be confusing and there is not much benefit.

Explicit `show` would require many changes to existing code so is not as backwards compatible.

### Explicit `hide`

We could make geometry shown by default and require an explicit `hide` keyword. It is closer to existing CAD software though (where geometry is shown by default and the user opts-in to making it construction geometry).

```
// Not rendered, not sure why you'd want to do this without assigning into a variable
hide makeSphere()
// Rendered
makeSphere()

// Result of pipeline, rendered (nothing intermediate in the pipeline is rendered)
startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Also rendered
object1 = startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Not rendered
hide object2 = startSketchOn(...)
  |> startPath()
  |> ...
  |> close()
  |> extrude()

// Variable is shown, result of pipeline is rendered
object2
```

The first example here is why I think explicit `hide` is a bit weird and explicit `show` is better.

### Property of objects

When creating an object, whether it is to be used for construction or rendering could be included as a parameter. This is used in at least some other CAD software and is perhaps a good model for the engine API. However I don't think it is a good fit for KCL. Here, we might want to create construction geometry not just to reference from other geometry, but also as a template which is replicated and transformed. In the latter case, we want the original to be construction geometry and its clones to be rendered. That would require changing the construction/rendered property of the object during replication, but when exactly the user wants to do that is not trivial and we'd need some way to specify it as part of the replication/transformation functionality.

More philosophically, I believe that whether an object is used for construction or rendered is not a property of the object itself, but of how the object is used.

## Extensions

### Leading underscores

A variable name with a leading underscore (including just an underscore) can be declared but not used (using it is an error or warning, likewise not using a variable without a leading underscore should also be a warning). This makes it easy to temporarily hide geometry using `_ = ...`. This is useful because it means the UI or user does not need to create a name for the variable, but also it shows intention: that this variable is being used primarily to hide the geometry, not necessarily for reuse.

### Implicit `return`

When defining functions, we could allow eliding the `return` keyword in the return expression (final line) of the function body. This follows Rust, but note that Rust uses semicolons and KCL does not, so there is a different feel.

This would make functions a bit more like the top-level of a module and I think that would be intuitive. It would also match the usual semantics for blocks in programming languages (and how I imagine they would work in KCL). However, one difference is that we should require explicit `return` on other lines of a function body, whereas at the top-level this is not needed.

Not requiring explicit return elsewhere in the function would make it a bit easy to have weird bugs, although this could be addressed somewhat with syntax highlighting in the editor. It would also have different semantics to the top-level where we effectively return all values and continue executing, whereas in functions we return one value and terminate execution (we could allow multiple implicit returns as sugar for returning a tuple of value and explicit returns for early returns, but I think that is getting a bit weird).

## Open questions

### Sending construction geometry to the engine

In theory, KCL should only send rendered geometry to the engine and construction geometry should only be used locally. However, that doesn't fit the current design. For example, we might create a line as construction geometry and then terminate a rendered line at the centre of the line. The engine would require knowledge of the construction line. We could send all geometry to the engine, labelled as construction/rendered or we could try to infer which geometry is required to be sent to the engine. I would hope the latter is purely an optimisation of the former, but haven't tried to prove that out.

### Construction geometry within sketches

E.g, (from [#1553](https://github.com/KittyCAD/modeling-app/issues/1553)):

```
startSketchOn('XY')
  |> startProfileAt()
  |> line()
  |> line()
  |> construction(circle())
  |> line()
  |> construction(line({from:[], to:[]}))
```

Neither implicit nor explicit `show` address that very well. We could support a `construction` function specifically for use within sketches/pipelines. However, I think that fundamentally, construction geometry does not fit well with the ordering ethos of pipelining. I've been thinking of a block syntax for sketching which does not have an ordering, and in that case, construction geometry works similarly to functions. For example, the above example would become:

```
sketch(on = XY) {
  l1 = line(...)
  l2 = line(...)
  cnstCircle = circle(...)
  cnstLine(...)
  l3 = line(...)
  enclose(l1, l2, l3)
}
```

As in the original example, three lines are rendered (the result of the `enclose` function) and a circle and one line are used for construction.
