# KCL 4.0 assembly blocks: implementation specification

## 1. Summary and compatibility requirements

Add the standard-library function `assembly()` and allow KCL functions to accept a trailing block:

```kcl
@settings(kclVersion = "4.0-preview")

result = assembly() {
  description = "Example assembly"
}
```

A call to `assembly` evaluates its arguments, opens the exports of `std::assemblies` in a new scope, and executes the supplied block there. Injected inputs retain their original `Solid` or `Assembly` type during block execution. The call recursively wraps solids and nested assemblies in its named members as opaque `Component` values only during successful finalization, before returning the opaque `Assembly`. Other values remain ordinary members.

The implementation adds three opaque KCL types and matching `KclValue` variants:

- `Block`: deferred KCL code with its captured lexical environment.
- `Assembly`: a collection of named members with assembly identity and ownership.
- `Component`: a read-only wrapper for a solid or nested assembly in a finalized assembly.

Define all three KCL types in `rust/kcl-lib/std/types.kcl`. Name the module `std::assemblies` to avoid a collision with `assembly`. It must initially export no functions. Assembly blocks must automatically see any KCL functions the module exports in the future.

This document describes work still to be implemented.

### 1.1 Version boundary

Here, "KCL 4" means `4.0-preview` or later. The preview must support the feature. Use the existing version representation and a boundary equivalent to `KclVersion::V4Preview`. Do not require a final 4.0 release or a separate experimental opt-in.

Use the effective language version of the entry point. Keep existing module-version compatibility checks and import mismatch diagnostics. Preserve older versions' per-module behavior.

For all versions before KCL 4:

- Preserve existing parsing, evaluation, name resolution, and sketch-block behavior.
- Do not expose the new standard-library function, module, or types.
- Do not reserve `assembly`, `assemblies`, `Block`, `Assembly`, `Component`, or `block` as new keywords.
- Keep user-defined identifiers with those names valid in older programs.
- Do not apply new ownership checks or change existing geometry identity or consumption behavior.
- Preserve the serialized representation and digest of call expressions without a trailing block.
- Preserve valid legacy syntax where an independent object expression follows a call. Do not reinterpret it as a trailing block.

Keep the dedicated sketch-block AST representation and current sketch semantics in every language version.

### 1.2 Boundaries

Assemblies use the current execution mode. Do not use them to run sketch mode or add assembly APIs to `frontend.rs`. Do not add an assembly UI editing mode, scene grouping, engine artifact kinds, or frontend execution entry points.

This implementation sends no assembly engine message. Add a no-op finalization hook with a TODO for that future message. Code inside an assembly uses the existing engine execution and batching rules.

KCL code can receive, store, return, and forward a `Block`, but cannot execute one directly in this version. Rust standard-library implementations can execute blocks through a shared internal block executor.

Leave assembly cloning, mating constraints, constraint wire formats, and a general static type checker for future work. Keep the structure needed for future static analysis.

## 2. Language and standard-library contract

### 2.1 General trailing block arguments

Extend ordinary function calls to accept one optional trailing block:

```kcl
result = someFunction(argument = value) {
  local = 42
}
```

Pass the trailing block as a value under the argument name `block`. Passing it does not execute the body. Ordinary KCL functions, imported functions, aliases, and Rust standard-library functions can receive it if they accept that argument.

A KCL wrapper can forward the block:

```kcl
@settings(kclVersion = "4.0-preview")

fn buildAssembly(block: Block): Assembly {
  return assembly(block = block)
}

result = buildAssembly() {
  answer = 42
}
```

Forwarding preserves the caller's lexical environment. It does not add the wrapper's parameters or locals to the capture.

| Situation | Required behavior |
| --- | --- |
| A call has a trailing block | Supply a captured `Block` as the argument named `block`. |
| A call supplies `block = existingBlock` | Use ordinary argument binding and type validation. |
| A call supplies both an explicit `block` argument and a trailing block | Reject the duplicate argument. |
| The called function cannot accept `block` | Report the normal invalid-argument error; do not silently discard it. |
| A function accepts a block but does not use it | Do not execute the block. |
| A function stores or returns the block | Keep the captured lexical environment and source context alive. |
| The same block is evaluated more than once by Rust code | Give each evaluation a fresh local scope while retaining its lexical capture. |
| KCL attempts `block()` | Reject it because `Block` is not a callable function. |
| A block contains a `return` belonging to that block | Reject it in the initial implementation. |
| A function declared inside the block contains `return` | Preserve ordinary function-return behavior. |
| KCL contains a bare `{ ... }` expression | Preserve object-expression syntax; it does not create a `Block`. |

Keep the existing eager argument evaluation order. Capture a syntactic trailing block after evaluating the other arguments and before entering the callee. Capturing the block must not execute statements, move geometry, or recursively inspect values in the captured environment.

A trailing block adds no parameters or parameter-declaration syntax. Its free names resolve through the captured caller environment and the scopes the Rust block executor supplies.

Only `assembly` supplies an empty block when the caller omits `block`. Other functions keep the usual rules for required and optional parameters.

Do not change the semantics of `return` in sketch blocks.

Add a TODO to revisit `return` semantics in trailing blocks. Do not give it nonlocal-return semantics in this implementation.

### 2.2 Assembly function

Declare the function in the standard-library prelude using the repository's conventions for Rust-implemented functions:

```kcl
@(impl = std_rust, added_in = "4.0", feature_tree = true)
export fn assembly(
  components?: {},
  block?: Block,
): Assembly {}
```

The `components` parameter uses the existing object type. Validate its property values in Rust. No new typed-dictionary syntax is needed.

Use these defaults for `assembly`:

- Missing `components` means an empty object.
- Missing `block` means an empty block evaluated in the caller's context.
- Both may be omitted, so `assembly()` returns an empty assembly.
- `assembly(components = { ... })` is valid without a trailing block.
- An explicitly supplied `block` must be a `Block`.

Use ordinary KCL call syntax and existing optional-argument conventions. Do not add special positional arguments for assemblies.

Each property of `components` must be a `Solid`, `Assembly`, or array of `Solid` or `Assembly`. Reject other values, including a `Component`, a sketch, an array of other values, or an arbitrary nested object. The object maps names directly to transferable values that keep their input types during body execution.

For example:

```kcl
empty = assembly()

group = assembly(components = {
  first = firstBody,
  second = secondBody
})

namedGroup = assembly(components = { first = anotherBody }) {
  label = "Named group"
}
```

Passing a component as a new input cannot detach it from its owning assembly. To get an independent, movable solid from a solid-backed component, use `clone`.

### 2.3 Types and module

Add these declarations to `types.kcl`, using existing conventions for opaque Rust-implemented types:

```kcl
@(impl = std_rust, added_in = "4.0")
export type Block

@(impl = std_rust, added_in = "4.0")
export type Assembly

@(impl = std_rust, added_in = "4.0")
export type Component
```

Register the types in the existing standard type system, runtime type representation, coercion, and type reporting. These are named opaque types. Keep them out of the keyword set.

Add `rust/kcl-lib/std/assemblies.kcl`. It may contain module documentation and required annotations, but must initially export no KCL functions. Register it through the normal standard-library module loader.

Expose the module and new declarations only in KCL 4. Keep the usual explicit-import behavior in that version. Open the module's actual exports inside assemblies so future exported KCL functions need no hard-coded list.

### 2.4 Scope construction and returned members

Use this scope structure, from outermost to innermost:

1. The block's captured caller environment.
2. A scope containing the exports opened from `std::assemblies`.
3. The assembly's body scope, containing injected input values and declarations made directly in the body.

Open `std::assemblies` when executing the block. Only sketch blocks automatically open `std::solver`.

This gives `std::assemblies` exports the usual opened-module precedence over captured outer names. Body declarations can shadow outer names. Injected inputs and direct body declarations share a scope, so duplicate-definition rules apply.

Move and inject each input before executing the first body statement. Its property name becomes its local name. A raw `Solid` remains a `Solid`, and an `Assembly` remains an `Assembly`. An injected solid supports ordinary solid operations during construction. An injected assembly retains its already finalized members, including their existing component protection.

Wrap solids and nested assemblies only when finalizing the returned member hierarchy, after block execution and before `assembly()` returns an `Assembly`.

The returned assembly contains:

- Every injected input, even if the block did not reference it.
- Every value binding declared directly in the body, including numbers, strings, functions, sketches, arrays, objects, solids, and assemblies.

It does not automatically contain:

- Names visible only through the captured caller environment.
- Exports introduced by opening `std::assemblies`.
- Locals belonging to called functions or nested scopes.
- Internal runtime bindings used to evaluate the block.

Keep existing import restrictions, including top-level-only imports. An assembly block does not create a new import context.

Collect member names from direct body declarations and input properties. Keep them even if the geometry is consumed or no longer live, or the block never uses the value. This preserves the name set for future static analysis.

At finalization, recursively convert member values:

| Value during construction | Value exposed through the returned assembly |
| --- | --- |
| `Solid` | A `Component` targeting that solid, preserving its live or consumed state. |
| `Assembly` | A `Component` targeting that nested assembly. |
| `Component` | The existing component with validated ownership and identity, without an additional wrapper. |
| Object | An object with the same keys and recursively finalized values. |
| Array | An array with recursively finalized elements. |
| Any other value | The ordinary value, retaining its existing behavior. |

Keep an array's original element-type descriptor during block execution. When finalization changes a homogeneous array's elements from `Solid` or `Assembly` to `Component`, update its element-type descriptor too.

A declaration of `count = 2` exposes a number at `result.count`. A declaration of `pair = { left = makeBody(), right = makeBody() }` exposes components at `result.pair.left` and `result.pair.right`.

### 2.5 Member access, read-only components, and cloning

Support ordinary dot access:

```kcl
result.part
result.label
result.subassembly.part
```

Member access on an `Assembly` reads its named member map. Member access on an assembly-backed `Component` delegates to the nested assembly's members while retaining ownership checks.

A solid-backed `Component` allows inspection of the solid and its geometry metadata, including applicable face, edge, tag, and sketch access. Do not expose an unrestricted raw `Solid` through this inspection.

Components are read-only:

- Allow inspection and read-only geometry queries.
- Allow cloning a solid-backed component into a new independent `Solid`.
- Allow component geometry references to be used to construct independent geometry.
- Reject operations that mutate, consume, transform, delete, or otherwise modify the component's existing body.
- Reject appearance changes or other engine operations that modify the existing body.
- Reject merge or similar construction modes that would modify a component body through a face, edge, sketch, or other derived reference.

Injected raw solids and locally created raw solids remain mutable and consumable until finalization wraps them as components. Moving an existing assembly into a block preserves the read-only protection of its existing components.

Do not make `Component` a general subtype of `Solid`, and do not globally unwrap it during coercion. A user-defined parameter typed `Solid` continues to reject a component. It accepts an injected raw `Solid` during construction.

Non-solid geometry, such as an exported sketch, keeps its normal modeling behavior. Exporting it adds no read-only restriction. It still cannot bypass protection of a component's body.

Extend `clone` to accept a solid-backed component and return an independent raw `Solid`. Preserve existing clone argument and collection behavior. Reject cloning an `Assembly` or an assembly-backed `Component` in this implementation.

Add a TODO for future assembly cloning.

### 2.6 Complete example

```kcl
@settings(kclVersion = "4.0-preview")

fn makeCylinder(): Solid {
  profile = sketch(on = XY) {
    rim = circle(start = [5mm, 0mm], center = [0mm, 0mm])
  }

  return extrude(
    region(segments = [profile.rim]),
    length = 10mm
  )
}

fn copyBody(body: Solid): Solid {
  return clone(body)
}

seed = makeCylinder()

machine = assembly(components = { first = seed }) {
  label = "Two cylinders"
  second = copyBody(body = first)
}

independentCopy = clone(machine.first)
```

During the block, `first` is a raw `Solid`, so it can be passed to `copyBody`'s `Solid` parameter. The helper returns an independent raw `Solid` for `second`. Both values become components only during finalization.

After the call:

- `seed` has been moved and cannot be used as a live geometry handle.
- `machine.first` and `machine.second` are solid-backed components.
- `machine.label` is an ordinary string.
- `independentCopy` is an independent, mutable `Solid`.

## 3. Ownership, validation, and execution semantics

### 3.1 Input transfer

Validate the complete input object before transferring any input or running any body statement.

Preflight validation must confirm that:

1. Every property has an accepted input type.
2. Every input handle is valid and has not already been moved.
3. Every geometry value being transferred is live.
4. An input assembly contains no consumed geometry in its public member hierarchy.
5. No live body or assembly identity occurs more than once across the inputs.

Once preflight passes, move the inputs to the new assembly's construction owner and inject new valid handles with the same `Solid` or `Assembly` types as the inputs. This transfer changes ownership and handle validity while preserving the value's type. Defer component wrapping until finalization. Do not clone geometry or send an engine command to perform this logical move.

A move invalidates the original value and every alias of its old ownership handle before the body executes. Captured aliases become invalid too.

Moving a whole assembly also invalidates previously saved handles to its descendants. Reissue valid handles under the new assembly hierarchy. The injected value remains an `Assembly`, and its existing component members remain read-only.

For example:

```kcl
saved = child.part
parent = assembly(components = { child = child })
```

After the move, `child` and `saved` are invalid for geometry access. Access through `parent.child.part` is valid. Previously copied scalar metadata remains valid.

Do not rewrite closures that captured an old handle to refer to the new owner.

### 3.2 External geometry boundary

Code inside an assembly may access captured non-geometry values normally. It must not implicitly use engine-backed geometry from outside that assembly.

Transfer external geometry through `components`. This parameter accepts only solids and assemblies. Create other engine-backed geometry inside the assembly or access it through an explicitly transferred assembly.

Apply the boundary to all engine-backed geometry and references, including sketches, regions, faces, edges, imported geometry, and references derived from an external engine body.

Check runtime provenance and ownership to enforce this rule. Do not rely on identifier names or a syntax scan of the block.

| Case | Required behavior |
| --- | --- |
| Read a captured number, string, function, or other non-geometry value | Allow it. |
| Read an external solid directly or through an alias | Reject it unless it has been transferred and is accessed through a valid injected handle. |
| Call a helper that accesses captured external geometry | Reject the geometry access inside the helper. |
| Call a helper that creates fresh geometry during this assembly's execution | Allow it; the fresh geometry belongs to the active assembly construction. |
| Access geometry stored at module top level | Treat it as external, including when the module is initialized lazily during the call. |
| Call an imported factory that creates fresh geometry | Allow the newly created geometry under the active assembly boundary. |
| Select a scalar field from an external object that also contains geometry | Allow the scalar projection. |
| Select or pass the geometry-bearing field of that object | Reject the external geometry. |
| Pass, bind, or otherwise use the entire geometry-bearing container | Recursively validate it and reject the external geometry. |
| Capture a function or block whose environment contains geometry | Do not eagerly scan the environment; validate geometry when the deferred code accesses it. |

For example, `config.width` is valid if it produces a scalar, even when `config.body` is an external solid. Reject evaluation of `config.body` or use of the whole `config` object in the block's computation.

Abstract axes, standard planes, and similar values that do not identify externally owned engine resources remain usable.

Assign module provenance during module initialization, even if an assembly is active. Otherwise lazy initialization could misclassify external geometry as local.

Apply the same rules in mock execution and engine-backed execution.

### 3.3 Construction values and final transfer

Injected raw solids and raw solids created during block execution remain `Solid` values and stay mutable throughout construction. The block may transform, combine, or consume them with existing modeling operations. Injected and locally created nested assemblies remain `Assembly` values during construction; their existing components stay read-only.

On successful finalization, transfer injected and locally created live solids and nested assemblies in the returned member hierarchy into their final ownership positions and expose them through components. Apply recursive wrapping only after body execution and before returning the `Assembly`. Preserve consumed members as described in section 3.4.

Invalidate construction-time handles to transferred solids and nested assemblies, including aliases in closures or other escaped values. A nested assembly transfer also invalidates saved descendant handles. Do not rewrite captured lexical environments to point at the new component handles.

A closure that captured a scalar continues to work. A closure that later tries to use a raw solid handle invalidated by finalization receives the normal moved-handle error.

Non-solid exported geometry retains ordinary modeling behavior, subject to ownership validity and the rule against indirectly modifying a component body.

### 3.4 Consumed inputs and locals remain members

Do not drop consumed named bodies from the result.

For example, a block that consumes raw solids `left` and `right` in a union and binds the result as `joined` still returns all three names as assembly members. This applies whether `left` and `right` were injected inputs or declared directly in the body.

At finalization, `left` and `right` become components that retain the consumed state of their original values. Attempts to use them as geometry fail. Wrapping a consumed value must not make it live or assign it a new live identity.

Ignore consumed values when checking for duplicate live bodies. Retaining a consumed input alongside its live result does not create a duplicate by itself.

Reject a later attempt to move an assembly that contains consumed geometry anywhere in its public member hierarchy. Include the offending member path in the diagnostic.

### 3.5 Duplicate policy

Reject duplicate live geometry in two places:

1. Across the complete `components` input, before any transfers or body execution.
2. Across the prepared output member hierarchy, before committing finalization.

Traverse public members recursively through objects, arrays, and nested assemblies. Two different names or paths must not expose the same live engine body, even if the values have different wrapper identities or ownership generations.

Also reject repeated assembly identity, including repeated references to an empty assembly.

Examples include:

- Two input properties containing aliases of the same solid.
- A direct solid input and a nested assembly input containing that same live body.
- Two body-local names pointing at the same live solid.
- A body-local alias of an injected solid or assembly, when both names would appear in the output.
- Duplicate live geometry buried in separate object properties or array elements.

Independent clones have distinct bodies and are valid.

When checking public members for duplicates, skip private solid fields such as stored sketch or topology data. Skip closure environments too. Neither adds public assembly members.

Consumption is the only liveness-based exclusion from the duplicate-body check. It is not a reason to remove a member from the result.

Keep traversal, identity extraction, and the duplicate rejection policy in separate helpers. This lets the policy change later without rewriting assembly execution.

Duplicate diagnostics should report both conflicting public paths in deterministic traversal order.

### 3.6 Engine behavior and failure handling

Run body code through the existing engine command path. Keep normal batching and command order. An assembly adds no restriction on engine commands.

After successful evaluation, member preparation, and output validation, invoke the no-op engine finalization hook. Run it before final ownership transfers and component wrapping.

Add a TODO at this hook for the future engine message. Future assembly functions may collect mating constraints during the block and send that information once at the end. Do not add constraint accumulation, a placeholder wire schema, or a fake engine command in this implementation.

Do not allocate an engine object ID for the assembly, force an extra engine flush, or change command visibility for this hook.

If argument validation, body evaluation, ownership validation, or finalization fails:

- Return the appropriate existing execution error or exit condition.
- Restore lexical scopes, source context, and the active assembly ownership boundary.
- Do not return a partially constructed assembly.
- Skip the engine finalization hook if an earlier phase fails.
- Do not cache the failed evaluation as a successful result.

Keep existing execution failure semantics. Do not add transactional rollback for engine commands or ownership transfers completed before a later failure.

### 3.7 Interaction with sketches and nesting

Allow nested assemblies, assemblies inside sketch blocks, and sketch blocks inside assemblies, subject to existing sketch restrictions.

Assemblies must inherit the current execution context.

Apply these rules:

- An assembly called inside a sketch can see solver names already present in the captured sketch scope.
- It still automatically opens only `std::assemblies`.
- Existing sketch replay restrictions on engine commands continue to apply.
- An assembly must not grant permission to send engine commands during a sketch execution phase that normally forbids them.
- An assembly between two sketch blocks must not bypass the existing prohibition on nested active sketches.
- Existing sketch-mode execution and cache behavior must not acquire an assembly-specific mode or entry point.

Extend value traversal and solver substitution to resolve solver values inside assembly members, components, objects, and arrays.

## 4. Implementation architecture

### 4.1 Existing integration points and separation of concerns

The relevant implementation is in `rust/kcl-lib`.

| Concern | Existing integration points |
| --- | --- |
| Parsing and AST | `src/parsing/parser.rs`, including `fn_call_or_sketch_block`; `CallExpressionKw`, `SketchBlock`, and `Block` in `src/parsing/ast/types/mod.rs`. |
| Call evaluation | Argument handling in `src/execution/fn_call.rs` and `FunctionSource::call_setup`. |
| Executor | `src/execution/machine.rs`. |
| Lexical capture | Existing `EnvironmentRef`, environment snapshots/epochs, and `push_new_env_for_call` behavior. |
| Opened module scope | Existing module-export loading. Use `load_sketch2_into_current_scope` as a reference for opening exports only. |
| Standard library | `std/prelude.kcl`, `std/types.kcl`, and registration in `src/std/mod.rs`. |
| Runtime types and values | `KclValue`, `PrimitiveType`, standard type resolution, coercion, and value traversal. |
| Geometry identity | `src/execution/geometry.rs` and existing solid/region consumption tracking. |
| Tooling and serialization | AST walkers, node paths, recasting, LSP, CodeMirror grammar, generated bindings, `KclValueView`, and `OpKclValue`. |

Separate generic block capture and execution from assembly construction. Keep component ownership, assembly engine hooks, and sketch mode out of the generic code. Keep the assembly standard-library adapter small, with lifecycle and ownership rules in shared helpers.

### 4.2 AST, parser, formatting, and source paths

Extend the ordinary keyword-call AST with an optional trailing block:

```rust
pub block: Option<Node<Block>>,
```

Use serde defaults and omit the field when absent. Serialize calls without a block exactly as before to preserve compatibility with existing AST consumers. Make the field optional in generated TypeScript types too.

Do not add `Expr::AssemblyBlock`. Preserve the existing `SketchBlock` representation and sketch parser branch.

Update constructors and synthesized call expressions to initialize the new field to `None`. Include any trailing block in source ranges and digests. If no block is present, leave the existing digest unchanged.

Support traversal of the block and its body items for:

- AST walking and rewriting.
- Rename, replacement, and identifier lookup.
- Source-range lookup and node-path generation.
- Recasting and formatting.
- Comment and non-code metadata preservation.
- LSP and editor tooling that descend through expressions.

Give the call's trailing block and its body items their own node-path steps. When using existing argument iterators, preserve the distinction between the deferred block and eagerly evaluated expression arguments.

The parser needs the effective version before deciding whether a brace following a call is a trailing block. The current flow discovers settings after the main parse. Add a token-aware settings prepass or another method that discovers the version early.

The early version discovery must:

- Keep existing settings-annotation semantics. The last applicable setting wins.
- Respect comments and strings.
- Handle settings that appear after other top-level syntax.
- Preserve malformed-annotation diagnostics through the normal parser.
- Use the entry-point version for imported-module parsing while preserving existing import mismatch rules.
- Avoid changing old valid call-followed-by-object programs.

Also check the feature's version boundary when executing externally supplied or deserialized ASTs. Parser checks do not cover those paths.

Keep ordinary conditional bodies unchanged. In an unparenthesized condition such as `if predicate() { ... }`, the brace starts the conditional body. Require parentheses around a condition that includes a trailing block, for example `if (predicate() { value = 1 }) { ... }`.

Preserve the existing special sketch syntax and its parsing behavior.

### 4.3 Runtime data and opaque types

Add runtime representations that store this information:

| Runtime value | Required information |
| --- | --- |
| `Block` | Shared block AST, captured environment reference and snapshot/epoch, and source/module context. |
| `Assembly` | Internal assembly identity, ownership/generation information, named member map, and normal metadata. |
| `Component` | Component handle identity/generation, owning assembly, a target that is either a `Solid` or an `Assembly`, and normal metadata. |

The assembly builder must retain each input's original `Solid` or `Assembly` type in its body-scope bindings. Create new component wrappers only during finalization of the returned member hierarchy. Nested assemblies may already contain components from their own finalization.

Use shared ownership where needed to keep captured environments and nested assemblies alive. Do not copy a large AST each time a block is forwarded.

Add matching `KclValue` variants and runtime type cases. Update all exhaustive matches and consumers for type reporting, coercion errors, metadata and source extraction, environment remapping, serialization views, operation recording, solver-value substitution, and recursive value walking.

Keep identity, ownership, and target fields private. Expose assemblies to KCL through their named members and components through controlled inspection.

Blocks stored in objects, arrays, functions, or assembly members must retain their captured environments after the original function call returns.

Do not introduce structural coercions between these opaque types and ordinary objects, or new structural equality semantics for them.

### 4.4 Generic block capture and execution

For ordinary calls with a trailing block:

1. Resolve the callee using existing rules.
2. Evaluate ordinary arguments in their existing order.
3. Capture the trailing block in the caller's current lexical context.
4. Supply it as the labeled `block` argument.
5. Use normal signature validation and parameter binding.
6. Execute the callee normally.

Provide a shared internal Rust block executor that accepts a captured `Block`, extra scope layers, and bindings from its caller. Return the final body-local bindings or the existing error or exit outcome.

The executor must restore all pushed scopes and source/module context on every exit path. Block evaluation has no implicit function return value. The `assembly` function uses the resulting local bindings.

When native standard-library code executes a user-supplied block, temporarily restore the block's captured user-source context. Do not let an `inside_stdlib` flag suppress user diagnostics or grant standard-library privileges to the user's code. Restore the surrounding native-call context afterward.

Keep assembly ownership checks and finalization outside the generic block executor.

### 4.5 Ownership and geometry provenance

Track ownership and provenance across the whole execution, including module-local state swaps. Module-local consumption state alone is not enough when assemblies call across modules or transfer nested assemblies.

Track:

- A geometry resource's identity and creation origin.
- Whether a handle generation is still valid.
- Its current owner.
- Whether a solid body is protected as a read-only component.
- Existing consumption state.
- Source information for useful errors.

A copied alias retains the same access token or generation. A transfer issues valid handles for the new owner and invalidates the old generation. Transferring an assembly must invalidate saved descendant handles as well as the outer handle.

Track ownership separately from component protection. Moving a raw solid into the construction owner preserves its type and mutability. Finalization protects the body when wrapping it as a component. Moving an existing assembly preserves the protection of its already finalized components.

Register provenance when trusted code creates geometry. Do not assign local ownership just because the executor encounters a value for the first time. That could misclassify external geometry as local.

Assign geometry that a helper creates during assembly construction to the active assembly construction owner. Use module provenance for module initialization. Each nested assembly has its own ownership boundary.

Track this ownership boundary in runtime bookkeeping. It adds no execution mode.

Validate the value selected by a projection. Reading `config.width` must not fail because `config` also contains geometry. When an object or array is used as a whole value, validate it recursively.

Check geometry access through name lookup, member access, function arguments, native coercion and adapters, imported values, and other access paths. Do not scan a function's or block's captured environment when the value is accessed. Check actual geometry accesses when it runs.

Check whether a mutation is allowed before queuing an operation that would modify a protected body. Do not reject an already admitted construction command just because it flushes after the body becomes a component.

Integrate with existing consumed-solid and consumed-region behavior without replacing their identities or changing earlier-version semantics. Keep the new ownership behavior gated to KCL 4.

### 4.6 Assembly lifecycle

Implement the assembly lifecycle as follows:

1. Verify the feature gate and validate ordinary arguments.
2. Substitute an empty components object and empty block where omitted.
3. Preflight all input types, handle validity, consumed descendants, and duplicate identities.
4. Allocate an internal assembly identity; do not allocate an engine object.
5. Prepare the captured lexical context and the scope opened from `std::assemblies`.
6. Transfer inputs to the construction owner and bind new valid handles with their original `Solid` or `Assembly` types into the body scope.
7. Establish the assembly construction ownership boundary.
8. Execute the block.
9. Collect injected input names and directly declared body-local value names.
10. Prepare recursive finalization of the collected values, planning component wrappers for solids and nested assemblies from both inputs and body declarations. Keep consumed members.
11. Validate the complete output for duplicate live identities and unauthorized external geometry.
12. Invoke the no-op engine finalization hook. Future engine messages in this hook may translate or rotate subassemblies and bodies, so it must run before wrapping them in read-only `Component` values.
13. Commit final ownership transfers for injected and locally produced live solids and nested assemblies, create the prepared component wrappers, and update affected array element-type descriptors.
14. Restore the surrounding execution context and return the `Assembly`.

Centralize cleanup so every error or early exit restores the same context. Complete all input preflight checks before the first move.

Keep output validation separate from transfer commits. A duplicate or invalid output must not leave the returned hierarchy partially finalized.

### 4.7 Executor integration

Execute assembly bodies and finalize assemblies through the executor's existing continuation mechanism for resumable standard-library calls. Run blocks on the CESK machine's existing run loop. Do not start a separate nested CESK machine or synchronously recurse through arbitrary nested block calls. Nested block execution must not grow the native stack.

Retain the assembly builder and pending finalization state across suspension. Restore them after normal completion, error, or exit.

Preserve:

- Argument evaluation and capture order.
- Member names and values, including original input types during construction and component wrapping only at finalization.
- Ownership and identity behavior.
- Engine command and operation ordering.
- Validation and failure timing.
- Scope and boundary cleanup.

### 4.8 Native operations and component adapters

Do not add a general `Component -> Solid` conversion. Give supported native operations an explicit, narrow input policy, such as read-only solid access or solid cloning.

Injected raw solids use existing solid signatures and operations during construction. Component adapters apply to existing components in nested assemblies and to component values exposed by finalized results.

Initial integration includes:

- `isSolid` and `isSurface` for applicable solid-backed components.
- `faceOf` and `planeOf`.
- `faceId` and `edgeId`.
- `clone`.

For functions that accept a solid directly, allow a solid-backed `Component` in the effective KCL 4 input type only when the operation's policy permits it. Apply this rule to the supported array-element case for cloning too.

Queries that accept references still need ownership checks and must carry protected-body lineage into their results. They do not need unrelated signature changes.

Adapters may obtain a private read-only view of the target solid. They must not return an unrestricted raw solid to KCL. Derived references must retain enough information to reject later operations that would mutate or consume the protected body.

Reject assembly-backed components in solid-only operations. For `clone`, report that assembly cloning is unsupported.

Older language versions resolve standard-library signatures eagerly. Unconditionally naming `Component` in a shared declaration can therefore break an older program before it calls the function. Keep written signatures resolvable in older versions. Widen accepted types by version through native-function metadata or the existing equivalent signature mechanism.

Use the same metadata to validate calls and generate displayed and documented KCL 4 signatures. For older versions, exclude new `added_in = "4.0"` declarations before resolving their parameter types.

### 4.9 Tooling, value views, caches, and documentation

Update Rust and TypeScript AST consumers and generated bindings for the optional call block. Format, highlight, traverse, and diagnose general trailing blocks as ordinary call syntax. Keep them separate from sketch blocks.

Sketch selection and source traversal must still reach sketches nested inside a general block where appropriate. Keep assembly APIs out of `frontend.rs`.

Extend value views and operation-value representations:

- `Block`: an opaque marker or summary, without serializing ASTs or captured environments.
- `Assembly`: a view of its named public members.
- `Component`: its target kind and inspectable value, without exposing mutable raw-handle access or private ownership state.

During construction, value views must retain each injected value's `Solid` or `Assembly` type.

Update all generated union consumers to handle the new variants. Keep ordinary solid operations and their existing engine/artifact behavior. Do not create a new engine scene or artifact kind for an assembly.

Keep component protection, ownership, and provenance state consistent with runtime memory across caching and resumption. Store this bookkeeping with the relevant runtime state or invalidate incompatible cached state. Discarding it must never remove component protection or make a stale cached handle valid.

Preserve existing sketch cache behavior and ensure recursive solver-value substitution handles new value containers. Prevent ownership state or new-version values from leaking into earlier-version executions.

Add language and standard-library documentation with examples and diagnostics. Generate standard-library documentation and bindings through the repository workflow. Do not edit generated output by hand.

## 5. Verification, implementation order, and explicit defaults

### 5.1 Acceptance scenarios

Run language-level scenarios against the executor. Add engine-backed tests where real geometry identity, command validation, or derived-reference behavior matters.

| Area | Required scenarios |
| --- | --- |
| Version boundary | Default version, 1.x, 2.x, 3.0-preview, and 3.x preserve prior behavior; 4.0-preview enables the feature. Keep the feature boundary suitable for later 4.x versions. |
| Earlier-version names | User-defined names matching the new function, module, and types remain valid where they were valid before. |
| Legacy parsing | Preserve old call-followed-by-object parsing, ordinary conditional bodies, comments, and settings appearing after other syntax. |
| Generic block syntax | Empty and nonempty blocks; comments; multiline calls; qualified calls; function aliases; calls in pipelines; parenthesized condition calls; source ranges and recast round trips. |
| Argument binding | Fixed `block` parameter, explicit forwarding, duplicate explicit-plus-trailing block, wrong block type, and a callee that does not accept the argument. |
| Deferred behavior | Ignored block never executes; stored and returned blocks retain lexical capture; repeated execution creates fresh local bindings. |
| Return and exits | Direct block return is rejected; nested function return works; body errors and exits restore state and skip the engine finalization hook. |
| Defaults | Empty `assembly()`, components-only calls, block-only calls, and explicit block forwarding; other functions receive no implicit empty block. |
| Scope | Module export precedence, injected input names, same-scope duplicate definitions, local shadowing of outer names, and no captured/module/helper-local leakage into members. |
| Result membership | Numbers, strings, functions, sketches, objects, arrays, raw solids, nested assemblies, and consumed values retain their declared names; recursive conversion updates array typing only at finalization. |
| Input validation | Reject unsupported property types and components passed as inputs; validate the complete object before any move or body side effect. |
| Input movement | Old input handles and all aliases are invalid before the first body statement; injected values have new valid handles with their original `Solid` or `Assembly` types. |
| Construction types | Injected solids satisfy `Solid` parameters and support normal transforms, combinations, and consumption; injected assemblies satisfy `Assembly` parameters and retain protected component members. |
| Finalization | Run the engine finalization hook after output validation and before final ownership transfers and wrapping. Both injected and locally created solids and nested assemblies become components only then, before returning; construction-time handles captured in closures become invalid. |
| Descendant movement | Moving an assembly invalidates a previously saved child handle; the new hierarchy remains usable; copied scalar metadata remains valid. |
| Consumed input assemblies | Moving an assembly with consumed geometry is rejected with the member path. |
| External captures | Reject direct, aliased, imported, container-held, and helper-accessed external geometry; allow fresh helper-created geometry. |
| Module provenance | Lazy top-level module geometry remains external; imported factory-created geometry belongs to the active construction. |
| Mixed objects | Allow a scalar projection from an object that also contains external geometry; reject the geometry projection or whole-container use. |
| Component inspection | Permit supported solid queries and member access without exposing a mutable raw solid. |
| Component protection | After wrapping, reject mutators, transforms, consuming operations, casts that would unwrap a solid, and indirect mutation through derived faces, edges, sketches, or merge operations; existing components inside injected assemblies stay protected throughout construction. |
| Protection before move invalidation | Raw aliases retained after wrapping cannot mutate or consume a component's body; derived references and cached values retain the same protection. |
| Independent construction | Allow construction of new independent geometry using permitted references to a component. |
| Clone | Cloning a solid component yields an independent mutable solid; preserve collection behavior; reject assembly cloning. |
| Other geometry | Exported non-solid geometry keeps ordinary modeling behavior and cannot bypass component protection. |
| Duplicate inputs and outputs | Reject direct aliases, body-local aliases of injected solids or assemblies, nested-object/array duplicates, cross-subassembly duplicates, and repeated assembly identity; allow independent clones. |
| Consumed members | When the block consumes injected or locally created solids, retain their names as consumed components; fail when used as live geometry and ignore them only for duplicate-live-body checks. |
| Nesting | Nested assemblies, assemblies in sketches, and sketches in assemblies work under existing restrictions; an assembly does not bypass active nested-sketch checks. |
| Sketch replay | No new engine permissions, frontend mode, or cache behavior; solver substitution traverses new value containers. |
| Engine commands | Ordinary commands still execute; finalization sends no new message and adds no mandatory flush; legitimate pending construction commands on injected or locally created solids can flush after finalization. |
| Failure and caching | Setup, body, and finalization errors unwind correctly; failures are not cached as success; stale generations and version changes cannot revive handles. |
| Tooling and views | Formatting, comments, node paths, hover/signatures, highlighting, generated bindings, and value views handle new variants without serializing private environments or ownership fields. |
| Deep execution | The CESK machine handles deep nesting through continuations and cleans up state without growing the native stack per nesting level. |

Test opening future `std::assemblies` exports and resolving name collisions with test-only module fixtures. Keep the production module empty.

Use mock execution for language and ownership cases that do not require an engine. Use engine simulations to test actual moves and identity, cloning, rejection of mutations through derived references, and sketch/assembly interaction.

Every new feature example and fixture must opt into `4.0-preview`. Use earlier-version KCL fixtures only for compatibility tests.

### 5.2 Suggested implementation units

Build the `4.0-preview` prototype through unit 4 before implementing the full ownership registry and validation:

1. Add general blocks, including version-aware parsing, the optional AST field, lexical capture and forwarding, the `Block` type, generic evaluation support, traversal, and formatting.
2. Add opaque `Assembly` and `Component` values, with runtime type integration, member access, views, and generated bindings.
3. Add assembly standard-library support with the declaration, empty `std::assemblies` module, shared lifecycle that preserves input types and defers component wrapping to finalization, executor integration, and no-op engine hook. Validate arguments and input types.
4. Add narrow read-only component adapters, solid-component cloning, and mutation guards for raw aliases and derived references. Enforce protection independently of handle invalidation.
5. Implement ownership rules for provenance, generation validation, transfers, descendant invalidation, external-geometry checks, consumption integration, and helpers for duplicate detection. Add preflight and output validation at the lifecycle points in section 4.6.
6. Finish integration and compatibility work for nested sketch behavior, caching, documentation, tooling, and regression coverage.

Add tests with each unit. Complete all units and the full acceptance matrix before stable `4.0`. During preview, programs accepted only because validation is missing remain outside the contract in sections 2 through 4 and may be rejected as checks are added. The goal is to get to a minimally-working implementation in fewer implementation steps, and then add checks necessary for 4.0 stabilization after that.

Keep logically separate changes reviewable. Do not alter CI configuration to make the feature pass.

### 5.3 Verification commands and workflow

During implementation:

- Format Rust using `cargo +nightly fmt`.
- Run `cargo clippy --all-targets`.
- Run focused KCL parser, execution, standard-library, ownership, and tooling tests.
- Run feature tests with the repository's existing test commands.
- Run relevant engine-backed simulation tests where credentials and infrastructure are available.
- Regenerate bindings with `npm run bindings:update` when the Rust interfaces change.
- Build the required WASM artifacts before testing application integration that depends on them.
- Run affected CodeMirror, LSP, AST/recast, and value-view checks.
- Regenerate standard-library documentation through the repository workflow.

Use unchanged earlier-version tests and snapshots to check compatibility. Do not change historical snapshots to hide changed semantics.

Follow repository Rust error-handling conventions. Return errors for invalid states. Avoid panics and `unreachable!()`. Keep Rust comments ASCII.

### 5.4 Explicit defaults and deferred work

These decisions are settled for this implementation:

- The module is `std::assemblies`; the function is `assembly`; the input parameter is `components`.
- The generic block parameter name is `block`.
- General blocks use lexical capture and an opaque `Block` value.
- Only Rust code executes a block; no general KCL block-evaluation API is added.
- Existing `SketchBlock` AST nodes remain separate.
- Assembly members are accessed with ordinary dot syntax.
- Every injected input and directly declared body-local value becomes a member.
- Injected inputs retain their original `Solid` or `Assembly` types during block execution; raw solids remain mutable and consumable during construction.
- Solids and nested assemblies become components only during finalization, before returning the `Assembly`.
- Objects and arrays are recursively converted during finalization where they contain solids or assemblies; affected array element-type descriptors are updated then.
- Consumption does not remove member names.
- Live-body duplication and repeated assembly identity are errors, with deterministic member paths.
- Components are read-only, including existing components inside injected assemblies; other exported geometry retains normal modeling behavior.
- A whole-assembly move invalidates saved descendant handles.
- Captured environments are not rewritten after ownership transfers.
- Imports remain subject to the existing top-level-only rule.
- Ordinary failures and engine batching retain existing semantics.
- No assembly execution mode, frontend API, scene grouping, or engine message is added.

Leave TODOs at these implementation points:

1. Revisit return-value and `return` semantics for general trailing blocks.
2. Implement cloning of assemblies and assembly-backed components.
3. Accumulate future mating constraints and send the new engine message from the finalization hook once the engine API is available.

The feature is complete when the executor implements these semantics and earlier KCL versions retain their behavior. The finalization hook must run after output validation and before final ownership transfers and component wrapping, without sending a new engine message.
