# Functions

This document proposes several changes to how function work in KCL. It supports a lot of the ergonomic changes proposed elsewhere. Although it changes the implementation a lot, I believe the 'feel' of KCL functions will not change too much, other than removing the many flavours of similar function in std (e.g., `line`, `lineTo`, `angledLine`, `xLine`, etc are all replaced by the single `line`).

There is a lot in this design doc and it might feel like too much complexity for a small, domain-specific language. However, most of this complexity exists to facilitate very simple and clear usage patterns, and the detail in the doc is to ensure it will all work as desired. Bear in mind that this doc is aimed at implementers, not users, and documentation for users would approach these topics in a very different way. I'm pretty sure that the complexity here fits well with the desired learning curve: beginners will not have to concern themselves with much of this, more advanced users can do more as they learn more. Skip to the end to see an elaborated example of how this all fits together and how it would feel to a user of KCL.

A small syntactic change to function declarations was proposed in the [syntax design doc](syntax.md) - removing `=` and `=>` from the declaration. Adding type declarations to functions was proposed in the [types design doc](types.md). This doc assumes both of those changes.

I talk a fair bit about types in this doc, which may be surprising! I'm still thinking about types in KCL, there's a lot still to work out. I'm currently thinking that in the short-term, typing is dynamic (but strong) and the only required type annotations are on function declarations. I believe this will give us a good way forward to either keep dynamic type checking or to move to static checking. If we want to avoid requiring type annotations on functions, I believe we can do that, though I think we should make it optional and use them in the standard library (if for nothing more than documentation). In that case some of this design would need adjusting, likewise adjustments will be needed depending on prioritisation of changes for 1.0.

## Returned value and type

Functions may or may not have a return type (e.g.,  `fn foo(...) { ... }` or `fn foo(...): Type { ... }`). If they have a return type, then the function body must end with a return statement (`return expr`). If there is no return type then the function must end with either a return statement with no expression (`return`) or have no return statement. Early return is permitted in a similar way.

For consistency, the return type of functions with no explicit return type is `void`, but this should not be user-facing.

## Arguments

Note on terminology, I use 'argument' and 'parameter' mostly interchangeably. Where necessary I use 'formal' and 'actual' to differentiate between the argument in the function declaration, and the value passed to a function when called.

A function may have a single unnamed argument, called `self`, which must be declared first (and may not be optional)[^self]. A function may have any number of named arguments, any of which may be optional. An optional argument may have a default value. Syntax:

```
fn-decl ::= 'fn' id '(' ('self': ty)?, arg* ')' (: ty)? block
block ::= '{' stmt* ('return' expr?)? '}'
arg ::= id '?'? ':' ty ('=' value)?
```

(the definition of `value` for default arguments is left open for now and may get more flexible over time. Using literals should be fine, we could extend this all the way to any expression).

E.g.,

- `foo()` called as `foo()`
- `foo(self: num)` called as `foo(42)`
- `foo(a: num, b: num)` called as `foo(a = 0, b = 42)`
- `foo(a: num, b?: num)` called as `foo(a = 0)` or `foo(a = 0, b = 42)` or `foo(a = 0, b = None)`

For optional arguments with no default, the type of the variable within the function is the option type of the declared type. For optional arguments with a default, the type is the declared type. E.g., in `foo(a?: num = 0, b?: num)`, `a` has type `num`, `b` has type `num?`. `b` must be unwrapped, there is no way to tell if `a` is a supplied argument or is missing and using the default.

An optional argument may not have an optional type, i.e., `a?: T?` is illegal for all `T`. If `None` is passed for an optional argument, that is equivalent to not specifying the argument, e.g., `foo(a = 0)` and `foo(a = 0, b = None)` are equivalent.

Note that if a non-optional argument has optional type, then it must be supplied (though could of course be `None`) and cannot be elided (c.f., optional arguments).

If `self` has array type and the function is called with multiple (unlabelled) values with a single super-type, they will be packed into an array.

E.g., `foo(self: [num])` can be called as `foo([])` or `foo([0, 1, 2])` or `foo(0, 1, 2)` but not `foo()` or `foo(0, 1, "hello")`.

If an actual argument is a simple variable whose name matches the formal argument name, then the argument name can be elided, e.g., `fn foo(a: num, b?: num)` may be called with `foo(a, b)` if `a` and `b` are local variables. Note that if the function has a `self` parameter, then it takes priority, e.g., for the function `fn foo(self: num, a? = num)`, `fn foo(a)` is interpreted as `a` being `self` (types are ignored for this decision). This likely makes this feature interact poorly with array `self` arguments, and we should probably just not check for matching argument names in that case).

[^self]: Alternatively, we could let the user choose the name of `self`. This might be better because although the `self` parameter is used in KCL for dispatch as in OO languages, we do not have class/impl declarations which include methods, and our `self` parameter is used for much more than just OO dispatch. It might better be called the magic parameter, as the only non-named parameter it is treated specially in many ways.

### Pipeline operator

See also [Pipelines](pipelines.md)

The pipeline operator is `|>`, the syntax is `expr '|>' pipe-expr ('as' id)?`, where `pipe-expr ::= fn-call` but may be expanded in the future.

If the function call on the rhs of `|>` has a `self` argument, then the function call may elide the `self` argument and the result of the lhs is used as the `self` argument[^self2]. Furthermore, this applies to any functions in sub-expressions. If a function is used in a pipeline and it has no actual `self` argument, then the lhs of the pipeline is not used.

If there are versions of a function in scope both with and without formal `self` arguments, then when calling a function without an explicit `self` argument within a pipeline, at the top level the version with `self` takes priority, and in sub-expressions, the version without `self` takes priority[^yikes].

E.g., for `fn foo(self: num)`, `42 |> foo()` is equivalent to `foo(42)`. `42 |> foo(0)` is not allowed.

E.g., for `fn foo()`, `42 |> foo()` is equivalent to `foo()`, `42` is ignored (in this example, it should trigger a warning, more generally the lhs may be used elsewhere).

E.g., for `fn bar(self: num, a: num)` and `fn baz(self: num): num`, `42 |> bar(a = baz())` is equivalent to `bar(42, a = baz(42))`.

E.g., more realistically (see below for function definitions and a full example):

```
startSketch(on = XY)
  |> startPath(at = pt(0, 0))
  ...
  |> line(to = start())
```

- `startSketch` has no `self`, it's result becomes the `self` for `startPath`, the result of `startPath` is used for the next pipeline stage, and so forth.
- For the final `line` call, the previous pipeline result is used as `self` for both the call to `line` and `start`.

In the [Pipelines](pipelines.md) doc I proposed using `as` to name sub-stages of a pipeline. To see how that works, lets look at `... |> line(to = start()) as foo`. This labels the result of the `line` call as `foo`. Note tthis line call is dispatched to the 'pipeline object' which is presumably a builder object for the sketch or path (see below for a strawman version of how this could work). That version of `line` returns a new builder, not the constructed line, therefore `foo` refers to the intermediate sketch, not the line itself. This is good: sometimes you want the intermediate sketch and sometimes you want the line itself, but it is much easier to go from the sketch to the line (`lastSegment(foo)` or similar) than from the line to the sketch!

Furthermore, we can make things even more ergonomic for the user by having a convention that functions which would typically refer to a path segment can also be applied to a path or sketch (by taking advantage of overloading using `self`). For example, `start` might return the start point of a line or arc, and on a path returns the start point of the first segment. Likewise for `end` and the end point of the last segment. `tangent` might refer to the end angle of the last segment of the path, and so forth.

[^self2]: This may remind you of method call syntax in other languages using `.` or `->`. It sort-of is. It also follows `|>` syntax from F#. Note that we use `.` in KCL for field access, including accessing functions if functions are stored in a field (in particular this happens with imported [modules](modules.md)). Note though, that using `.` just locates the function, it does not treat the lhs as the receiver/`self` argument.

[^yikes]: Yikes! This seems a bit subtle and error-prone. However, it does mean that the following examples work: `path |> line(angle = perpendicular(), len = segmentLen())` (`perpendicular` and `segmentLen` are called with `self = path`) and `path |> mirror(line(from = (0, 0), to = end()))` (the non-self version of `line` is used, `path` is passed as `self` to `end`). To get other behaviour, the user can always break the pipeline and use a variable.

### Block syntax

If a function has a `self` argument, then it can be called using a block syntax: `fn-call ::= ... | id '(' namedArg* ')' '{' stmt* '}'`. This is syntax sugar for having the block in `self` position, e.g., `foo(x = 42) { ... }` is equivalent to `foo({ ... }, x = 42)`. Note that the value of the block is assumed to be the expression used on the last line of the block.

The motivation for this is as an alternate syntax for creating sketches which does not emphasise ordering and which better supports constraints and construction geometry, e.g.,

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

Here, having the block inline would be less readable.

## Naming and overriding

Function declarations must have unique names. E.g.,

```
fn foo() {...}      // ok
fn bar() {...}      // ok
fn bar(): num {...} // error: duplicate names
```

Exception: functions may have the same name if they have fully distinct `self` types. A function with no self argument is considered distinct to any functions with `self`. If the `self` types are unrelated, there is no restriction on the other arguments or return type. If the `self` types are subtypes, other arguments must match exactly[^contra] and return types must be covariant. Due to the `self`'array sugar, `T` is not considered distinct from `[T]` or `[T; 1]`.

E.g., (note I use a 'top type' called `Any` in these examples, but I'm not actually proposing we have such a type, it's just easy to explain with).

```
fn foo(a: bool) { ... }
fn foo(self: num) { ... }           // ok
fn foo(self: T1): T1 { ... }        // ok
fn foo(self: T2): T2 { ... }        // ok
fn foo(self: [num]) { ... }         // error self: num and [num]
fn foo(self: Any, a?: num) { ... }  // error args must match, return types must match
fn foo(self: Any): Any { ... }      // ok
```

E.g., point constructors (similarly for `vec`/`Vec2d`/`Vec3d`):

```
fn pt(self: [num; 2]): Point2d { return Point2d { x = self[0], y = self[1] }}
fn pt(self: [num; 3]): Point3d { return Point3d { x = self[0], y = self[1], z = self[2] }}

// Used as
p = pt(0, 0)    // p: Point2d
q = pt(0, 0, 0) // q: Point3d
```

From the perspective of naming and overriding, there is no difference between a function declared in a module and one imported into a module. In the future, we might allow functions to be imported or declared in smaller scopes, for example in the body of a function or block (which are inside a wider scope such as a module or function body or block). Such names should not be visible outside the narrower scope (we might want some export mechanism, but in that case the names would be referred to similarly to field access on objects, rather than as bare names). The remaining question is whether a name in an inner scope can hide a name in an outer scope. I believe they probably should and that this should be integrated with the rules around `self` types, however, I don't think this is important to finalise now.

[^contra]: extension: non-`self` parameters could be allowed to be contravariant, including removing optional arguments or making them non-optional (with or without optional type).

### Dispatch

A function name whether imported from a module or defined locally, defines *a set of functions*. Likewise, referring to a function via a module (`moduleName.fnName`) references a set of functions. Passing or storing a function may refer to a specific function (`a = fn(...) { ... }`) or a set (`fn foo(...) { ... }; a = foo`); in the former case, no dispatch rules are applied (the latter case needs some thought and I would not support it initially).

The dispatch problem is choosing a specific function from the above sets of functions. Functions which are not in scope are never considered. Dispatch only considers the structure and type of `self` (or absence of a `self` argument), never the other arguments or how the function came to be in scope (import vs declaration). The algorithm for dispatch assumes the various desugarings around `self` have already been applied.

If there is no `self` argument, then a function with no `self` parameter is used (or error if there is no such function). Otherwise, a function with no `self` parameter is excluded from the candidate set.

The actual `self` argument is evaluated and it's dynamic type is used. The function with formal `self` parameter type with the most specialised super-type of the actual `self` argument's dynamic type is chosen. If there is no function in the set with a supertype `self` type, there is an error (note that this can be checked statically, with the actual dispatch done dynamically, which is standard for OO languages).

Once a specific function is chosen, then the arguments are checked, and the function can be executed.


### Import and export

Functions may be exported as today. When multiple functions with the same name are defined in a module, and then imported, all exported functions are imported, and if renamed, they are all renamed. Functions may be imported even with the same name as other imported or declared functions, as long as they follow the overriding rules defined above.

As an example of how functions can be organised, I'll go over a subset of the std lib. This isn't meant to be a proper proposal for std, but to demonstrate how the features in this design doc facilitate modularisation and organisation. I've elided most function bodies and private functions (and a whole bunch of functionality, obvs); consider the types to be strawmen (see [types design doc](types.md)). See the next section for an example of using these functions.

```
// std.kcl

// Re-exports of child modules.
export import * from 'math.kcl'
export import * from 'sketch.kcl'

export fn extrude(self: Sketch, len?: num, path?: Path): ExtrudedSolid { ... }

export object Point2D { ... }
export object Point3D { ... }

export fn pt(self: [num; 2]): Point2D { ... }
export fn pt(self: [num; 3]): Point3D { ... }
```

```
// math.kcl

export PI = 3.14

export fn sin(self: num): num { ... }
export fn sqrt(self: num): num { ... }
export fn pow(self: num, exponent: num): num { ... }
```

```
// sketch.kcl

import Point2D, Point3D from 'std.kcl'

export object Sketch { ... }
object PathBuilder { ... }

// Note that this interface and subtyping stuff is very premature
export interface SketchComponent
export object Path <: SketchComponent { ... }
export object Profile <: SketchComponent { ... }

export interface PathSegment
export object Line <: PathSegment { ... }
export object Arc <: PathSegment { ... }

export fn sketch(self: [SketchComponent], on: Plane) { ... }
export fn enclose(self: [PathSegment]): Profile { ... }

export fn line(from: Point2D, to?: Point2D, rel?: Vec2D, angle?: num, len?: num): Line { ... }

export fn arc(
  from: Point2D,
  angleStart?: num(-360..360),
  angleEnd?: num(-360..360),
  offset?: num(-360..360),
  to?: Point2D,
  rel?: Vec2D,
  radius?: num,
): Arc { ... }

export fn startSketch(on: Plane): Sketch { ... }
export fn startPath(self: Sketch, at: Point2D): PathBuilder { ... }

export fn line(self: PathBuilder, to?: Point2D, rel?: Vec2D, angle?: num, len?: num): PathBuilder {
  // Calls the no-self version of line,
  // note that we're relying on the 'arg = arg' shorthand and the partial equivalence of optional args and optional types
  l = line(from = self.segments.last.end, to, rel, angle, len)

  // append returns a new PathBuilder with an internal array of PathSegments which is extended by l
  return self.append(l)
}

export fn arc(
  self: PathBuilder,
  angleStart?: num(-360..360),
  angleEnd?: num(-360..360),
  offset?: num(-360..360),
  to?: Point2D,
  rel?: Vec2D,
  radius?: num,
): PathBuilder {
  // Strawman syntax for handling optional types, I believe the logic here will actually be more
  // complex because of the interaction between other arguments and startAngle.
  angleStart = if angleStart {
    angleStart
  } else {
    tangent(self.segments.last)
  }
  a = arc(from = self.segments.last.end, angleStart, angleEnd, offset, to, rel, radius)
  return self.append(a)
}

export fn enclose(self: PathBuilder): Sketch { ... }

export fn start(self: PathBuilder): Point2D {
  return self.segments.first.start
}
```


## Putting it all together: sketch examples

Types of the result of a function are shown in comments, that is just for explaining things here, they are not required nor would it be expected for users to write such comments. Ideally, the user never really thinks about types except as documentation or when implementing functions.

```
// std is implicitly imported, eqivalent to `import * from 'std.kcl'`

// A simple path-based sketch, using builder syntax
startSketch(on = XY)                  // : Sketch
  |> startPath(at = pt(0, 0))         // : PathBuilder
  |> line(rel = vec(6, 0))            // : PathBuilder
  |> arc(rel = vec(3, 3))             // : PathBuilder
  |> line(rel = vec(0, 6))            // : PathBuilder
  |> line(to = start())               // : PathBuilder
  |> enclose()                        // : Sketch

// The same sketch using, block syntax (not the ideal demonstration, but shows equivalence)
sketch(on = XY) {                                                    // : Sketch
  a = line(from = pt(0, 0), rel = vec(6, 0))                         // : Line <: PathSegment
  b = arc(from = a.end, angleStart = tangent(a), rel = vec(3, 3))    // : Arc <: PathSegment
  c = line(from = b.end, rel = vec(0, 6))                            // : Line <: PathSegment
  d = line(from = c.end, to = a.start)                               // : Line <: PathSegment
  enclose(a, b, c, d)                                                // : Profile <: SketchComponent <: [SketchComponent; 1] <: [SketchComponent]
}
```

Some notes on how the above works:

- `pt` and `vec` use the array `self` type sugar and overloading based on the `self` type (`[num; 2]` c.f., `[num; 3]`) which determines the return type (`Point2D` c.f., `Point3D`).
- In the pipeline example,
  - we start with a `Sketch` which is turned into a `PathBuilder`, the `enclose` call dispatches to `enclose(self: PathBuilder): Sketch` resulting in a `Sketch` (`PathBuilder` must hold a reference to the original sketch, to be concatenated with the profile found by enclosing the path).
  - For all the calls to line, arc, and enclose, `self` is taken to be the current `PathBuilder` in the pipeline
  - The call to `arc` results in something like today's `tangentialArcToRelative` since the start angle is taken from the previous segment of the path (see definition of `arc(self: PathBuilder, ...`)
  - The call to `start()` takes `self` from the pipeline (the current `PathBuilder`), to find the point at the start of the path.
- In the block syntax example,
  - the block becomes `self` of the call to `sketch`
  - the call to `arc` is again like today's `tangentialArcToRelative`, but with more explicit use of `tangent`
  - the `to` argument of `d` refers to the start of line `a`
  - `enclose` uses the array `self` syntax and is dispatched to `fn enclose(self: [PathSegment]): Profile`
- I am thinking about a type-powered shorthand where if the type is known, the `pt`/`vec` function name could be elided. In this case, I believe every use of `pt(...)` would become `(...)` and likewise for `vec` (I think the use of `pt`/`vec` is a bit noisy in these examples).
