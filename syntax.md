# Syntactic improvements

This doc proposes a bunch of improvements with the goal of a simple, cleaner, more intuitive syntax. Note that some of these changes go beyond just syntax changes though the effect is a cleaner syntax.

Currently, the first impression of KCL is poor - it feels very noisey with a lot of punctuation and symbols with non-obvious meanings and which are hard to search for.

For example, consider this sample

```
lugHoles = startSketchOn(lugBase, 'END')
  |> circle({
       center: [lugSpacing / 2, 0],
       radius: 16 * mm() / 2
     }, %)
  |> patternCircular2d({
       arcDegrees: 360,
       center: [0, 0],
       instances: lugCount,
       rotateDuplicates: true
     }, %)
  |> extrude(-wheelWidth / 20, %)
```

A newcomer might question:

- What does `|>` do?
- What does `'END'` mean? Why is it quoted?
- Why doe I need `{}` in `Circle` but not `startSketchOn`
- What is `%` for?
- Why does `mm()` have the `()` and why is it multiplying the number
- What is `[]` for when using `center`

In other cases there are also `$` to declare tags, function declarations which are noisey (e.g., `fn spoke = (spokeGap, spokeAngle, spokeThickness) => { ... }`) - why is `=` and `=>` needed? Why does this look different to the std lib docs (e.g., `angledLineOfXLength(data: AngledLineData, sketch: Sketch, tag?: TagDeclarator)` - note no `=`, `: type` on arguments, and note that the `{}` has different semantics to those in the snippet). Points which are a key primitive are sometimes declared as `{ x: 1, y: 0, z: 0 }` and sometimes as `[0, 1, 2]`, and so forth.

This doc proposes some small changes. Other related changes are suggested in other docs:

- Changes to pipeline syntax and removing `%`, in [pipelines and tags design doc](pipelines.md)
- Leading underscores (including just an underscore) are variables which can't be used, in [construction geometry design doc](show.md)
- Remove anonymous objects, in [records design doc](records.md)

## Function declarations

Remove `=` and `=>`. E.g., `fn spoke = (spokeGap, spokeAngle, spokeThickness) => { ... }` becomes `fn spoke(spokeGap, spokeAngle, spokeThickness) { ... }`.

See [type system](types.md) for adding type annotations. See [functions](functions.md) for changes to organising and calling functions.

## Field initialisation

Use `=` rather than `:`, e.g., `a = { x = "hello", bob = 42 }`.

## Point and vector types and constructors

We introduce `Point2D`, `Point3D`, `Vector2D`, and `Vector3D` objects for absolute and relative points in space, respectively.

The constructor functions `pt` and `vec` produce these objects, depending on the number of arguments. E.g., `pt(0, 0, 0)` produces an object `Point3D { x = 0, y = 0, z = 0 }` (see [functions](functions.md) for details). This facilitates access using `foo.x` or `foo['x']` (see [records design doc](records.md)), the latter useful for iterating over coordinates.

Where the type is unambiguous (see [type system](types.md)), the user can just write `(0, 0, 0)` and KCL will infer whether to use `pt` or `vec`. I'm not sure if this desugaring should be made extensible or limited to the point and vector types.

## Use constants instead of magic strings

E.g., `XY` instead of `'XY'`.

Remove use of magic strings such as `END` for identifying geometry (see [tagging](TODO) for a proposed alternative).

## Example

Repeating and combining the initial examples:

```
fn spoke = (spokeGap, spokeAngle, spokeThickness) => {
  ...
}

lugHoles = startSketchOn(lugBase, 'END')
  |> circle({
       center: [lugSpacing / 2, 0],
       radius: 16 * mm() / 2
     }, %)
  |> patternCircular2d({
       arcDegrees: 360,
       center: [0, 0],
       instances: lugCount,
       rotateDuplicates: true
     }, %)
  |> extrude(-wheelWidth / 20, %)
```

would become

```
fn spoke(spokeGap, spokeAngle, spokeThickness) {
  ...
}

lugHoles = startSketchOn(lugBase.faces.end)
  |> circle(
       center = (lugSpacing / 2, 0),
       radius = 16mm / 2,
     )
  |> patternCircular2d(
       arcDegrees = 360,
       center = (0, 0),
       instances = lugCount,
       rotateDuplicates = true
     )
  |> extrude(-wheelWidth / 20)
```

Notes

- `lugBase.faces.end` is a strawman for finding geometry, requires work
- `16mm` relies on work on units of measure, assuming mm is the default unit for the project, just `16` should work
- `center = (0, 0)` could also be written as `center = pt(0, 0)`
- fully annotated with types, the function might look like `fn spoke(spokeGap: Number, spokeAngle: Number, spokeThickness: Number): Solid` or `fn spoke(spokeGap: num, spokeAngle: num, spokeThickness: num): Solid` or `fn spoke(spokeGap: Number(1d), spokeAngle: Number(angle), spokeThickness: Number(1d)): Solid`, we wouldn't need any other type annotations in the language.

I believe this keeps the spirit and advantages of KCL while making the syntax friendlier and more attractive.
