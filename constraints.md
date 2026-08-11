# Constraints

Some thoughts on improving constraints in KCL (partially in response to [Jon](https://github.com/KittyCAD/kcl-experiments/pull/13/files?short_path=ef936d3#diff-ef936d31dc0597616f11930d898cc0e5442a9139c5030dc78acb905381e50749) and [Josh](https://github.com/KittyCAD/notes/blob/main/docs/personal/Gomez-Josh/kcl/kcl-refactor.md)'s docs).

## How I understand the constraints issue

- We have some constraint support in KCL
  - It doesn't meet expectations from trad CAD software
  - It has the issue that constraints are implicit in the code, which makes them hard to understand for programmers and fragile for the UI to manipulate
  - There are some advantages of the current approach, see [#111](https://github.com/KittyCAD/modeling-app/issues/111)
- Constraints are a key part of the trad CAD workflow, not just another feature
  - There is a very strong need for us to have a good solution
- Constraints are a hard feature to support
  - The UI seems fundamentally difficult to make usable
  - There are complex interactions between UI, KCL, and the engine
  - There is a huge design space for KCL and the UI; designing an ergonomic system of constraints is hard
  - Implementation is really, really hard. There is a huge amount of prior art to lean on, but even understanding the lay of the land is a big job, choosing and integrating an existing constraint solver is going to be huge too. Writing out own would be fun, but a lot of work.
  - Generally, constraint solving is
    - opaque, so giving good user feedback and error messages is hard
    - undecidable, so some things which should work will not, and this is often hard to predict or to communicate to users
    - not performant, constrain solving can take a long time and use a lot of resources; how slow/resource-hungry it is to solve any given system of constraints is unpredictable
  - Designing a 'small' constraint system which has an easy solution is hard. A lot of trivial constraint problems require a fully powerful constraint solver

Jon and Josh's docs are not just suggesting a change to the constraint system, but a deep change to the language and to typical workflows. It is good to think about this!

But time-wise, it is tricky, not just because of the time to design and implement a new system (and this will be long because of the way the implementation of KCL is tightly integrated with the UI and engine, we're basically thinking about big changes to the whole system), but also because once we get the big changes done, there are a lot of little changes to go from the equivalent of where we are now, to something we're happy to launch (like even without any change to the feel of the language, we'd have to make big compromises in order to launch early next year). Anyway, 1.0 isn't the end of the story and I'm not going to consider timing too much in this doc, but something to keep in mind when we think about what comes now and what comes later.


## What's missing

What is actually missing from the modelling app to make a good user experience?

- Features. A lot of the things Josh did in OnShape and couldn't in KCL are not fundamental things, just missing features or features that need to be more flexible and powerful. We should track these and make sure we're working towards a good place, but I'm not too worried about it.
- UX. The OnShape UI is clearly more polished, more usable, and more fully-featured than ours. We need to work on that, but again, I don't think there is anything fundamental, just incremental improvements.
- Profiles which are not paths. Our profiles are optimised for paths. For Josh's sketches which were paths, the workflow for OnShape and KMA were similar. For profiles which aren't paths, we fall off our ergonomic happy path and it feels bad.
- Forward constraints. We can only reference previously declared and fully constrained geometry to define new geometry.
- Tagging. Our tagging system is a bit rough round the edges and not expressive enough. I think this is orthogonal to the issues of 'feel' of the language and constraints. I've been thinking about it a fair bit and have ideas, but out of scope for this doc.
- Dimensioning constraints. Being able to constrain geometry by specifying *multiple pieces* of geometry and its length in some dimension is a powerful feature which we don't support.
- First-class points. This is related to privileging paths over other profiles and to our tagging. In theory, we could always do something like `line.start` to access a point, or define points up front and then reference them when creating a line. In conjunction with using explicit constraints, explicit constrained-ness of geometry, and improving non-path profiles, this may be enough. We do need support for working with points in the UI, but I think fixing this on the language side of things is 'just' polishing on top of fixing other issues.
- Constraints are implicit in our code (mentioned above - hard to understand, fragile to manipulate)
- Constrained-ness of geometry and components is explicit in OnShape. As well as constraints themselves being explicit, there is an explicit notion of whether geometry is intentionally constrained by the user vs a rough guess. This intention is not reified in KCL. E.g., a point at `(0.01, 0.1)` might be a fixed and absolute requirement, or it might be a rough guess to be refined by applying constraints. We have a heuristic about this based on literals vs variables, but it is implicit and non-obvious.
- Construction geometry. We don't support explicitly marking geometry for construction in KCL. IOW we don't distinguish between geometry which should be rendered or not (the engine makes some decisions about this but it's not user-controlled or very satisfactory). This is also somewhat orthogonal to the 'feel' and constraints, it is a thing I've been thinking about.
- Can only apply a single constraint to a segment.

To summarise, I think the 'hard' (which is not to say that improving the UI or adding features is not a lot of work, just that it doesn't have fundamental implications for the design of the system or KCL) issues we need to address (or plan to address) are:

- Ergonomics of non-path profiles
- Dimension constraints
- Representation of constraints in KCL (i.e., making them more explicit to the user)
- Communicating the intention of primitive geometry (points, numbers, etc) being fixed/constrained or free/variable
- Forward constraints

For construction geometry, I'll use a strawman syntax of explicit `show` keywords (if a variable is not marked `show` either when declared or later on, then it is construction geometry and not rendered, or rendered as dashed wireframe or something). I have ideas for making this better, but I'll go for the most explicit, easiest to explain thing for now. Please don't worry too much about this here, we'll develop and discuss elsewhere.


## Proposal

I'll describe features individually, and put them all together at the end.

This is an extremely early design doc, like v0.01. There's a lot still to do, but my hope is we can reach consensus on the broad feel of things, particularly to think about prioritisation, before doing the hard design and implementation work.

I use "v1" and "v2" as shorthand for 'early version', 'long-term vision'. There is a prioritisation discussion required before deciding on what goes into the 1.0 release, and I'm not proposing that the v1 here is what should be in KCL 1.0.

### Explicitly unconstrained primitives

Introduce a keyword (I prefer `var`, `free` and `roughly` were suggested on Slack, or use a sigil e.g., `~`) to mark numbers as 'guesses'. This could be extended to applying to points or other data structures, but that would require a bit more design for how it's propagated in non-trivial data types and where exactly it is allowed.

E.g., the user clicks to start a sketch, this generates code like `startSketch(at = pt(var 0.01, var 0.2))`. If the user constrains the point to the x axis, the code would be changed to `startSketch(at = pt(0, var 0.2))` and then constraining to the y axis: `startSketch(at = pt(0, 0))`. This would work too if the user or UI factors out the point, e.g., `startPoint = pt(var 0.01, var 0.2); startSketch(at = startPoint)`; constraining in the UI would change to `startPoint = pt(0, 0); startSketchAt(at = startPoint)`.

We also allow just using `var` without a number, e.g., `startPoint = pt(var, var)` for an unconstrained point without an approximate location. This is treated similarly by KCL but it can't be rendered by the UI (nor could other values derived from it).

The interpreter would track `var`-ness through execution and could inform the user if the final model included any numbers derived from `var` inputs. *Question*: I'm confident we can do this with something similar to the current system, but not sure if we can do this with a constraint solver.

This replaces `coincident` constraints.

### Syntax for sketch primitive functions

Replace the various `line` functions with a single `line` function (and similarly for `arc`, etc.). Support named and optional arguments and remove anonymous structs. See the [functions design doc](functions.md) for details.

E.g., `line(to = pt(3, 4))` (to absolute point), `line(rel = vec(0, 4))` (relative to start point), `line(angle = 90deg, len = 4)` (angle and length).

Lines may be under-constrained or over-constrained statically, e.g., `line(angle = 90deg)` or `line(to = pt(3, 4), len = 5)`. These are only checked at runtime. Over-constrained is fine as long as the constraints agree. In v1, under-constrained will be an error, in v2 external constraints will be taken into account and solved appropriately (see below).

Lines can also have a `from` argument. `a |> line(rel = vec(...))` is equivalent to `line(from = a.end, rel = vec(...))`. This is motivated by further changes below.

Constraints are expressed by changing how lines are defined by their parameters, this is something of a middle-ground between fully explicit constraints and our current system. I believe that the proposed style is easier than the current one to read and understand, and in combination with `var` is less fragile for the UI (we are only relying on the arguments to a function, rather than the function itself and the kind of arguments, i.e., literal vs variable vs expression).

### Block syntax for sketches (c.f., builder)

Builder syntax is the familiar pipeline style using `|>`. We'll probably want to change that design a bit because of this proposed change as well as some other orthogonal changes to improve it's ergonomics. We'd still support it in some form as a way to construct paths (though we should explore the alternative of building up the segments into an array and making that into a sketch in a single step).

I propose a block syntax as an alternative for making sketches. The block (using `{}`) is just a block to group statements, as is common in many PLs. The last line of the block becomes the value of the block during evaluation. We will also make local variables visible from outside by using the `export` keyword, so if we use `export p1 = ...` to define a point inside a sketch named `s`, then we can use `s.p1` from outside. I will elide `export`ing from the rest of this discussion and examples.

The main reason to do this is to keep the local variables organised in the place they're used rather than polluting the global scope. In the extensions below it's also useful to limit the search space for constraints.

Example, path to draw a square (note that these are the end results of construction, they don't reflect the process of creation, see below for that):

```
// Builder syntax
startSketch(on = XY)
  |> startPath(at = pt(0, 0))
  |> line(rel = vec(0, 1))
  |> line(rel = vec(1, 0))
  |> line(rel = vec(0, -1))
  |> enclose()

// Block syntax
sketch(on = XY) {
  a = line(from = pt(0, 0), rel = vec(0, 1))
  b = line(from = a.end, rel = vec(1, 0))
  c = line(from = b.end, rel = vec(0, -1))
  d = line(from = c.end, to = a.start)
  enclose(a, b, c, d)
}

// Also block syntax (equivalent to above, ideally, we should be able to get from one to the other using refactoring tools)
sketch(on = XY) {
  a = pt(0, 0)
  b = pt(0, 1)
  c = pt(1, 1)
  d = pt(1, 0)
  l1 = line(from = a, to = b)
  l2 = line(from = b, to = c)
  l3 = line(from = c, to = d)
  l4 = line(from = d, to = a)
  enclose(l1, l2, l3, l4)
}
```

Here's a non-path example, adapted from Josh's notes to draw a faceplate (sketch003).

```
// Josh's code
sketch003 = startSketchOn('XY')
    circle001 = circle()
    circle002 = circle()
    circle003 = circle()

    makeConstruction(circle003)

    contraint.coincident(circle001.center, origin)
    contraint.coincident(circle002.center, origin)
    contraint.coincident(circle003.center, origin)
    
    dimension.diameter(circle001) = 2
    dimension.diameter(circle002) = 6
    dimension.diameter(circle003) = 4.5

    // mounting holes
    circle004 = circle()

    constraint.coincident(circle004.center, circle003.edge) // coincident constraint that lives on the edge of the circle
    constraint.vertical([circle004.center, origin])

    dimension.diameter(circle004) = 0.75

    patternCircular({
        entities: [circle004],
        center: circle001.center,
        instances: 4,
        angle: 360
    })

// Proposed block syntax
sketch(on = XY) {
  center = pt(0, 0)

  // Using `-` for removing one shape from another, could also be a `remove` function or whatever.
  // Note `center` is shorthand for `center = center`.
  ring = circle(center, diameter = 6) - circle(center, diameter = 2)

  // Hand-wavey code for compositional patterns
  ring |> p in pattern(path = circle(center, diameter = 4.5), instances = 4) {
    ring - circle(center = p, diameter = 0.75)
  }
}
```

Big *question*: how this is communicated to the engine, especially once we support constraints as described below.

#### Bidrectional/complex constraints

In v2, we extend the block syntax to allow adding constraints in the block. We'll design the exact syntax etc. later. We might want to support some constraints in v1 but they would have to be unidirectional constraints (e.g., `len(line1) <= len(line2)` is unidirectional since if we adjust `line2` then `line1` will follow, but if we adjust `line1` then it changes the constraint rather than changing `line2`, `len(line1) == len(line2)` is bidirectional, you can adjust either line and affects the other) and only depend on previously declared geometry.

We could also support hints for solving and `maximise`/`minimise` constraints as Jon suggests in v2.

Example, to define the diamond in sketch005 of Josh's doc using constraints:

```
sketch(on = plane001) {
  a = pt(0, 0)

  // Strawman multiple var decl syntax
  var b, c, d

  l0 = line(from = a, to = b)
  l1 = line(from = b, to = c)
  l2 = line(from = c, to = d)
  l3 = line(from = d, to = a)

  equal(len(l0), len(l1), len(l2), len(l3))
  vert = line(from = a, to = c)
  horz = line(from = b, to = d)
  parallel(vert, Y)
  parallel(horz, X)
  yDim(vert) == 6
  xDim(horz) == 15

  enclose(l1, l2, l3, l4)
}
```

We might want some shorthand for applying the line constraints to pairs of points and obviously there's a bunch of questions around naming and organising constraints

In v1 we could also allow constraints such as `yDim(vert) == 6` as checks (i.e., they cannot be used to compute the positions of geometry but they will be used to assert that the result is correct).

#### Ordering of constraints and reordering of code

In v1 we would require ordering of constraints by syntactic order (i.e., can only use variables which have already been defined), and any geometry must be fully constrained where it is declared. However, to mitigate the ordering issue, geometry in block syntax does not need to be path-ordered. For example, the square example above could be refactored to:

```
sketch(on = XY) {
  a = pt(0, 0)
  b = pt(0, 1)

  // Vertical lines
  v1 = line(from: a, to: b)
  v2 = line(from: a, angle: parallel(v1), len: len(v1))

  // Horizontal lines
  h1 = line(from: a, angle: perpendicular(v1), len: len(v1))
  h2 = line(from: b, angle: parallel(h1), len: len(h1))

  enclose(v1, v2, h1, h2)
}
```

This makes our current constraints more flexible and relieves some of the pressure for more powerful constraints. Furthermore, we could support automatic reordering (including checking that we don't move uses of variables before their declarations) as part of constraint application.

### Example and walk-through

Imagined walk-through for Sketch001 from Josh's doc (an open path).

- User enters sketch mode, uses the line and arc tools to make a 3-segment path

```
startSketch(on = YZ)
  |> start(at = pt(var 0.2, var 0.08))
  |> line(to = pt(var 8.9, var -0.1))
  // Note `var tangent()`: `var` applied to expression/function call, no argument to `tangent` is equivalent to using `%` today.
  |> arc(radius = var 2.5, startAngle = var tangent(), sweep = 90deg)
  |> line(to = pt(var 12.1, var 5.5))
```

- User selects start of the sketch and constrains it to the origin
- User selects first line and makes it parallel to the X-axis
- User selects last line and makes it parallel to the Y-axis

```
startSketch(on = YZ)
  |> start(at = pt(0, 0)
  |> line(angle = parallel(X), len = var 8.7)
  |> arc(radius = var 2.5, startAngle = var tangent(), sweep = var 87deg)
  |> line(angle = parallel(Y), len = var 2.6)
```

- User selects the arc and makes the radius 3, confirms the start angle, and set the end angle to be a tangent to the following line.

```
// v1
startSketch(on = YZ)
  |> start(at = pt(0, 0))
  |> line(angle = parallel(X), len = var 8.7)
  // Note that the sweep is computed by the UI
  |> arc(radius = 3, startAngle = tangent(), sweep = 90deg) as arc001
  |> line(angle = parallel(Y), len = var 2.6) as line001

assert(arc001.endAngle == angle(line001))

// v1 alternative (probably not supported by UI, but possible by hand)
sketch(on = YZ) {
  line000 = line(from = pt(0, 0), angle = parallel(X), len = var 8.7)
  line001 = line(from = pt(var 8.7, 3), angle = parallel(Y), len = var 2.6)
  arc001 = arc(from = line000.end, radius = 3, startAngle = tangent(line000), endAngle = tangent(line001))
  openPath(line000, line001, arc001)
}

// v2 (note automatic transformation from builder syntax to block syntax)
sketch(on = YZ) {
  line000 = line(from = pt(0, 0, angle = parallel(X), len = var 8.7)
  arc001 = arc(from = line000.end, radius = 3, startAngle = tangent(line000), endAngle = tangent(line001))
  line001 = line(from = arc001.end, angle = parallel(Y), len = var 2.6)
  openPath(line000, line001, arc001)
}
```

- User specifies the width of the whole sketch as 12 and the height as 6 (I haven't described this in the doc, I'm imagining this is a pure UI feature in v1 and integrated into KCL and the engine in v2)

```
// v1
// Note that the lengths are computed by the UI, possible because they are `var`, but the properties of the arc are not. Not sure if this is possible in many cases?
startSketch(on = YZ)
  |> start(at = pt(0, 0))
  |> line(angle = parallel(X), len = 9)
  |> arc(radius = 3, startAngle = tangent(), sweep = 90deg) as arc001
  |> line(angle = parallel(Y), len = 3) as line001

assert(arc001.endAngle == angle(line001))
assert(xDim(sketch001) == 12)
assert(yDim(sketch001) == 6)

// v2
sketch(on = YZ) {
  line000 = line(from = pt(0, 0), angle = parallel(X))
  arc001 = arc(from = line000.end, radius = 3, startAngle = tangent(line000), endAngle = tangent(line001))
  line001 = line(from = arc001.end, angle = parallel(Y))

  // Strawman syntax - no params means use the whole sketch
  xDim() == 12
  yDim() == 6

  openPath(line000, line001, arc001)
}
```

- UI communicates that the sketch is fully constrained because there is no `var`-ness in the final result and all geometry is fully specified
- The sketch could be assigned into a variable and then used as a path to sweep/extrude along by the `extrude` function.

## Rationale: a few 'big' points about KCL

### Code is first-class

The UI and KCL are alternate views of a project. A core principle of KittyCAD and a key differentiator from competitors (AIUI) is that the code view is first-class, not just a file format. For this to be true, I believe the following must hold

- the UI feels like a good UI, not a wrapper around a language, which sacrifices usability for the language
- KCL feels like a good language, not a wrapper around a GUI, which sacrifices usability for the GUI
- the two must work seamlessly together

For the last point, I think this is a high-level goal, not a feature requirement. For example, currently, changes in the UI are reflected immediately in the code, which is nice but I think this is just one way (neither necessary nor fully sufficient) to partly satisfy the seamless integration requirement.

Furthermore, I think that it should be possible to define a model:

- purely using code (typing + IDE features, only using the rendering in the UI for visualisation/debugging)
- purely using the UI
- using a hybrid of code and UI

The end result by each method should be roughly the same, perhaps after *a little* refactoring (this is important! If that is not true, then one of the methods is likely sub-optimal with respect to the others), however, the *process* for creating the model might be very different.

### Programming, not scripting

KCL is a programming language, not a scripting language. I don't mean scripting language in the sense of 'easy to learn, dynamically typed'. I mean that it is a program that describes the model, not a script describing the user creating the model in the UI. While all the information about a model should be reflected in the code, that doesn't mean that every step to get there should be reflected in the code. Furthermore, we should lean on programming techniques (control flow, functions, etc) to make the code a nice representation of the model, rather than relying on copy and paste and trying to be close to the user's actions.

### A new paradigm

We *want* KittyCAD to be a paradigm shift for mechanical engineers. We want to bring the benefits of software engineering to mechanical engineering. AIUI, that's a key principle of our work. Without arrogantly assuming that what exists already is bad or that we just know better, we should be confident that we can make things better and that customers will want that.

There will be a learning curve for users, and we should do everything we can to make it easier for them (design of the language and UI, docs and other onboarding, etc, etc) and as much as we can we should go to our users, rather than forcing them to come to us. BUT we do want them to learn some new things and do things in a better way then they are used to.


## Rationale: my goal

Design a language which can eventually support all of the features from the 'what's missing' section, including requiring a constraint solver. Design a subset of that language (without a constraint solver) which addresses the biggest issues, provides a good user experience, and is relatively incremental from today's KCL. We should be able to get from the second to the first without any serious breaking changes. The languages should be close enough that feedback from and polishing of v1 is useful for v2, i.e., when we implement v2 we are not starting from scratch in terms of understanding and satisfying our users.
