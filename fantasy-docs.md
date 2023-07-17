# Intro

These are imaginative, ambitious docs designed to explore designs for a good CAD language. None of this is final, everything is experimental. We're making this public to stimulate your imagination and help workshop our ideas.

Feedback is welcome via GitHub issues or PRs.

Again, this is **experimental** and **exploratory** work.

# KCL Fantasy Docs

KCL is a language for describing manufacturing projects. KCL projects represent real-world objects. By modelling them in KCL, you get the ability to visualize, measure, analyze and plan them. Much like a programming language lets you define how your software works, KCL is a CAD language: it lets you define how your physical objects work.

KCL builds up big complex objects from small, simple parts. In a typical project, you'll start by drawing one-dimensional _lines_, and joining many lines into two-dimensional _paths_. If your path is a closed shape, like a square, it can be turned into a _Solid3d_ by extruding it or rotating it around an axis. KCL supports other workflows like [NURBS](https://www.3ds.com/store/cad/nurbs-modeling) too.

## Design principles

In most programming languages, your code creates a program which _runs_. A program starts running at a certain moment, executes calculations, reacts to events, changes or recalculates data, executes side-effects, and eventually exits. KCL isn't _really_ a programming language. KCL code doesn't create a program, and it cannot be run. Instead, KCL files merely describe real-world 3D objects (describing their geometry, materials, manufacturing instructions, etc). This means KCL can't print to a terminal, make HTTP requests, mutate variables or even loop over a range of numbers. All it can do is describe 3D objects. 

In many ways, KCL is more like a markup language for math and physics than a programming language. But it does borrow two key features from programming languages: types and functions. So if you've used a static, functional language (like OCaml, Haskell or Elm), KCL will seem familiar.

## CAD types

In KCL, your big complicated final design is made up of many small, simple pieces. These pieces are called _objects_, and they can be lines, 2d shapes, 3d shapes, etc. Every KCL object has a type (for example, Solid3d or Line) which defines what kind of operations you can perform on it. Some types can be constructed from other types (e.g. a Solid3d can be made by extruding a Solid2d up into the third dimension).

KCL types include:

### Point
A _point_ is a point in 3D space. It has an X, Y and Z component, all of which are fractional, possibly-negative numbers.

Arithmetic operations on two numbers can be used on two points, applying the operation component-wise (e.g. `p1 + p2` = `(p1.x + p2.x, p1.y + p2.y)`).

### Path segment
A _path segment_ is a pair of points in 3D space and an equation for how to move between them. KCL supports several different types of path segment:

 - Straight line segments
 - Bezier curves
 - Arcs (subset of a circle's perimeter)

One path segment can be:
 - Translated
 - Rotated
 - Scaled

### Path

A _path_ is a sequence of path segments. The first segment ends where the second segment begins. The second segment ends where the third segment begins, etc etc.

Create a path by:
 - Joining one or more segments

One path can be:
 - Translated
 - Rotated
 - Scaled

Two or more paths can be:
 - Joined (concatenated)

### Solid2d

_Solid2ds_ are shapes like squares, circles, polygons, etc. 

Create a Solid2d by:
 - Closing a path

One Solid2d can be:
 - Translated
 - Rotated
 - Scaled

Two or more Solid2ds can be:
 - Unioned (outputs a shape with the combined area of input shapes)
 - Subtracted (outputs the first shape, but missing any area covered by the other shapes)
 - Intersected (outputs a shape with only the area shared by both input shapes)
 - Excluded (outputs a shape with only the area contained by exactly one input shape, i.e. XOR)

### Solid3d

_Solid3ds_ can be simple shapes like spheres and cubes, or a combination of them (a sphere on top of a cube).

Every type we've previously examined had geometric properties, like length or height. But Solid3d is different, because it also has a _material_. Materials (for example, aluminium or plastic) let KittyCAD analyze your model and report important data like its weight or tensile strength. We'll discuss materials more below.

Create a Solid3d by:
 - Extruding a 2D shape into the third dimension (along a straight line, or any path)
 - Rotating a 2D shape around a 3D axis

One Solid3d can be:
 - Translated
 - Rotated
 - Scaled

Two or more Solid3ds can be:
 - Joined (like a Solid2d)
 - Subtracted (like a Solid2d)
 - Intersected (like a Solid2d)
 - Excluded (like a Solid2d)

### NURBS

NURBS are an alternative way to build 3D shapes. They define their shapes using polynomial equations, instead of from extruding 2D solids. 

Create a NURBS by:
 - Using KCL's NURBS library to render a polynomial

Operations on one or more NURBS shapes will be fleshed out soon.

### Text

Some standard library functions render text onto 2D/3D shapes. These functions need a `Text` type, which is any UTF-8 text. The standard library has a `text` package with other functions for choosing fonts, specifying font sizes, etc.

### Measurements

Some programming languages (e.g. Javascript) just use one number type. It's called `number` and it's any real number (subject to approximations from how the language chooses to represent numbers within 4 bytes). Other languages use several different number types, distinguishing whether the number can be negative, or fractional, or its maximum and minimum. Using different types for different kinds of numbers helps programmers ensure their code is correct. For example, you could use a signed integer for representing a bank account balance, an unsigned integer for representing the number of apples in a bag, and a signed fractional number for representing the average change in temperature over an hour.

The problem with both these approaches is that the programming language has no idea what unit of measurement those numbers represent. Javascript doesn't know if 123 is measuring inches, pounds, or lightyears per second. This means that programmers need to be very careful about naming variables, or using comments, to prevent mistakes. You'd cause [a lot of chaos][unit-chaos] if one teammate assumed your models were measured in inches, and another assumed they were using centimeters!

To avoid this, KCL doesn't use numbers. Instead, it uses specific measures, like Distance or Quantity, which support various units. This means that your American teammates can work in inches, and normal people can use centimeters. KCL seamlessly interoperates between these different measurements, and the KittyCAD API can accept queries and return responses with any set of units.

This ensures you never pass a distance into a function expecting an angle, eliminating a whole class of bugs that other programming languages have. All types come in signed and unsigned variants, so you can declare a function which takes Distance or UDistance. 

KCL measurement types include:
 - Distance (fractional number, constructors include centimeters, inches, etc)
 - Area (fractional number, constructors include square centimeters etc)
 - Volume (fractional number, constructors include cubic centimeters etc)
 - Angle (fractional number, can be radians, degrees, etc)
 - Time (Supports units from nanoseconds to hours, integer)
 - Linear acceleration (distance / time)
 - Angular velocity (angle / time)
 - Mass (fractional number, constructors include pounds, grams, etc)
 - Force (fractional, constructors include Newtons, pound-force, etc)
 - Moment (fractional, constructors include inch-pounds etc)
 - Pressure (fractional, constructors include psi, kPa, etc)
 - General purpose Integer and Real types
   - These are for uses like "make N wheels" or "take the average of these distances"
   - Users won't need them very often, because the KCL standard library tries very hard to use more structured Measurement types.

A measurement type (e.g. `Force` or `Distance`) may have a lot of different constructors, but only one type. For example, here are some different ways to specify a value for some measurement:
 1. A predefined unit, e.g.
     a. `Force.Newton(1)`
     b. `Force.Pound(0.224809)`
     c. `Distance.Metre(1)`
     d. `Distance.Yard(0.9144)`
 2. A combination of different units, using a standard mathematical definition, e.g.
     a. `Force.mdt(Mass.Kilogram(1), Distance.Metre(1), Time.Second(1))` (force = mass * distance * time)
     b. `Area.rectangle(Distance.Metre(1), Distance.Metre(2))` (area = distance_x * distance_y)
     c. `Area.rectangle(Distance.Metre(1), Distance.Yard(1.8288))` (same equation as above, but mixing imperial and metric)

Both these expressions have the same _type_ (`Force`) but use different _constructors_ to give the specific value. For more about syntax, see the Syntax section below.

#### Operations on measurements

KCL supports various mathematical operations on your measurements. Some of these are trivial -- adding two distances gives you their combined distances. But adding a distance and an area gives you a compilation error.

Every mathematical operator in KCL corresponds to a function. For example, `x + y` is shorthand for `add(x, y)`. You can look up the docs for any operator function like `add` to see which types it supports and what its shortcut operator is. Most types support general-purpose arithmetic operators like +, -, *, / and ** (exponent) in the way you'd expect. 


### Materials

KCL is a language for designing physical, real-world goods. This means when you design a cube in KCL, it's not sufficient to just give the cube's dimensions. You have to give its material too. A 10 foot cube made of plastic has very different properties from a 10 foot cube made of uranium!

KittyCAD maintains a database with many predefined materials. They're organized by general "substance" and then by "alloy". For example, the substance aluminium has several alloys, including AISI_201, Ferrallium_225 and A-286. You can use any of these predefined materials in KCL, or you can define your own materials (see Syntax below).

Each material has a _set of properties_. Each property is a name, measurement and value (for more about measurement types, see above). For example:
 - `("density", Density<Pounds, CubicInch>, 33.4234)`
 - `("tensile_strength", Pressure.Pascal, 222.1)`

KCL supports a long list of various property names. Materials can define any number of properties. This way, when you're starting out a project, you only need to define the properties that make sense for your specific problem. As your project expands, you can add more properties as you need them.

If you try to run a KittyCAD query on your model, but one of its materials is missing a property required for the measurement, KittyCAD API will return an error and let you know which properties are missing from which materials. Your other queries will still succeed, though. This means you can start modeling your designs quickly without getting bogged down by measuring every single property of every single material you might ever need.

KCL also has a special material called None. This is the default material if you don't specify a Solid3d's material. It doesn't have any properties defined, so KittyCAD queries about an object whose material is None will fail. This material is suitable for use in 3D graphics work (where you don't care about physical materials), or for very early-stage prototyping of real-world goods, or for designers who are just experimenting with the superficial appearance of the object. When it comes time to actually manfacture your design, KittyCAD's API won't let you execute queries or manufacturing orders if None is used anywhere. The KCL linter also shows a warning if you use None, so that you don't accidentally send your designs to a partner without defining the material.

## General purpose types

### Arrays

An array is an ordered list of values. For example, an array of points contains a first point, second point, etc. All the values in an array must be the same type -- you can have an array of points, or an array of lines, but not an array with both points and lines. Arrays can have any number of elements (0 or more).

Arrays exist to help you repeat shapes many times. For example, if you want to create five gears in a row, each 1cm apart, you can create an array of numbers from 1 to 5, then map each of them to a distance, then map each of them to your gear function.

You can create an array by:
 - Supplying all the values it should have
 - Giving a range of integers (e.g. an array of numbers from A to B)
 - Sorting an existing array (if the array's type variable is `comparable`, see below)
 - Inserting, changing or deleting an element from an existing array

Arrays support operations like:
 - Map
 - Reduce
 - Filter
 - Length
 - Get an element

### Type variables

Technically there's no such type as `Array`, rather we have `Array Point` or `Array PathSegment` etc. The general way we refer to arrays is `Array a`, where `a` is a _type variable_. The `Array a` type is _generic_ over the _type variable_ `a`. When you have a specific element in the array, e.g. `Array Point`, the type is no longer generic -- its type variable no longer exists, it has a specific value now.

There are a few special type variables called _constrained type variables_. These include:
 - Number
 - Comparable

Constrained type variables are similar to interfaces/traits/protocols in other languages, except currently KCL defines its own constrained type variables, and there's no way for users to define their own. We're very happy to support that in the future, but for now, we don't have a clear reason to add them.

### Tuples

Frequently you'll want to return multiple values from a function. You do this via _tuples_, which are an ordered list of values. If your function should return a distance, point and path segment, it doesn't return three separate values. Instead it returns one tuple containing three elements: `(Distance, Point, PathSegment)`. Tuples are similar to arrays, except:

 - They have a fixed number of elements
     - This means you cannot remove or add new elements
 - They can contain multiple types of values

A tuple of _n_ elements supports being created from _n_ separate elements, or destructured (broken apart) into _n_ separate elements.

Tuples, like arrays, are generic over a type variable. E.g. the type `(a, b)` is a tuple generic over a type variable `a` and `b`. These variables can also be constrained.

### Enums

An enum can be one of several possible values. For example:

```kcl
enum ExtrudeTop = Open | Closed
```

defines a new type `ExtrudeTop` with two possible values: `Open` and `Closed`.

Enums can have fields associated with them:

```kcl
enum ExtrudeTop = Open | Closed(Material)
```

has two possible variants. The first variant is `Open` and this variant only has one possible value: the `Open` itself. The second variant,`Closed`, has a field `Material`, so if you create a value of type `ExtrudeTop.Material` it must also have a `Material`.

The standard library uses enums, and users can create their own. There are no nulls or undefined in KCL, instead, we use the Option enum:

```kcl
enum Option a = None | Some(a)
```

This enum has two variants. Either it is `None` or it's `Some`, and if it's `Some` then it also has a value of type `a` (see the "Type variables" section above).

## Syntax

### Functions

A KCL program is made up of _functions_. A function has a name, parameters, and evaluates to an expression. They look like this:

```kcl
/// A can for our line of *awesome* new [baked beans](https://example.com/beans).
can_of_beans = (radius: Distance, height: Distance -> Solid3d) =>
    circle(radius)
    |> extrude_closed(height)
```

Let's break this down line-by-line.
 1. Docstring: a comment which describes the function below it. Your KCL editor probably supports showing this comment when you mouse over the function, or over a visualization of the 3D shape it outputs. Your editor/viewer also probably supports Markdown.
 2. This is where the function starts. First is the function name, "can_of_beans". The name is followed by an `=`, then the function signature. The signature describes the function's parameters and return types. This function takes two parameters, called "radius" and "height". Both parameters have type Distance. It returns a Solid3d. The `=>` marks the end of the function signature, and the start of the function body.
 3. The function body is an _expression_. The first line of the expression is calling the function "circle" with the parameter "radius".
 4. The |> operator composes two functions. If you see `f |> g` it means "calculate `f` then apply its output as input to `g`". So, this line takes the circle from the previous line, and uses it as the last parameter to `extrude_closed`. 

You could write this same function in a different way without the `|>` operator: 

```kcl
can_of_beans = (radius: Distance, height: Distance -> Solid3d) =>
    extrude_closed(height, circle(radius))
```

But we generally find the `|>` operator makes your KCL functions easier to read.

In this example function, we specified the types of both input parameters and the output type. But the KCL compiler is smart! It's smart enough to infer the types of our parameters and return types even if you don't specify them. So you could have written

```kcl
can_of_beans = (radius, height) =>
    circle(radius)
    |> extrude_closed(height)
```

Here, the KCL compiler:
 * Infers that `radius` must be a `Distance` because it knows the built-in function `circle` has a `Distance` as its first parameter. 
 * Infers `height` must be a `Distance` because the second parameter of `extrude_closed` is a `Distance`. 
 * Infers the function returns a `Solid3d` because it's returning the return value of `extrude_closed`, which is `Solid3d`.

To invoke the function, you'd do this:

```kcl
can_of_beans(Distance.Centimeter(10), Distance.Foot(1))
```

(Note that, as discussed above, this example uses KCL measurement types (e.g. distance) instead of general-purpose number types. This lets you seamlessly interoperate between different units of measurement, like feet and centimeters)

This is an expression which evaluates `can_of_beans` with its two input parameters, i.e. radius and height. You can use this anywhere an expression is valid, which currently is just 
1. As an argument to a function
2. As the body of a function


Some units have aliases, so you could also write

```kcl
can_of_beans(Cm(10), Ft(1))
```

See the [docs](units) for all units and aliases.

Every KCL function body is a single expression. If a function body gets really big, it might be hard to read in a single expression. So you can factor out certain parts into named constants, using let-in notation. Like this:

```kcl
let
  can_radius = Cm(10)
  can_height = can_radius * 5
in can_of_beans(can_radius, can_height)
```

The constants you create in `let` are scoped to the let-in expression. The value of the expression is the `in` part. Let-in blocks are a standard piece of notation from functional programming languages (e.g. in [Elm][elm-let-in], [OCaml][ocaml-let-in] and [Haskell][haskell-let-in]) that help make large expressions more readable. We find they make large KCL functions readable too.

#### Constants

KCL doesn't have any mutation or changes, so there aren't any variables. Files contain a number of functions -- that's it. 

Other languages have named constants and variables. KCL doesn't have variables (because the language describes unchanging geometry and physical characteristics of real-world objects). But it _does_ have named constants. Here's how you declare them.

```kcl
my_can = can_of_beans(Cm(10), Ft(1))
```

This declares a named constant called `my_can`, which is the result of calling the `can_of_beans` function we defined above. KCL compiler inferred the type, but you can add a type annotation if you want to:

```kcl
my_can: Solid3d = can_of_beans(Cm(10), Ft(1))
```

This named constant is actually just syntactic sugar for a function that takes 0 parameters. After all, functions called with the same inputs always return the same value -- they're fully deterministic. So a function with 0 parameters is just a function that always returns a constant value. Or, to simplify: it _is_ a constant value. 

Without the syntactic sugar, `my_can` could be declared like this:

```kcl
my_can = (-> Solid3d) => can_of_beans(Cm(10), Ft(1))
```

Note the function signature. Function signatures are always (parameters -> return type), but here we have no parameters, so the function signature just omits them.

#### Functions as values

Sometimes, functions are parameters to other functions.

```kcl
doubleDistance = (d: Distance) =>
    d * 2

doubleAllDistances = (distances: List Distance -> List Distance) =>
    List.map(doubleDistance, distances)
```
Here, the `doubleAllDistances` function takes a list of distances and returns a list where all distances are doubled. It does this using the standard library function `List.map`. This takes two parameters:

1. A function to call on every element of a list
2. The list whose elements should be passed into the above function

This is neat. You can do a lot with standard library functions like this. However, there's another way to write this code.

```kcl
doubleAllDistances = (distances: List Distance -> List Distance) =>
    List.map((x) => x * 2, distances)
```

In this version, we've replaced the named function `doubleDistance` with an _anonymous function_ (also known as a _closure_). These closures use the same syntax for function declaration -- parameters, then `=>`, then the body. This lets you keep your code a little bit more concise.

Again, you don't need to specify function types, but if you want to, you can.

```kcl
doubleAllDistances = (distances: List Distance -> List Distance) =>
    List.map((x: Distance -> Distance) => x * 2, distances)
```

#### Keyword arguments

All the functions you saw previously had a list of required arguments. Users will look at the function definition, read every parameter from first to last, and pass in the right values as arguments.

These are called _positional_ or _required_ arguments. But KCL also supports _keyword_ arguments. Like this:

```kcl
sphere = (radius: Distance, material: Material = Aluminium.ISO5052) => ...
```

Here, the `material` parameter is a _keyword parameter_. It's optional, so if the caller doesn't provide it, it takes a default value, in this case the `Aluminium.ISO5052` alloy from the KCL standard library.

You pass keyword arguments like this:

```kcl
/// Using a keyword argument.
sphere(Distance;:metre(1), material = Plastic.ISO1234)

/// Or, don't use a keyword argument and rely on the default.
sphere(Distance::metre(1))
```

Keyword arguments help keep your KCL programs readable, and allows us to add new features to the standard library in a backwards-compatible way. Suppose that KittyCAD releases KCL 1.4, which adds a new positional argument to a standard library function `sphere`. Any programs using the definition of `sphere` from KCL 1.3 would stop compiling when you upgrade to 1.4 (because they're missing a parameter to `sphere`). But if we add the new parameter as a _keyword parameter_, your existing programs will keep working -- they'll just use the default value for that parameter.

### KCL files 

KCL files are just a collection of KCL functions (including constants). 

### "Running" vs. "Using" KCL files

Traditional programming language files can be run, by calling a special "main" function. This executes side-effects, like printing to the terminal or opening a video chat session. But KCL describes physical goods, not software, so you cannot "run" a KCL function to do something. 

This invites the question: how do I use this function and why did I write it?

There is an open ecosystem of tooling that understands KCL files, and can visualize or analyze the functions contained therein. Here are some examples of what you can do with a KCL function.

1. Open it in a KCL viewer. The primary KCL visualizer is built into KittyCAD's [modeling app](untitled-app). However, other visualizers exist too. These visualizers help you understand your model and show it to teammates, clients, fans, etc. 
2. Send it to a service like KittyCAD's analysis API. Generally, you send a KCL file, a query type (e.g. "mass" or "cost to print") and your desired unit of measurement (e.g. "kilograms") to that API. Then the API will analyze your KCL, figure out the answer, and convert it to your requested units.
3. Export it to KittyCAD's GLTF file format, then send that to 3D printing services or manufacturing services. They'll print/manufacture the object your function describes.
4. Convert it to other, less advanced formats, for your colleagues stuck at legacy companies that use Autodesk. 

In all these cases, you can choose one or more KCL functions to visualize/analyze/print/export. If you don't specify, the KCL ecosystem generally defaults to looking for a function called `main`. This convention is useful! For example, if a client wants you to build a bookshelf, you can send them a KCL file, where the `main` function outputs the bookshelf. When they open the file in a KCL viewer, they'll see the bookshelf. But they can also open up the sidebar, and look at all the other KCL functions your bookshelf is composed of. Then they can visualize those function separately -- e.g. they might want to drill down to view only the shelf, or the backboard.

If your function accepts 0 parameters, then it can be visualized easily. But how do you visualize a function like `can_of_beans(radius: Distance, height: Distance)`? It doesn't describe a single can, it describes a kind of general formula for making cans like that. Luckily, tooling can generally accept values from the user and pass them into KCL function args. For example, KittyCAD's KCL viewer lets you choose a function to view. If that function has parameters, you can enter the appropriate values into the UI. This lets you experiment and see what your functions look like with various parameter values.

This is a really powerful way to let consumers customize the goods you've designed before buying or manufacturing them. For example, you might put a design for a cool 3D printed office chair on thingiverse.com which has a function `main(name: Text)`. This function describes a chair with the given name embossed into the back. When a consumer wants to 3D print it, the 3D printing service will let them input their desired name, view the chair with that name embossed, and then order it.

## Tests
Functions are marked as tests by putting the `#[test]` attribute above them. They're run via the KittyCAD CLI or by the test runner built into the KittyCAD modeling app.

```kcl
#[test]
division_by_1_doesnt_change_number = () => 
    assert_eq(4/1, 4)

// Because these functions take no arguments, you can use the syntax sugar
// from the "Constants" section above.
#[test]
division_by_10 =
let
    expected = 10;
    actual = 100/10;
in assert_eq(expected, actual)
```
The `assert_eq` function will fail the test if the arguments aren't equal. There are similar functions like `assert()` which just checks if its argument is true, or `assert_ne()` which asserts the two are not equal. Tests are run in parallel, because there's no way for two tests to interfere with each other.

Test functions cannot take parameters, nor can they return values. So, what if you want to test many different (expected, actual) pairs for your function? Well, you can call `assert_eq` on a list of values. Like this:

```kcl
#[test]
multiplication_by_zero() =
let
    n = 100
    inputs = List.range(0, n) // A list of numbers from `0` to `n`.
    expected = List.replicate(0, n) // A list of length `n`, every element is `0`.
    actual = List.map((x) => x * 0, inputs)
in
    List.map2(assert_eq, actual, expected)
```
Here, the function `List.map2` is a lot like `List.map` except it has _two_ input lists. Its function argument takes an element from each list, instead of just from one list. So, it takes a function of type `(a, b) => c`, a `List a` and a `List b` and passes them into the function, element by element, creating a `List c`.

So, used here, it takes the input lists `actual` and `expected`, then passes an element from each into `assert_eq`.

[units]: https://kittycad.io/docs/units
[untitled-app]: https://kittycad.io/untitled-app
[ocaml-let-in]: https://courses.cs.cornell.edu/cs3110/2021sp/textbook/basics/let_expressions.html
[haskell-let-in]: https://www.cmi.ac.in/~madhavan/courses/pl2009/lecturenotes/lecture-notes/node70.html
[elm-let-in]: https://elm-lang.org/docs/syntax#let-expressions
[unit-chaos]: https://www.mentalfloss.com/article/25845/quick-6-six-unit-conversion-disasters
