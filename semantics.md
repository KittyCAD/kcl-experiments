# ZMA value semantics

This doc only concerns the semantics of geometry values. Primitive values (which includes strings, arrays, and records) can just have copy or reference semantics and there are no issues.

## Constraints

- Engine: the engine doesn't really have a concept of objects which exist without being rendered. Objects which are part of others are consumed and can't be reused. They still exist and have ids, but they can't be reused. Objects can be cloned (new feature not yet used in KCL).
  - I'm unsure whether an object can be cloned after it is used or if using an object mutates it any way. Specifically whether updating tags is purely a KCL-side thing or has effects in the engine.
- Tags: in KCL objects are mostly immutable but geometric actions can update tag info within objects and tag declarators (I'd love to change this, but not before 1.0). Even after an object is consumed, we often want to refer to it's tags (which may or may not have been altered by the consuming).
- Implicit reference semantics: some functions consume an object (e.g., `extrude`), some functions do not (e.g., `startSketchOn`). This is not indicated in the function definition or the types of the arguments, etc.
- Value implementation: we don't have a concept of references as values, so when an object is morally referenced (e.g., when assigned into a variable or when we keep a reference to the sketch that is the base of a solid) it is actually copied. In theory this is fine because an object is just an id (i.e., it is itself a reference to the object in the engine), however, any metadata is duplicated (which makes our current treatment of tags a mess, and would make using a dirty bit or similar ideas a mess too). This is also not great for performance since there is a lot of duplication in memory, but it does make GC (and other implementation issues) much easier.

## Problems

Fundamentally, the issue is that the semantics of KCL objects does not match the semantics of values in the engine.

### Unexpected lack of reuse

```
a = someSketch
a |> extrude(...)
```

You might expect to be able to reuse `a` as the base for another solid, but this only sort-of works (primarily tags will be buggy, but also if `a` is mutated (e.g., rotated) it will affect the first solid not just the second).

### Unexpected mutation

```
a = someSketch
b = a |> translate(...)
```

In this case one might expect `a` to be un-transformed, but it is in fact transformed and equal to `b`

### Behaviour of imported objects

```
import a from 'foo.kcl'
a |> extrude(...)
a |> extrude(...)
```

Currently the two uses of `a` are different, so this code works. However, this means imported variables have different semantics to locally-defined variables (bad). This also requires executing `foo::a` multiple times (which arguably is not actually an inefficiency, it's the same as calling a function).

If we were to fix the different semantics by treating an import as if the variable were declared inline, there'd be other issues (the above example wouldn't work, but that would be desirable). Imagine three files:

```
// a.kcl
x = someSketch

// b.kcl
import x from a.kcl
x

// c.kcl
import x from a.kcl
x
```

`x` would only render in one of those files and which one would depend on the order in which the files are evaluated (currently deterministic, but could easily not be in the future). Furthermore, if b.kcl didn't use `x`, but then was changed to use `x`, that would break `c.kcl` with no clue to any of the files involved that it might happen.


### Parallel execution of modules

For performance, it would be good if we could evaluate modules in parallel (and in general evaluate them out-of-order, including for caching). However, this requires evaluating geometry ahead of time rather than on-demand which requires a change of semantics or knowing in advance how many times names are used. Even if we change the semantics to match local vars (i.e., used exactly once), we still need to know if a variable is used in the file. E.g., if we use `import a from 'foo.kcl'` and `foo.kcl` also defines `b` we need to know not to evaluate `b` (doable today, but needs implementation). However, if we write `import a, b from 'foo.kcl'` but don't use `b`, we also shouldn't render it. Although this is arguably a user-error, we don't check for unused variables today. Furthermore, if the user wrote `import * from 'foo.kcl'` but only uses `a`, we still shouldn't render `b`, but there is no unused variable.


## Solutions

For both solutions I'm assuming that the semantics for imported names should match locally declared ones.

### Linear semantics

Every value can be used exactly once. Values have move semantics and are rendered in the last place they end up (this is a formalisation of today's semantics). Furthermore, accessing a consumed value only gives access to its metadata or identity (e.g., use as a tag or reference). Any other use is an error (new). We make an explicit clone operation as a standard library function (new, but trivial).

For submodules and imports, we would need the following rules to ensure no spooky action at a distance:

- non-primitive variables cannot be exported (use a function or a whole-module import style)
- if a module has top-level geometry it must be imported if any functions are imported. If the variable is not used, it is rendered at the import site.

To make these rules easier to understand I propose that we split modules into two kinds: parts and libraries (possibly with different extensions, e.g., .kcl and .kclib). Libraries can't have any top-level geometry (i.e., opening them as a file renders nothing, though we could relax this for testing/experimentation). Parts can't also export constants or functions. We could go further and use slightly different syntax: `import 'foo.kcl` is an expression which produces the top-level geometry rather than creating a variable `foo` as today, e.g., `import 'foo.kcl' |> extrude(...)` or `a = import 'foo.kcl'`. And `from 'foo.kclib' import a, b, c` to import `a`, `b`, and `c` from foo.kclib.


#### Implementation

The hard bit of this proposal is enforcing the affine property of geometry. I think the best way is to implement references (see below) and then replace consumed values in memory with a consumed version. Consuming a consumed version triggers an error. Alternatively, we could evaluate sub-expressions within a context (consuming or referencing, where the former rewrites the value in memory), but this is unsound in general because the value could be aliased before it is consumed.

We could decide that linear semantics is the semantics we want but that re-using a variable doesn't cause an error, it's just undefined behaviour (which is basically what we have today, informally). We'd do the best we could with documentation and errors in the easiest cases, and fix things post-1.0.

### Copy semantics

In this solution, assigning into a variable or passing to a function logically makes a copy of the geometry both in KCL and the engine. However, for performance and correctness (we don't want to create objects in the engine that the user only wants to use as input for a pipeline) we'd onlyclone when necessary (see below for what that means).

One question this brings up is when objects should be rendered, e.g.,

```
a = someSketch
a |> extrude(...)
```

Under the current and linear semantics, `a` is moved into the pipeline when it's extruded. Under the copy semantics, we would copy `a` into the pipeline so that it could be reused for another object. So should we render `a` and the solid or just the solid?

One approach is to not render a value if it is used for something else and render it if not (following current semantics). However, if the user wants to render both, they would have to clone the variable, even though they wouldn't otherwise need to clone it. Alternatively, we could only render objects which reach the outer scope of a program (or, easier, but supporting side effects for functions, any scope). So assigning into a variable would not render, but evaluating an expression outside a function without using it or assigning into a variable would render it (which makes sense because we don't currently support variable reuse, so the only time we wouldn't render is for an unused variable). In both cases, only the solid in the above example is rendered, in the first case the programmer would write `a.clone()` or `clone(a)` or `a |> clone()` (depending on how `clone` is supported), in the second case, they would write `a`.

If we adopt copy semantics, then we don't need to ban exporting constants (as we might for linear semantics). If we want to evaluate imports in advance, we might want to avoid evaluating more than is necessary for performance reasons (figuring out what is necessary is easy if we just look at the imported names, but requires static analysis if we want to take account of the use of imported names. Partial evaluation of modules is not currently possible and would be hard at the moment, but for caching and on-demand evaluation (as an optimisation for IDE features) we want this anyway in the long term).

#### Implementation

We must support references (see below); after creation, geometry is then passed by reference. As with the previous solution, when consumed we replace the value with a 'consumed' version. However, if the value is consumed again, rather than signalling an error, the object is cloned (assuming objects in the engine can be cloned like this). Essentially, a copy-on-write scheme. With the second sub-approach, an unconsumed value in a variable which goes out of scope is hidden in the engine, and if a consumed value reaches the top-level scope of a program, it is cloned and rendered. In the first sub-approach, neither check is necessary.

When an object is cloned, the KCL object is replaced with a new (unconsumed) one with the id of the new object. The old object is forgotten unless there is a snapshot in place, in which case the previous metadata and id is stored in the snapshot to preserve closure hygiene.

A question here is what happens to tags for cloned objects? Presumably only the most recent tags are available and the older tags are lost (first example below).

```
a = someSketch
a |> extrude(...)
a |> extrude(...)

a.tag // refers to tag from the second extrude

a = someSketch
a |> extrude(...)
b = a
b |> extrude(...)

a.tag // should refer to the first extrude
```

In the second example above, we don't clone `a` until `b` is consumed by the extrude (although in the first sub-approach we should also clone if the second extrude is elided we should still clone and render when b goes out of scope). However, we don't want the extrude to overwrite the tags in `a`. I believe we can accomplish this by cloning the KCL object when `a` is assigned to `b` but only cloning the object in the engine when `b` is extruded (note that nothing is cloned when `a` is extruded, so this is still a lazy clone).

I don't think there is a good partial solution with copy semantics - if we don't get it right it won't work. We could clone too much and it should just affect performance (hopefully not too much), but I'm not sure that makes anything easier.


## Opinion

I prefer the copy semantics and the second sub-approach of only rendering geometry if it reaches the top-level scope outside of a variable.

I believe this is the right thing to do, because it's roughly the same amount of implementation effort, but gives errors in much fewer cases, leads to more natural and flexible semantics with imports, and doesn't require the concept of library files. It will also be more natural to folk with a programming background.

Although it is a little 'interesting' to require geometry to hit the top-level to be rendered, I think this will cause fewer bugs with functions (functions are easier to reason about because you only have to consider the returned value, not any side effects) and means users never have to think about cloning.

The major downside of this option is that there is a subtle but likely widespread breaking change that `a = ...` does not render, and to fix that you'd either need to remove `a = ` or add `a`. This would be fairly easy to give a useful warning (and a suggested fix) for. Another downside, is there is no way to have a function which renders multiple objects unless we add aggregation functionality (I regard this as a feature rather than a bug, but I'm not sure if this is used in practice).

# Appendix: implementing references

## Design 1

As noted above, we need to support references to make either of the above schemes work. References are also useful for performance and to improve the error-prone code around tag updating.

To support references, we would add an `Address` (see below) struct and a `KclValue::Address` which holds that. `Address`es have a name (generated from a counter) which can be used to lookup the addressee value from memory (i.e., memory continues to be indexed by strings which can represent either a variable or address). When an object is created, it is stored in memory and the result is it's address. We never pass geometry (or functions) by value. When geometry is used within other geometry we use an address rather than copying the geometry. Addresses are normalised so that we never a 'pointer to a pointer', however, variables stored in memory map to addresses.

```rust
struct Address {
  name: String,          // String-ified auto-incremented usize,
  ty: GeometryType,      // A simple enum of `Solid`, `Sketch`, etc.
  env: EnvironmentRef,   // The snapshot where the addressee was added to memory.
}
```

We'd continue to snapshot memory (i.e., addresses and variables). When an object is mutated and the old version is saved to a snapshot, any addresses to the current environment in the object should be updated to the most recent snapshot, this requires walking all (transitive) references in the object and updating all snapshots in the current version of the object, effectively making the clone-on-write a deep clone). Furthermore if the parent snapshot of the address's snapshot has changed, then references to this environment must be updated too (and transitively any parents until an unchanged snapshot is discovered). Other snapshots do not need updating since their environments must have been popped from the stack and therefore be read-only.

The below example (assuming copy semantics, linear semantics would require an explicit clone and so is easier) works because `a` keeps the same address for all uses, but that address is snapshotted when `foo` is defined.

```
a = someSketch
a |> extrude(...)
fn foo() { consume(a) }
a |> extrude(...)
foo() // This call should still see the tags from the first extrude.
```

Note that in the `Address` definition there is an `EnvironmentRef`, this is used to lookup the address in memory since addresses might point to stack frames which have been popped (effectively there is no heap in our memory, only a stack where stack frames may be preserved after they are popped from the stack. Separating heap from stack in the future is desirable, but we would then need to implement some kind of garbage collection and there are complications around tags and closures which make that non-trivial).

The `EnvironmentRef` identifies an environment and a snapshot of that environment (taking an address requires taking a snapshot if there are none already for the current stack frame, but don't require a new snapshot). When looking up an address from the current state of the stack, the snapshot part is ignored which gives us the most recent version of the addressee. When reading at a point in time (i.e., using a snapshot), we use the snapshot part of the `EnvironmentRef` to get the value at that point in time. Copying an address does not modify its snapshot.

Since references might exist to any address in a stack frame we can't drop callee stack frames even if there are no functions declared in them (as we do today), unless the return value can be proved to have no local addresses (trivial for primitive types, difficult for most geometry, and in which case we can drop the frame even if there are snapshots, since any function references can't escape the function). We could remove any primitives from the stack frame.

### Footnote: WTF?

This is all ridiculously complicated. Why do we need all this nonsense?

Most of the complexity comes from the intersection with snapshots which are necessary due to the semantics of functions (which are closures). There are a few ways to make this easier:

- Functions get the latest version of all captured variables and can refer to variables not defined where the function is declared. This could be very surprising (especially in conjunction with linear semantics for geometry).
- Declaring a function could copy all of memory. We used to do this, it was very poorly performing.
- Declaring a function could copy just the variables it uses. This doesn't really work in a world with references plus we'd need some static analysis to derive the set of captured variables.
- We fix tags so they don't need to cause mutation (which I hate for numerous reasons, this being just one of them) and allow future references (which is a bit weird in general, but matches the behaviour of functions and types in most languages and makes sense in a constraint-solving-focussed world).

IMO the only good solution is the last one and that is a lot of work, but I hope we can do it post-1.0.

## Design 2

- `Address` and `KclValue::Address` are similar to above.
- Heap and stack design
  - Heap is usize-addressed, reference-counted (for GC), stores only geometry
  - Functions could be stored in the heap and used via address as an optimisation.
  - Stack is string-addressed, GCed by frame rather than individual object, stores only primitives and addresses held in local variables
- Re-implement snapshots and tags
  - Memory keeps a global epoch counter, incremented on every snapshot point in the current (and proposed references) design.
  - Function decls keep an env index and epoch (no snapshots).
  - Every stack value stores the epoch when it was created. When accessing the stack at a point in time, we check the epoch to see if the value exists
    - Since heap values can only be accessed via the stack, there is no need for a creation-time check
  - Tags (and ids where necessary) are versioned by epoch, rather than mutated (i.e., heap objects are monotonic but not immutable). Accessing tags requires checking the epoch.
  - Objects cannot be deleted in a way that respects the timeline, but I think that's OK because we only delete in order to fixup memory for mock execution, and in this case we don't care about past times
- For parallelising execution, incrementing the epoch count can be atomic (I don't think it can be per-thread because when we merge heaps, the epoch counts still need to make sense). While executing a module, it has it's own heap (as well as stack frames), then these are combined after execution (they can't be read until then, i.e., we only support parallelisation of non-dependent modules). Merging a module into the global state would require locking, but that is so rare that it'll be fine.
- For per-module caching, we would need to be able preserve or delete a module's heap as well as it's stack frames. Otherwise, I think things are OK. The epoch counter must be monotonic. Since re-evaluating a module would invalidate any modules which depend on it, and modules are always evaluated in dependency order, we'll never get epoch anomalies (as long as the counter is monotonic).


