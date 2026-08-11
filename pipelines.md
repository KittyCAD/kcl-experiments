# Pipelines and tags

Note: since drafting this doc, I've been thinking more about constraints and the implications on our syntax and core control flow. I've also been thinking about sketches which are not paths and therefore don't work quite as well with pipelines. I think, therefore, that we'll need some changes to what is proposed here, but many of the key changes will still be needed. See [this gist](https://gist.github.com/nrc/efdec535bd961a0f7b470480676fa9eb). More soon...

This document proposes some design changes for pipelines, the pipeline operators (`|>` and `%`), and tagging.

In summary:

* Remove `%` and make the lhs of `|>` be an 'implicit' receiver argument
  - This is affected by making the receiver responsible for the `from` parameter for lines, etc. It will need a bit more design work.
* Remove `$` and tags as arguments; support `as` for declaring tagging variables
* Support `{}` blocks in pipeline
  - There is overlap with the block syntax I proposed as an alternative to pipelines for non-path sketches.
* Introduce pipeline-aware `for` expressions
* Support infix operators in pipelines

Caveat: this proposal only makes sense in the context of other proposals around the fundamental model of KCL currently in development.

## Motivation

Pipelines are currently the defining feature of KCL: they are the most common control flow and its most unique syntax. They are fundamental for building geometry and that is fundamentally the point of KCL. They are the operation which users will read and write more than any other.

My understanding of the pipeline concept is that it is a first-class, ergonomic, and expressive implementation of the [builder pattern](https://en.wikipedia.org/wiki/Builder_pattern). This is useful for KCL because geometry has many, many ways it can be configured and many of the options should be ignored in many cases; and because step-by-step construction closely matches the workflow in the UI of the modelling app.

For both the user and the underlying engine, it is frequently required to refer to existing geometry both to transform it and to refer to it when making other changes or as a reference point for other changes. Examples:

* Take this sketch and extrude it
* Take this edge (of a solid) and chamfer it
* Take this point and draw a line to it
* Take this face (of a solid) and draw a sketch on it

This is currently accomplished by using *tags*. A tag is essentially just a variable (note that variables in KCL are constants, not mutable, but I'm using 'variable' in the most inclusive sense of a symbol holding a place for a value represented in an abstraction of execution as a location) with some special support for naming parts of geometry and the facility to declare them and pass them as an 'out' parameter to standard library functions.

### Issues with the current design

* Pipeline syntax is noisey
  - We have a novel operator `|>` for connecting operations in some sense
  - We have a novel symbol `%` for the current target of the pipeline (a distinguished variable/tag) which appears many times in a typical pipeline
  - Declaring a new tag uses `$`
  - There are often a lot of brackets  (`()` to call functions, `{}` to group arguments, `[]` to declare common types like points, vectors, etc.) and other punctuation (`:` for 'named arguments' in functions, `[]` for array indexing, typically a mix of strings, math symbols, commas, etc.)
  - There is lots of nesting: both deliberate (e.g., `|> hole(circle(...), ...)`, parentheses for controlling precedence in arithmetic) and incidental (e.g., grouping of arguments into objects, arrays of data, requiring multiple function calls which are conceptually related)
* The syntax structure does not match the intuition of the common case, either from working things out from first principles or by reference to other PLs.
  - `a |> b` suggests that `a` is piped into `b` (especially since we use the terminology 'pipeline'). However, this is not done by `|>`, which is more of a sequencing operation. `%` is required to actually pipe `a` into `b` (e.g., if `b` is `f(%)`).
  - Many languages offer a similar `a.f()` syntax (replace `|>` with `.`, note the visual similarity in the common (in KCL) case of multiple steps) where `a` is passed to `f` in some way.
* The relationship between tags and variables is fuzzy
* Tagging cannot be used with user-defined functions
* 'Out parameters' are a notoriously difficult feature for beginner programmers, and we rely on this semantics for tagging.
* Most variables have to be declared before use, tags do not (although the first use is called a declaration, it is not a declaration that will look familiar to programmers or similar to other variable declarations)
* Tagging is not expressive enough
* Any non-trivial tagging is complex and unintuitive (i.e., the learning curve is not smooth).

This proposal will not fix all of the above, but will address some issues. In particular, I plan to think in more depth about the more complex tagging requirements.

# Proposed changes

## Core syntax and semantics

The pipeline operator becomes a method call operator.

Alternative: we could replace `|>` with `.` - this would be closer to most other languages, but I feel would lose some of KCL's character.

### Functions may have an explicit receiver

In the declaration, this is the first argument of a function and must use a distinguished keyword (most PLs use `self` or `this`) or symbol (we could reuse `%` either alone or as a decorator on a name, e.g., `%sketch`, or any other sigil). I prefer a keyword, I have no preference which. Note that in other PLs the receiver is used for dispatch of a method, not just the syntax of calls. That is not proposed yet.

In a call, the lhs of the `|>` operator becomes the receiver. If the function is called without the pipeline, then the argument must be specified.

E.g.,

```
f = fn (this) => { ... }

startSketchOn(...) |> f()
f(startSketchOn(...))
```

In both calls, the result of `startSketchOn` is passed to `f` as `this`.


### Remove `%`

The `%` operator is no longer supported.

In the common case, the use of `%` is replaced with use of a receiver (see above).

The other use cases for `%` I can find are for tagging (e.g., `profileStartX(%)`) and nested geometry (e.g., `hole(circle(..., %), %)``). I'm optimistic that by refining the design of std and implementing some extensions to tagging, we can reduce the need for these uses (both are future work). In the meantime, these uses can be implemented by breaking the pipeline with an intermediate variable, e.g.,

```
// Current KCL
sketch006 = startSketchOn('XZ')
  |> startProfileAt([0.1, 1], %)
  |> line([0.1, 0], %)
  |> angledLineToX({ angle: 10, to: 0.05 }, %)
  |> yLine(10, %)
  |> line([0.6, 0], %)
  |> yLine(-.05, %)
  |> tangentialArc({ radius: 0.6, offset: -90 }, %)
  |> lineTo([profileStartX(%), profileStartY(%)], %)
  |> close(%)
  |> revolve({ axis: 'y' }, %)

// Proposed KCL
sketch006 = startSketchOn('XZ')
  |> startProfileAt([0.1, 1])
  |> line([0.1, 0])
  |> angledLineToX({ angle: 10, to: 0.05 })
  |> yLine(10)
  |> line([0.6, 0])
  |> yLine(-.05)
  |> tangentialArc({ radius: 0.6, offset: -90 })
sketch007 = sketch006
  |> lineTo([profileStartX(sketch006), profileStartY(sketch006)])
  |> close()
  |> revolve({ axis: 'y' })
```

See below ('Tagging') for a further improvement to this example.

#### Alternative

Retain `%` or replace it with a keyword (which I would prefer) for the rare cases where it is required (c.f., the current situation where it is frequently required).

## Tagging

Remove the `$` operator, remove the concept of passing tags to functions. Allow the `as` keyword to be used within pipelines to assign intermediate results to a variable. Note that this usage of `as` follows its use in `import` statements and has the same semantics of introducing a new name.

This syntax is easier to read and comprehend, extends to user-defined functions, and reduces the total feature count of the language (since it is already used in imports).

Example: `foo = ... |> bar() as baz |> qux()`, here the final result is assigned into `foo` (no change to semantics), the result of executing `|> bar()` is assigned into `baz` (equivalent to `|> bar($baz)` in the current syntax).

The earlier example becomes:

```
sketch006 = startSketchOn('XZ')
  |> startProfileAt([0.1, 1])
  |> line([0.1, 0])
  |> angledLineToX({ angle: 10, to: 0.05 })
  |> yLine(10)
  |> line([0.6, 0])
  |> yLine(-.05)
  |> tangentialArc({ radius: 0.6, offset: -90 }) as arcSketch
  |> lineTo([profileStartX(arcSketch), profileStartY(arcSketch)])
  |> close()
  |> revolve({ axis: 'y' })
```

Question: does the tag name refer to the individual line segment or the current version of the sketch or path being built up? 

### Alternative

Rather than making `as` part of the pipeline syntax, it could be allowed in any expression. The above description would still apply, but it could also be applied to sub-expressions, e.g., `hole(circle(...) as innerCircle)`.

This would be more flexible and would avoid learners hitting the issue of knowing where `as` can be used ("I can use it here and here, so why can't I use it here?"). However, I believe it would lead to poor programming style: it is generally better to use a variable declaration to refer to reused geometry or data rather than use `as` since it will be easier to scan code for declarations and it encourages a less nested, more straightforward coding style. On the other hand, pipelining is ergonomic and allowing `as` there prevents breaking pipelines arbitrarily.

The precedence in general sub-expressions may be confusing, e.g., in `a + b as c + d`, does `c` refer to the value of `b` or of `a + b`?

If we allow this, then it suggests that within a pipeline it refers to the segment, then how do we refer to the current (or a previous) intermediate value of the object being built up? (Which is the use case for `%`).

## Braced blocks in pipelines

Braced blocks are allowed in pipelines. The current receiver of the pipeline is piped into the *last* expression of the block (the block must have a last expression, similar to `if` blocks). The result of the block is the result of the final expression. Variables within the block are scoped to the block. (Possible extension: use `export x = ...` or `... export as x` to make variables visible in the enclosing scope).

```
startSketchOn('XZ')
  |> {
    x = sin(42)
    y = cos(42)
    line([x, y])
  }
  |> close()

// Or equivalently

startSketchOn('XZ')
  |> {
    x = sin(42)
    y = cos(42)
    line([x, y])
      |> close()
  }
```

This allows more precise scoping of variables (in the above example, `x` and `y` are not available outside the block), but the primary motivator is for use in `for` expressions, see below.

## `for` expressions

Note: much overlap with patterns, and control flow is a headache with respect to symbolic execution, so this may need work...

Syntax: `'for' id 'in' expr_1 (|> expr_2 | block_expr)`. May appear within a pipeline or as an expression outside a pipeline.

In the simplest case (with no preceding pipeline), `expr_1` is evaluated to some kind of an object `s` with sequence type (an array or range). `expr_2` (or `block_expr`) is evaluated with each item in `s` bound in turn to `id`. The result of execution is an array of the results of evaluating `expr_2` (i.e., you can think of the expression as performing 'for each' or 'map' on its input).

Examples:

```
//prints "0\n1\n2\n3\n4\n"
for i in [0..5]
  |> println(i)

// x has value [0, 2, 4, 6, 8]
x = i in [0..5]
  |> i * 2

// Draws five objects starting at (0, 0), (1, 1), ...
for i in [0..5] {
  startSketchOn('XZ')
    |> startProfileAt([i, i])
    |> line([1, 0])
    ...
    |> close()
  }
}
```

When used in a pipeline, the input to `for` is passed into `expr_2` as the receiver for the first iteration. The result of that execution is passed into the next iteration and so forth until the final result is returned as the value of the for expression. I.e., it is a kind of reduce or fold operation. E.g.,

```
sketch
  |> for i in [0..n]
    |> drawOneSectorOfGear(i)
  |> extrude(...)
```

Here, `sketch` is the receiver of `drawOneSectorOfGear` on the first iteration, and the result of that is the receiver on the next. The final result is the extrusion of the inital sketch with `n` sectors.

Example with block:

```
sketch
  |> for i in [0..n] {
    x = sin(i)
    y = cos(i)
    drawOneSectorOfGear(x, y)
  }
  |> extrude(...)
```

Note that only a single pipeline stage is repeated (thus the indentation in the first example). To apply longer pipelines, use a block, e.g.,:

```
sketch
  |> i in for [0..n] {
    drawOneSectorOfGear(i)
      |> foo()
  }
  |> extrude(...)
```

Tagging using `as` works as normal within a block, tagging the whole block (or the whole single expression, if there is no block), produces an array of tags. E.g.,

```
sketch
  |> for i in [0..n]
    |> drawOneSectorOfGear(i) as x
  |> extrude(...)

// or

sketch
  |> for i in [0..n] {
    x = sin(i)
    y = cos(i)
    drawOneSectorOfGear(x, y)
  } as x
  |> extrude(...)
```

In both cases, `x` has type [T] if `drawOneSectorOfGear` returns a `T`.

Extension: Assumes we can use `export as` to export a variable from a block. Tagging within a for loop produces a somewhat special variable: it refers to the current iteration's tag within the loop and an array of tags outside the loop. E.g.,

```
sketch
  |> for i in [0..n] {
    drawOneSectorOfGear(i) export as x
      |> foo(x) // x has type T
  }
  |> bar(x)     // x has type [T]
  |> extrude(...)
```

### Alternative syntax

We could make the braced block mandatory rather than optional with a single expression, replacing `|>` in single expression versions:

```
x = i in [0..5] { i * 2 }

sketch
  |> for i in [0..n] {
    drawOneSectorOfGear(i)
  }
  |> extrude(...)

sketch
  |> for i in [0..n] as {
    drawOneSectorOfGear(i)
      |> foo()
  }
  |> extrude(...)
```

We could use  `for [...] as i` rather than `for i in [...]`. This was previously proposed. The `in` syntax is closer to other PLs and reads better, however, it does not reuse the `as` keyword and so requires a new keyword.

### Infix operators

Note: I think I prefer having some symbol or (preferably) keyword for the current object of the pipeline (`%` today), so operators would be used as `... |> % + 1` or whatever.

Infix operators (e.g., `+`, `-`) can be used in pipelines with the lhs of the pipeline operator being used as the lhs side of the infix operator. E.g., `1 |> + 1` is equivalent to `1 + 1`. This is not really necessary by itself, but might be if in the future we support infix operators for 2D or 3D geometry, e.g., `sketch |> extrude(...) |> union something |> union somethingElse()`, assuming `a union b` is allowed.

I expect this part of the proposal should be low priority to implement. I'm just including it here for completeness (and it seems to come up a lot in my examples).

## Alternative: rely less on pipelines

[Example](https://github.com/KittyCAD/modeling-app/issues/2728#issuecomment-2361355398):

```
squareSketch = sketchOn('XY', [
  line(4, 0), 
  line(0, 4), 
  line(-4, 0), 
  line(0, -4),
])
```

Advantages:

* Simpler
  - More declarative since we're declaring the result in one go rather than the steps to building it
  - Less visual noise (no `|>`, etc.)
  - Possibly fewer required steps
* More expressive - user can manipulate arrays rather than just build up objects
* Perhaps closer to the execution model of building a geometry model and sending that to the engine, rather than sending each step to the engine
* Easier to optimise pipelines by batching engine calls and reasoning about which calculations can be done locally

Disadvantages (or mitigating factors to the above)

* Programming with arrays requires more complex programming features (good for power users, bad for non-programmers and beginners) or incremental construction of arrays, in which case we're doing the same as using pipelines for building, just with an extra step (combining geometry into an array and turning the array into geometry, vs combining geometry into geometry).
* Some of the simplifications of this approach could be achieved in the pipelines approach by improving the design of the std lib functions
* Encourages either more nesting of expressions (hard to read) or many variables (inconvenient, hard to read).


# Further work

* Advanced tagging use cases
* Ergonomics of function declaration and calls, function organisation
* Arrays, collections, iteration, etc.
* More syntax polishing
* Rationalising std
