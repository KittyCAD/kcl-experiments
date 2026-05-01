# Sketch mode 2.0

Examples:

```kcl
start = [0, 0]
mySquare = sketch(on = XY) {
  // A path made of four lines
  line0 = line(start)
  line1 = line(start = line0.end)
  line2 = line(start = line1.end)
  line3 = line(start = line2.end, end = start)
  
  // All edges have equal length
  equalLength(line0, line1, line2, line3)

  // Corners are right-angles
  perpendicular(line0, line1),
  perpendicular(line1, line2),
  perpendicular(line2, line3),
  perpendicular(line3, line0),

  // Constrain the diagonal distance of the square to a fixed value.
  distance(start, line1.end) == 10mm
}

splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0, var 0], radius = var 5)
  circle1 = circle(center = [var 0, var 0], radius = var 2)

  line0 = line(start = [var 1, var -0], end = [var 1, var -10])
  line1 = line(start = [var -1, var -0], end = [var -1, var -10])

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  parallel(line0, line1, distance = 2)
  symmetric(line0, line1, axis = Y)

  return region(circle1, line0, circle0, line1)
}
```

See the end of the doc for a step-by-step example of creating the split washer.

Deferred work:

- more geometry: circle, rectangle, bezier, ... (should be fairly straightforward following the design philosophy in this doc)
- helper functions (rectangle)
- more editing functions: chamfer, fillet, linear/circular patterns, break/split, transformation functions


## Implementation design

TODO diagrams

See also https://gist.github.com/nrc/226333e93a2153b2d381344d2ef2591c

### Components

The *frontend* owns the UI (presumably providing multiple ways to specify constraints, etc. including editing the source code). The *interpreter* is responsible for executing the KCL code. Since the interpreter has knowledge of the semantics of the code, it is also responsible for creating and modifying code. The interpreter has a *solver* module which is specifically responsible for turning constraints into concrete values. The solver may include third-party code for the deep numeric work or may be all our code, but at the least will have an abstracting wrapper around the third-party code which we maintain. The interpreter sends commands to the *engine* which is responsible for rendering.

### Data flow

- User interacts with the UI - adding segments, dragging points, setting values, etc.
- Changes are sent to the interpreter in terms of an abstract model of the sketch
- The interpreter applies these changes to its representation of the sketch which is used to generate code for the sketch.
  - Code changes are returned to the frontend to update the code editor.
  - The interpreter uses the solver to find concrete values for the sketch and/or information about under-/over-constrained variables. The interpreter returns this information to the frontend to render the sketch and give feedback to the user.


## Sketch blocks

```
expr ::= ... | sketch(kwargs) { stmt* }
```

May be used like any other KCL expression, e.g., used as a stand-alone statement, assigned into a variable, etc. The kwargs include `on` and `face` for the plane or face to sketch on, and `tag` to tag the sketch. We can add more to match `startSketchOn` and `startProfile` if necessary.

Every sketch block must contain exactly one `return` statement. The returned value must be either a `Region` or `Path`. A return statement is extended to allow naming it within the sketch block. The type of the sketch block is the type of the returned expression. Either type may be coerced to `Sketch`.

Question: in the examples above, I use `return` in any position since order within a sketch block is mostly insignificant. However, `return` has a a very imperative vibe, so this might be confusing to readers. We could require `return` to be the last statement in the sketch block, or we could use a different keyword, e.g., `export` or something new. We could also use some non-keyword syntax such as a significant assignment (e.g., `@sketch = ...`) or significant position (e.g., the last line of the sketch block is the result).

```
stmt ... | `return` (id `=`)? expr 
```

The statements/expressions allowed within is limited:

* May not contain function or type declarations, nested sketch blocks, `if` constructs.
* May contain field/array access (access of fields/), function calls (to a limited set of functions, see below), arithmetic (`+`, `-`, `/`, `*`).
* Values may be assigned into variables, and those variables used. The order of assignment is not important, e.g., `z = 1 + x; x = 3` is valid and not an error. However, assignment will not use the solver and can only be used for straightforward substitution, e.g., `z = 1 + x; x = z * 2` is an error.
* Equivalence is denoted with `==` and introduces a constraint between the left and right hand sides. E.g., `z == 1 + x; x == z * 2` is not an error but is under-constrained and will produce different values for `x` and `z` (and an error), depending on their initial values. Note that variables cannot be used without being initiated, so `z == 1 + x; x == z * 2` will produce uninitialized variable errors, `z = var 0; z == 1 + x; x == z * 2; x = var -0.9` would be fine.


## `var`

Use the `var` keyword to indicate a numeric value that can be changed by the constraint solver (changed here means will have a different output value to the specified value, the KCL is not directly modified by the solver, though the frontend might do so).

```
expr ::= `var` num_lit
```

In the future we might permit `var` without a numeric literal, but it's useful for the solver to have a good starting point for numeric solution. For now, the frontend can just use `var 0` if there is no reasonable value to use.

E.g., `var 4` or `var 5mm` to indicate a variable numeric value, `[var 0, var 0]` to indicate a variable 2d point.

Question: the syntax is a bit non-obvious, we might prefer `var<3.21>` or `var(3.21)`, or a sigil (e.g., `~3.21`, `~-1`), or something else?


## Geometric functions and objects

Two-dimensional points are just as currently defined: a two-element array of `number(Length)`.

The following new functions are added to the standard library and are annotated with `@(impl = std_rust_fn_constrainable)`. They are added to a new module (perhaps named `constraints`) so as not to clash with existing functions. These are included as a prelude within `sketch` blocks, but not outside. Functions with `@(impl = std_rust_fn_constrainable)` cannot be used outside sketch blocks. In the future, user functions may be supported to allow arbitrary shapes to be used in sketch blocks.

```kcl
fn line(
  start: Point2d = [var, var],
  end?: Point2d = [var, var],
  midpoint?: Point2d = [var, var],
): Line

fn arc(
  start: Point2d = [var, var],
  end: Point2d = [var, var],
  center?: Point2d = [var, var],
  interior?: Point2d = [var, var],
): Arc

// also rectangle, circle, bezier, etc.
```

TODO regions should probably happen outside the sketch block

```
// TODO options for winding direction and handling of enclaves?
fn region(@input: [Segment | Path; 1+]): Region
```

The `region` function creates a 2d region from an array of segments (which may include paths, which are effectively flattened and treated as a series of segments). As well as creating a region object which can be manipulated and returned from the sketch block, the `region` function constrains each segment to intersect the next, and the last segment to intersect the first (to avoid this last behaviour and create an open profile as a region, create a region from a single unclosed path, alternatively we might take a `closed` or `open` optional argument). The intersecting points can be accessed via the `intersections` field on the returned `Region` object, e.g., `region(...).intersections[3]`.

Any argument to a function can be referred to as a field in the returned type, whether it is specified or not, e.g., `line().start`. The segments passed to `path` or `region` can be referred to by indexing, e.g., `path(...)[1]`.

### Types

`Line` and `Arc` are subtypes of `Segment`.

These types are defined in the standard library. `Line` and `Arc` are annotated with `@(impl = std_rust_constrainable)` and are understood by the interpreter as input to the solver. The attribute also permits use of `var` numbers. The current `Point2d` definition will also have this attribute.

In the future, user types may be similarly annotated to permit user-defined input to the solver.

### Boolean operations on regions

The following functions operate on regions (these don't introduce constraints and are available outside of sketch blocks):

```
fn intersect(@input: [Region; 2+]): Region
fn union(@input: [Region; 2+]): Region
fn subtract(@input: Region, tools: [Region]): Region
```

We also support `&` , `|` or `+`, `-` as sugar for `intersect`, `union`, and `subtract` respectively.


### Pipeline syntax (paths)

*Postponed feature*. The motivation here is to make converting existing pipelines easier to convert to sketch blocks and to make the case of drawing a path more ergonomic. Not required for initial implementation though.

Within sketch blocks, a variant of the pipeline syntax is supported, where each element has type `Segment`, or is the last element and is the function `close` (we won't special case `close` so much, but I'm not sure exactly how to define the restriction). The pipeline syntax is sugar for calling the `path` function. Calling `close` is equivalent to using `closed = true` when calling `path`. The syntax provides 'soft backwards compatibility' with current sketch syntax, i.e., allows easier transformation of code from the old to new syntax.


## Constraint functions and objects

Constraint functions are defined in the standard library and declared using a special attribute such as `@(impl = std_rust_constraint)`. Initially, the interpreter will special-case such functions, eventually we might be able to support user-defined constraint functions.

Constraint functions are mostly used within equivalence statements (or inequalities in the future), so appear as if they return a value. E.g., `distance(p1 = p[0].start, p1 = p[1].end) == 10mm` (or in reverse order). However, constraint function usages are desugared into calls to functions with the other side of the equivalence (`10mm` in the example) as their input argument. In their function declarations, the functions return a `Constraint` object which is used internally.

Constraint functions:

```kcl
fn coincident(@input: [Point2d | Segment; 2+])

fn concentric(@input: [Arc | Circle; 2+], distance?: number(Length))
fn parallel(@input: [Line; 2+], distance?: number(Length))
fn colinear(@input: [Line; 2+])
fn perpendicular(@input: [Line; 2])
fn horizontal(@input: Line | [Point2d; 2])
fn vertical(@input: Line | [Point2d; 2])

/// Constrains the angle between two lines.
fn angle(@input: [Line; 2]): number(Angle)

/// Constrains the straight-line distance between two points.
fn distance(@input: [Point2d; 2]): number(Length)

/// Constrains the x or y distance between two points.
fn horizontalDistance(@input: [Point2d; 2]): number(Length)
fn verticalDistance(@input: [Point2d; 2]): number(Length)

/// Constrains the perpendicular distance between a line or arc and a point.
fn perpendicularDistance(@input: Segment, point: Point2d): number(Length)

// Note that `axis` must be a fixed line, not a constrained line, probably need a better type.
fn symmetric(@input: [Point2d | Segment; 2], axis: Line)

/// Constrains two segments to intersect at a point.
fn intersects(@input: [Segment; 2]): Point2d

/// Constrains two segments to touch at a point.
fn tangent(@input: [Segment; 2]): Point2d

/// Trims or extends `input` so that it touches `to`. If `input` has ends (e.g., it's a line or arc) then `from` is
/// required and indicates the end of the segment to be trimmed or extended.
/// Returns a segment which is either added or removed from `input`. `input` is approprately constrained by this function
/// so the returned segment is likely just construction geometry and can be ignored in the program code.
fn trimOrExtend(@input: Segment, from?: Point2d, to: Segment): Segment

/// Constraints a point to lie at the midpoint of a segment.
/// TODO it would be easy enough to generalise this to any proportion of a segment
fn midpoint(@input: Line | Arc, point: Point2d)

fn length(@input: Line): number(Length)
fn horizontalLength(@input: Line): number(Length)
fn verticalLength(@input: Line): number(Length)
fn radius(@input: Arc | Circle): number(Length)
fn diameter(@input: Arc | Circle): number(Length)
fn arcLength(@input: Arc): number(Length)
fn arcAngle(@input: Arc): number(Length)
fn circumference(@input: Circle): number(Length)
```

Where the input argument is an array, the function may be called with the array 'spread' as arguments, e.g., `coincident(p1, p2, p3, p4)`.

Note that `Segment` is a super-type of `Line` and `Arc`, but also of non-segment lines including the axes.

Discussion on API design: I believe there are three axes along which the API can be adjusted:

- The degree of overloading, i.e., using many functions with specific arguments, vs using few functions with more optional arguments.
- The number of shorthand functions, e.g., we could have a `parallel` function which is equivalent to `intersectsAngle(...) == 0`.
- Whether we constrain absolute or relative values or both, e.g., we might have a system which only allows specifying the angle between a line and the y-axis, and to constrain the angle between two lines, we use a constraint of the form `a - b == c`.


## Constraint semantics

All expressions in a sketch block are lowered to constraints (possibly with side effects, e.g., `region` creates a region object) which are then passed to the solver. These might include the constraint functions (including implicit calls from geometric functions), arithmetic operations, equivalences (`==`), and equivalences from the arguments of geometry functions (e.g., `l1 == line(start = [0, 0]); line(end = l1.start)` is equivalent to `p1 == [0, 0]; p2 == [var, var]; l1 == line(); l2 == line(); l1.start == p1; l1.end == p2; l2.start == p2`).

The solver will make a best effort to solve all the constraints and return a data structure to the frontend with resolved points for rendering, etc. It will also produce errors for conflicting constraints and under-constrained sketches, and information about over-constrained sketches and the final values of `var` values (the frontend could replace these with user consent).


## Display semantics

All lines and arc, etc. are construction geometry by default, i.e., not displayed or exported. The only way for a segment to be displayed/exported is to be used (directly or indirectly) in the region returned from the sketch block. Note that the region returned can be an unclosed path (an open profile) which can be used to extrude a surface, etc. Note that from a UI user's perspective, the default is that all segments are displayed and are not construction geometry. This is because using the point and click interface would generate calls to include all segments in a region.


## Tags and tagging

The design as specified has no concept of tags or tagging. This section specifies how other features replace tagging in new code and compatibility with old code.

All geometry within a sketch block is identifiable using variable naming and field access (all arguments in segment constructor functions are available as fields on the segment types). Since we don't use pipeline syntax, all declared geometry is ergonomically nameable using variables. Rather than taking tags as arguments, constraint functions take typed arguments. E.g., `angledLineThatIntersects` (which currently exists) takes an argument `intersectTag: TaggedEdge` to identify the edge which the newly-created line intersects with, whereas the new `intersects` constraint takes the intersecting segments as its unnamed input argument with type `Segment`.

Some current KCL functions take tag declarator arguments to identify multiple faces created by the function, e.g., `extrude` can create tags for `tagStart` and `tagEnd`. No such functions exist in the current spec, but if they did, then rather than take tag declarators, they would return an object where the faces are available as fields.

### From outside sketch blocks

Variables declared within a sketch block are available as fields on the object created by the sketch block. E.g.,

```kcl
foo = sketch(on = XY) {
  a = line()
  // ...
}

extrude(foo)
  |> fillet(radius = 1mm, tags = [foo.a])
```

Note the use of `foo.a` to access the variable `a` within the sketch block.

Where a name already exists on the object, variables with that name are hidden. E.g., if sketch objects have a field `appearance` then that would take priority over a variable named `appearance`.

For similarity with existing code and to allow access to variables hidden by other names, all variables are available via the `tags` field. E.g., in the previous example, `foo.tags.a` could also be used to access `a`.

`Segment` can be coerced to an `EdgeTag` allowing usage wherever a tagged edge is currently allowed. If extruded (or otherwise extended into 3D), then `Segment`s can be treated as `FaceTag`s (in the same way as `EdgeTag`s are today).

For any variables of a sketch block accessed from outside, any accessible component values are the fully resolved, concrete values resulting from constraint resolution, not the pre-resolution values. This means any `var`-ness has been erased and is not observable outside of a sketch block. E.g.,

```kcl
foo = sketch(on = XY) {
  a = var 1
  a == 2
}

// foo.a is 2, not 1
assert(foo.a, isEqualTo = 2)

bar = sketch(on = XY) {
  l = line(from = [var 1, var 1])
  coincident(l.from, [2, 0])
}

assert(bar.l.from[0], isEqualTo = 2)
```

### Alternative and rationale: exporting variables

An alternative would be to only allow variables to be accessed from outside the block if the programmer opts-in, e.g., by using `export`. I prefer not to do this. I believe the syntactic overhead is not worth the benefit. As I see it, the benefit is being intentional about which values are 'intermediate results', and which are 'final results' which should be externally observable. However, in contrast to functions and modules, I don't think encapsulation is a primary motivator for sketch blocks. The fact that variables are accessed using dot syntax rather than directly (i.e., introducing them into the surrounding scope) is enough to indicate the intermediateness of variables. Furthermore, sketch block (like regular blocks and in contrast to functions and modules) are lexically close to where they are evaluated so it is less likely that they need to be encapsulating (and if a sketch block is to be used outside of its lexical scope, it is protected by the usual KCL scoping and privacy rules).


## Backwards compatibility

Surprisingly, I think we can do all of this proposal without any (significant) breaking changes. The big item is that `sketch` becomes a contextual keyword, i.e., when used as `sketch(...) { ... }`, it is treated as a keyword. I think this is a breaking change, since it can currently be used as the name of a function, called and followed by a block (I'm not sure if we currently allow blocks without a function declaration or `if`, etc.). `sketch` is likely to be used as an identifier, however, the usage with a block is unlikely to be common, so I think this is a breaking change which will not have much impact (it may not be breaking at all).

All other parts of the proposal are used within a sketch block, which don't currently exist, so are not breaking. To make the function names work without breaking changes will require some slightly weird rules for importing prelude functions, but I think that is worth the cost to avoid breakage.

Not a back-compat issue, but more about consistency: in the `line` function , `end` is equivalent to the current `endAbsolute`, and the current `end` (as well as the `xLine` and `yLine` functions) are specified using `xLength` and `yLength`. Although this is a bit inconsistent, it is (I think) the clearer option. I'm not confident about this though.


## Step-by-step example

The comments indicate the user action in the point and click UI.

```kcl
// User initiates a sketch
// Note that as currently specified this is an error since there is no return statement. We could allow sketch blocks without returns to have void type so this is not an error.
splitWasher = sketch(on = YZ) {
}

// User draws a circle.
splitWasher = sketch(on = YZ) {
  // Note that the UI should indicate that the circle is under-constrained.
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  return circle0
}

// User draws a second circle, since it is entirely within the first circle, the frontend assumes it should be subtracted
splitWasher = sketch(on = YZ) {
  // Note that the UI should indicate that the circles are under-constrained.
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)
  return circle0 - circle1
}

// The user locks the centers of the circles to be concentric and at the origin and specifies the radii.
splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  return circle0 - circle1
}

// The user draws two lines. Since there is no obvious link to the existing sketch, the frontend treats these as construction geometry.
// The user uses grid snapping which gives tidy values, but still `var` values.
splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  return circle0 - circle1

  // Note that the UI should indicate that the end points of the line are under-constrained.
  line0 = line(start = [var 1, var -0], end = [var 1, var -10])
  line1 = line(start = [var -1, var -0], end = [var -1, var -10])
}

// The user uses the region tool, selecting the lines and circles to create the split washer shape.
splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  // Note that the UI should indicate that the end points of the line are under-constrained.
  line0 = line(start = [var 1, var -0], end = [var 1, var -10])
  line1 = line(start = [var -1, var -0], end = [var -1, var -10])

  return region(circle1, line0, circle0, line1)
}

// The user adds a parallel constraint between the two lines.
splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  // Note that the UI should indicate that the end points of the line are under-constrained.
  line0 = line(start = [var 1, var -0], end = [var 1, var -10])
  line1 = line(start = [var -1, var -0], end = [var -1, var -10])

  parallel(line0, line1)

  return region(circle1, line0, circle0, line1)
}

// The user sets the distance between the lines.
splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  // Note that the UI should indicate that the end points of the line are under-constrained.
  line0 = line(start = [var 1, var -0], end = [var 1, var -10])
  line1 = line(start = [var -1, var -0], end = [var -1, var -10])

  parallel(line0, line1, distance = 2)

  return region(circle1, line0, circle0, line1)
}

// The user adds a symmetric constraint on the two lines with the y-axis as the line of symmetry.
splitWasher = sketch(on = YZ) {
  circle0 = circle(center = [var 0.1, var -0.3], radius = var 5.0)
  circle1 = circle(center = [var -0.1, var -0.2], radius = var 2.1)

  coincident(circle0.center, [0, 0])
  concentric(circle0, circle1)
  radius(circle0, 5)
  radius(circle1, 2)

  line0 = line(start = [var 1, var -0], end = [var 1, var -10])
  line1 = line(start = [var -1, var -0], end = [var -1, var -10])

  // Note that the UI should indicate that the following constraints cause the sketch to be over-constrained, but not conflicted.
  parallel(line0, line1, distance = 2)
  symmetric(line0, line1, axis = Y)

  return region(circle1, line0, circle0, line1)
}
```
