# Implementation/low-level design notes on sketch 2.0 (KCL interpreter parts)

## New language features

All of these will need parsing, checking, execution.

- `var` syntax (maybe use postfix `?`)
- `sketch` blocks
  - insignificant ordering (nothing to implement as such, but a difference to normal blocks)
  - return/export (need to decide on design)
  - which expressions are valid/invalid inside/outside `sketch` blocks
- `==` constraint expressions
- new attributes on std functions and types
- spread sugar for array input arguments (I think this just needs implementing, I don't think there's anything notable, but likely some fiddly interactions with coercions)


## Attributes and constraint function semantics

New options for the `impl` attribute (only valid inside std):

- `std_rust_constrainable` valid on functions and types.
- `std_rust_constraint` valid on functions only.

I expect the geometry functions (`impl = std_rust_constrainable`) are straightforward KCL returning the appropriate types. The types probably are Rust impls but we can and should write them in KCL too for documentation and ensure that the 'double declaration' works. The constraint functions will probably be defined in Rust (although possibly they are KCL which return constraint objects which the interpreter knows about, rather than functions which the interpreter knows about).

Rules for calling functions:

- Function calls from sketch blocks are allowed (subject to following rules), which is a change to the spec.
- No engine calls from sketch blocks (however indirect).
- Definitions:
  - a type is *var* if it is a number literal declared using `var` syntax or a type which has a *var* component (e.g., `[4, var 5]` is *var* because it contains `var 5`).
  - a type is *constrainable* if it is a number, an array of numbers, or a record declared with `@(impl = std_rust_constrainable)`.
- Only *constrainable* types may be *var*.
- *var* types can only be initialised lexically within a `sketch` block.
- *var* types can only be passed to a function if the function is *constrainable* and only from a `sketch` block.
- functions with `@(impl = std_rust_constraint)` can only be called from within a `sketch` block.
- arithmetic with non-*var* types is allowed within `sketch` blocks, but not with *var* types.

Some implications of the above rules:

- constrainable functions can be used outside of a sketch block, but can't be constrained (we probably should just error to start with rather than put the work in to make them work with 1.0 sketches, but we can do this if there is demand)
- regular functions can be called from sketch blocks but can always be safely evaluated before solving and can't interact with solving (likewise arithmetic)
- `var` data can't escape from a sketch block, but the concrete (fully solved) types can (required to access values and tags of geometry within sketches)

## Evaluating sketch blocks

- `var` values are desugared into variables, e.g., `x = 4?; line(start = [0?, x])` is desugared to `$0 = 4?; $1 = 0?; line(start = [$1, $0])`. These variables will become solver input variables and are never substituted with the values (i.e., they are a new kind of `KclValue`).
- expressions are evaluated. Non-*var* expressions can be fully evaluated. Functions will all be evaluated down to datatypes (which may include *var* data) and constraints, see below for equivalence expressions.
  - during evaluation, the above rules for *var*-ness are checked
- we then have a bunch of variables, data, and constraints which can be fed into the solver


## Equivalence expressions

`a == b`

E.g., `distance(p1, p2) == distance(p2, p3)` or `distance(p1, p2) == 54` or `distance(p1, p2) == 4?`, where `fn distance(@input: [Point2d; 2]): number(Length)`.

The logical return type of `distance` (and similar constraints) is a number which will be *var*.

Equivalence expressions are desugared into constraints. If both expressions are non-*var* then the equivalence is desugared to an assertion. If one side is *var*, then the non-*var* side will be fully evaluated and the *var* side converted into a fixed constraint, e.g. `distance(p1, p2) == 54` becomes a fixed distance constraint. If both sides are *var* then if the sub-expressions are compatible, then they will be lowered to a fixed or equal constraint. E.g., `distance(p1, p2) == distance(p2, p3)` becomes `equalDistance(p1, p2, p2, p3)` and `distance(p1, p2) == 4?` becomes `$0 = 4?; fixedDistance(p1, p2, $0)`. Incompatible sub-expressions cause an error, e.g., `angle(...) == distance(...)` or `angle(...) == 45?` (angles can only be constrained to non-*var* numbers). Note that compatibility is not just type-driven, it depends on the actual constraints used.
