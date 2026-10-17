# KCL 4.0 assembly blocks: implementation specification

## 1. Summary and compatibility requirements

Introduce an ordinary standard-library function, `assembly()`, and a general facility for passing a trailing block to a KCL function:

```kcl
@settings(kclVersion = "4.0-preview")

result = assembly() {
  description = "Example assembly"
}
```

The assembly function evaluates its arguments, creates a scope with the exports of `std::assemblies` opened, and executes the supplied block in that scope. It returns an opaque `Assembly` value containing its named members. Named solids and nested assemblies become opaque `Component` values. Other values remain available as ordinary members.

The implementation introduces three opaque KCL types and corresponding `KclValue` variants:

- `Block`: deferred KCL code with its captured lexical environment.
- `Assembly`: a collection of named members with assembly identity and ownership.
- `Component`: a solid or nested assembly owned by an assembly.

Define all three KCL types in `rust/kcl-lib/std/types.kcl`. The new module is named `std::assemblies`, plural, so it does not collide with the function named `assembly`. It starts without any exported functions. Future exported KCL functions in this module must become available automatically inside assembly blocks.

This document specifies future implementation work. Saving this document does not implement the feature or change the codebase.

### 1.1 Version boundary

For this feature, “KCL 4” means `4.0-preview` or later. The preview is a prerelease of version 4.0 and must receive the feature. Use the existing version representation and a boundary equivalent to `KclVersion::V4Preview`; do not require a final 4.0 release or a separate experimental opt-in.

Use the effective entry-point language version for this feature. Preserve existing module-version compatibility checks and import mismatch diagnostics. Do not change historical per-module behavior for older language versions as an incidental part of this work.

For all versions before KCL 4:

- Preserve existing parsing, evaluation, name resolution, and sketch-block behavior.
- Do not expose the new standard-library function, module, or types.
- Do not reserve `assembly`, `assemblies`, `Block`, `Assembly`, `Component`, or `block` as new keywords.
- Preserve the ability of old programs to define ordinary identifiers with those names.
- Do not apply new ownership checks or change existing geometry identity or consumption behavior.
- Preserve the serialized representation and digest of call expressions without a trailing block.
- Preserve valid legacy syntax where a call is followed by an independent object expression; do not reinterpret it as a trailing block.

Existing sketch blocks keep their dedicated AST representation and their current semantics, including in earlier language versions.

### 1.2 Boundaries

Assemblies do not introduce an execution mode. They do not power sketch-mode execution, get new APIs in `frontend.rs`, or create a new UI editing mode. Do not add assembly-specific scene grouping, engine artifact kinds, or frontend execution entry points.

There is no assembly engine message in this implementation. Add a no-op finalization hook with a TODO for the future engine message. Ordinary engine commands issued by code inside an assembly continue to run under the existing execution and batching rules.

General KCL code can receive, store, return, and forward a `Block`, but cannot directly execute one in this version. Rust standard-library implementations can execute blocks through a shared internal facility.

Do not implement assembly cloning, mating constraints, constraint wire formats, or a general static type checker. Preserve enough structure for future static analysis without trying to implement those future features.

## 2. Language and standard-library contract

### 2.1 General trailing block arguments

Extend ordinary function calls to accept one optional trailing block:

```kcl
result = someFunction(argument = value) {
  local = 42
}
```

The trailing block is passed as the argument named `block`. It is a value, not an instruction to execute the body immediately. This applies to ordinary KCL functions, imported functions, aliases, and Rust standard-library functions that accept that argument.

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

The forwarded block retains the caller's lexical environment. It does not capture the wrapper's parameters or locals merely because the wrapper passes it to another function.

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

Use existing eager argument evaluation order. Capture a syntactic trailing block after evaluating the ordinary arguments and before entering the callee. Capturing the block must not execute statements, move geometry, or recursively inspect values in its captured environment.

A trailing block does not add parameters or a new parameter-declaration syntax. Its free names resolve using the captured caller environment and the scopes supplied by the Rust block executor.

Only `assembly` implicitly substitutes an empty block when no block argument is provided. Other functions retain ordinary required and optional parameter behavior.

Add an explicit TODO to revisit the semantics of `return` in trailing blocks. Do not give it nonlocal-return semantics in this implementation.

### 2.2 Assembly function

Declare the function in the standard-library prelude, following the repository's existing Rust-implemented function conventions:

```kcl
@(impl = std_rust, added_in = "4.0", feature_tree = false)
export fn assembly(
  components?: {},
  block?: Block,
): Assembly {}
```

The `components` parameter uses the existing object type. Validate the accepted property values in Rust; this feature does not require a new typed-dictionary syntax.

Defaults are specific to this function:

- Missing `components` means an empty object.
- Missing `block` means an empty block evaluated in the caller's context.
- Both may be omitted, so `assembly()` returns an empty assembly.
- `assembly(components = { ... })` is valid without a trailing block.
- An explicitly supplied `block` must be a `Block`.

Use ordinary KCL call syntax and existing optional-argument conventions. Do not add special positional arguments for assemblies.

Each property of `components` must be either a raw `Solid` or an `Assembly`. Reject other values, including a `Component`, a sketch, an array, or an arbitrary nested object. The input object itself maps names directly to transferable values.

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

A component cannot be detached from its owning assembly by passing it as a new input. To obtain a movable independent solid from a solid component, use `clone`.

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

Register the types through the existing standard type machinery, runtime type representation, and coercion/type-reporting paths. They are ordinary named opaque types, not language keywords.

Add `rust/kcl-lib/std/assemblies.kcl`. It may contain module documentation and required annotations, but must initially export no KCL functions. Register it through the normal standard-library module loader.

Expose the module and new declarations only in KCL 4. Preserve ordinary explicit-import behavior in that version. Automatically opening the module inside an assembly must use its actual exports, so adding a future exported KCL function does not require another hard-coded list.

Do not automatically open `std::solver`. Solver names already available in the captured lexical environment remain available normally.

### 2.4 Scope construction and returned members

Use this scope structure, from outermost to innermost:

1. The block's captured caller environment.
2. A scope containing the exports opened from `std::assemblies`.
3. The assembly's body scope, containing injected input components and declarations made directly in the body.

This gives assembly-module exports their normal opened-module precedence over outer captured names. Body-local declarations can shadow names in outer scopes. An injected input component and a body declaration with the same name are in the same scope, so ordinary duplicate-definition rules apply.

Move and inject each input before executing the first body statement. Its property name becomes the local name, and its local value is immediately a `Component`, even when the input was a raw `Solid`.

The returned assembly contains:

- Every injected input, even if the block did not reference it.
- Every value variable declared directly in the body, including numbers, strings, functions, sketches, arrays, objects, solids, and assemblies.

It does not automatically contain:

- Names merely visible from the captured caller environment.
- Exports introduced by opening `std::assemblies`.
- Locals belonging to called functions or nested scopes.
- Internal runtime bindings used to evaluate the block.

Preserve existing restrictions on imports, including the current top-level-only rule. An assembly block does not create a new import context.

Determine membership from the directly declared names and input property names. Do not remove members based on geometry liveness, consumption, or whether a value was used. This preserves the name set for future static analysis.

Recursively finalize member values as follows:

| Value | Value exposed through the assembly |
| --- | --- |
| `Solid` | A `Component` targeting that solid, preserving its live or consumed state. |
| `Assembly` | A `Component` targeting that nested assembly. |
| `Component` | A component with validated ownership and identity. |
| Object | An object with the same keys and recursively finalized values. |
| Array | An array with recursively finalized elements. |
| Any other value | The ordinary value, retaining its existing behavior. |

Update homogeneous-array element typing when conversion changes the elements from `Solid` or `Assembly` to `Component`. Do not retain a stale element-type descriptor.

For example, if a block declares `count = 2` and `pair = { left = makeBody(), right = makeBody() }`, the result exposes a number at `result.count` and components at `result.pair.left` and `result.pair.right`.

### 2.5 Member access, read-only components, and cloning

Support ordinary dot access:

```kcl
result.part
result.label
result.subassembly.part
```

Member access on an `Assembly` reads its named member map. Member access on an assembly-backed `Component` delegates to the nested assembly's members while retaining ownership checks.

A solid-backed `Component` permits inspection of the solid and its existing geometry metadata, including applicable face, edge, tag, and sketch access. It must not expose an escape hatch that returns an unrestricted raw `Solid`.

Components are read-only:

- Allow inspection and read-only geometry queries.
- Allow cloning a solid-backed component into a new independent `Solid`.
- Allow references to component geometry to be used to construct independent geometry.
- Reject operations that mutate, consume, transform, delete, or otherwise modify the component's existing body.
- Reject appearance changes or other engine operations that modify the existing body.
- Reject merge or similar construction modes that would modify a component body through a face, edge, sketch, or other derived reference.

Do not make `Component` a general subtype of `Solid`, and do not globally unwrap it during coercion. A user-defined parameter typed `Solid` continues to reject a component.

Non-solid geometry exported from the block, such as a sketch, keeps its ordinary modeling behavior. Exporting a sketch does not make it read-only. It still cannot be used to bypass protection of a component's underlying body.

Extend `clone` to accept a solid-backed component and return an independent raw `Solid`. Preserve existing clone argument and collection behavior. Reject cloning an `Assembly` or an assembly-backed `Component` in this implementation.

Add an explicit TODO for future assembly cloning.

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

seed = makeCylinder()

machine = assembly(components = { first = seed }) {
  label = "Two cylinders"
  second = makeCylinder()
}

independentCopy = clone(machine.first)
```

After the call:

- `seed` has been moved and cannot be used as a live geometry handle.
- `machine.first` and `machine.second` are solid-backed components.
- `machine.label` is an ordinary string.
- `independentCopy` is an independent, mutable `Solid`.

## 3. Ownership, validation, and execution semantics

### 3.1 Input transfer

Validate the complete input object before transferring any input or running any body statement.

Preflight validation must establish that:

1. Every property has an accepted input type.
2. Every input handle is valid and has not already been moved.
3. Every geometry value being transferred is live.
4. An input assembly contains no consumed geometry in its public member hierarchy.
5. No live body or assembly identity occurs more than once across the inputs.

Once preflight succeeds, move the inputs into the new assembly and inject their component handles. Do not clone geometry or send an engine command to perform this logical move.

A move invalidates the original value and every alias of its old ownership handle. It takes effect before the body executes, so capturing an old alias does not provide continued access.

Moving a whole assembly also invalidates previously saved handles to its descendants. Reissue valid handles under the new assembly hierarchy.

For example:

```kcl
saved = child.part
parent = assembly(components = { child = child })
```

After the move, `child` and `saved` are invalid for geometry access. Access through `parent.child.part` is valid. Previously copied scalar metadata remains an ordinary value and does not become invalid.

A closure that captured an old handle is not rewritten to refer to the new owner.

### 3.2 External geometry boundary

Code executing inside an assembly may access captured non-geometry values normally. It may not implicitly use engine-backed geometry originating outside that assembly.

External geometry must enter through an explicit `components` transfer. Because the input accepts only solids and assemblies, other engine-backed geometry must be created inside the assembly or be available through an explicitly transferred assembly.

Apply the boundary to all engine-backed geometry and references, not just `Solid`. This includes sketches, regions, faces, edges, imported geometry, and references derived from an external engine body.

Enforce the rule using runtime provenance and ownership, not identifier spelling or a syntactic scan of the block.

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

For example, `config.width` is valid if it produces a scalar, even when `config.body` is an external solid. Evaluating `config.body` or passing the whole `config` object into the block's computation must fail.

Abstract axes, standard planes, and similar values that do not identify externally owned engine resources remain usable.

Module initialization must have module provenance even if it runs while an assembly is active. Otherwise a lazily initialized module could accidentally make external geometry appear locally created.

Apply the same rules in mock execution and engine-backed execution.

### 3.3 Local geometry and final transfer

Raw solids created during block execution remain ordinary mutable solids while construction is in progress. The block may transform them, combine them, or consume them through existing modeling operations.

At successful finalization, transfer the locally created geometry represented in the returned member hierarchy into the assembly and expose it through components.

Invalidate old raw handles to transferred geometry, including aliases retained in closures or other escaped values. Do not rewrite captured lexical environments to point at the new component handles.

A closure that captured a scalar continues to work. A closure that later tries to use a raw solid handle invalidated by finalization receives the normal moved-handle error.

Non-solid exported geometry retains ordinary modeling behavior, subject to ownership validity and the rule against indirectly modifying a component body.

### 3.4 Consumed locals remain members

Do not drop consumed named bodies from the result.

For example, if a body declares `left` and `right`, consumes them in a union, and binds the result as `joined`, all three names remain members of the returned assembly.

The `left` and `right` components retain the consumed state of their original values. Attempting to use them as geometry fails. Wrapping a consumed value must not resurrect it or assign it a new live identity.

Ignore consumed values when checking for duplicate live bodies. Retaining a consumed input alongside its live result does not itself create a duplicate.

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
- A body-local alias of an injected component, when both names would appear in the output.
- Duplicate live geometry buried in separate object properties or array elements.

Independent clones are distinct bodies and are valid.

Do not recurse into private solid implementation fields, such as the sketch or topology data stored inside a solid, for membership-based duplicate detection. Do not traverse closure environments. Those are not additional public assembly members.

Consumption is the only liveness-based exclusion from the duplicate-body check. It is not a reason to remove a member from the result.

Keep traversal, identity extraction, and the duplicate rejection policy in separate helpers. This makes it possible to change the policy later without rewriting assembly execution.

Duplicate diagnostics should report both conflicting public paths in deterministic traversal order.

### 3.6 Engine behavior and failure handling

Body execution uses the existing engine command path. Preserve normal command batching and ordering. An assembly is not an engine-command prohibition boundary.

After successful evaluation, member preparation, validation, and ownership finalization, invoke a no-op assembly finalization hook before returning the `Assembly`.

Place an explicit TODO at this hook for the future engine message. Future assembly functions may accumulate mating constraints during the block and send the accumulated information once at the end. Do not implement that accumulation, a placeholder wire schema, or a fake engine command now.

Do not allocate an engine object ID merely to represent the assembly, force an extra engine flush, or change command visibility for this placeholder.

If argument validation, body evaluation, ownership validation, or finalization fails:

- Return the appropriate existing execution error or exit condition.
- Restore lexical scopes, source context, and the active assembly ownership boundary.
- Do not return a partially constructed assembly.
- Do not call the successful-finalization hook.
- Do not cache the failed evaluation as a successful result.

This feature does not add transactional rollback of engine commands or ownership transfers that already occurred before a later failure. Preserve existing execution failure semantics.

### 3.7 Interaction with sketches and nesting

Allow nested assemblies, assemblies inside sketch blocks, and sketch blocks inside assemblies, subject to existing sketch restrictions.

Assembly execution must inherit the current execution context rather than establish a new execution mode.

Consequences:

- An assembly called inside a sketch can see solver names already present in the captured sketch scope.
- It still automatically opens only `std::assemblies`.
- Existing sketch replay restrictions on engine commands continue to apply.
- An assembly must not grant permission to send engine commands during a sketch execution phase that normally forbids them.
- Putting an assembly between two sketch blocks must not bypass the existing prohibition on nested active sketches.
- Existing sketch-mode execution and cache behavior must not acquire an assembly-specific mode or entry point.

Extend existing value traversal and solver-substitution paths so solver values contained in assembly members, components, objects, or arrays are resolved correctly.

## 4. Implementation architecture

### 4.1 Existing integration points and separation of concerns

The relevant implementation is in `rust/kcl-lib`.

| Concern | Existing integration points |
| --- | --- |
| Parsing and AST | `src/parsing/parser.rs`, including `fn_call_or_sketch_block`; `CallExpressionKw`, `SketchBlock`, and `Block` in `src/parsing/ast/types/mod.rs`. |
| Call evaluation | `Node<CallExpressionKw>::execute`, argument handling, and `FunctionSource::call_setup`. |
| Evaluators | `src/execution/exec_ast.rs` and `src/execution/machine.rs`. |
| Lexical capture | Existing `EnvironmentRef`, environment snapshots/epochs, and `push_new_env_for_call` behavior. |
| Opened module scope | Existing module-export loading; `load_sketch2_into_current_scope` is a reference for opening exports, not a template for assembly execution mode. |
| Standard library | `std/prelude.kcl`, `std/types.kcl`, and registration in `src/std/mod.rs`. |
| Runtime types and values | `KclValue`, `PrimitiveType`, standard type resolution, coercion, and value traversal. |
| Geometry identity | `src/execution/geometry.rs` and existing solid/region consumption tracking. |
| Tooling and serialization | AST walkers, node paths, recasting, LSP, CodeMirror grammar, generated bindings, `KclValueView`, and `OpKclValue`. |

Separate generic block capture/execution from assembly construction. The generic layer must not know about component ownership, assembly engine hooks, or sketch mode. Keep the assembly standard-library adapter small and put the lifecycle and ownership rules in shared implementation helpers.

### 4.2 AST, parser, formatting, and source paths

Extend the ordinary keyword-call AST with an optional trailing block:

```rust
pub block: Option<Node<Block>>,
```

Use serde defaults and omission when absent. Preserve the existing serialized representation of calls without a block, including compatibility with existing AST consumers. Update generated TypeScript types to reflect an optional field.

Do not add `Expr::AssemblyBlock`. Preserve the existing `SketchBlock` representation and sketch parser branch.

Update constructors and synthesized call expressions to initialize the new field to `None`. Include a present block in source ranges and digests; an absent block must contribute nothing new to the historical digest.

Support traversal of the block and its body items for:

- AST walking and rewriting.
- Rename, replacement, and identifier lookup.
- Source-range lookup and node-path generation.
- Recasting and formatting.
- Comment and non-code metadata preservation.
- LSP and editor tooling that descend through expressions.

Add explicit node-path steps for the call's trailing block and its body items. Do not pretend the block is an ordinary eagerly evaluated expression argument when using existing argument iterators.

The parser must know the effective version before deciding whether a brace following a call is a trailing block. The existing parsing flow discovers settings after ordinary parsing, so add a token-aware settings prepass or equivalent early version-discovery facility.

The early version discovery must:

- Use the existing settings-annotation semantics, including the last applicable setting winning.
- Respect comments and strings.
- Handle settings that appear after other top-level syntax.
- Preserve malformed-annotation diagnostics through the normal parser.
- Use the entry-point version for imported-module parsing while preserving existing import mismatch rules.
- Avoid changing old valid call-followed-by-object programs.

Also enforce the feature boundary when executing an externally supplied or deserialized AST. Parser gating alone is insufficient.

Resolve conditional-expression ambiguity without changing ordinary condition bodies. In an unparenthesized condition such as `if predicate() { ... }`, the brace remains the conditional body. A trailing block in the condition must be explicitly parenthesized, for example `if (predicate() { value = 1 }) { ... }`.

Preserve the existing special sketch syntax and its parsing behavior.

### 4.3 Runtime data and opaque types

Introduce runtime representations with these responsibilities:

| Runtime value | Required information |
| --- | --- |
| `Block` | Shared block AST, captured environment reference and snapshot/epoch, and source/module context. |
| `Assembly` | Internal assembly identity, ownership/generation information, named member map, and normal metadata. |
| `Component` | Component handle identity/generation, owning assembly, a target that is either a `Solid` or an `Assembly`, and normal metadata. |

Use shared ownership where needed to keep captured environments and nested assemblies alive. Do not copy large ASTs on every forwarding operation.

Add corresponding `KclValue` variants and runtime type cases. Cover all exhaustive consumers, including type reporting, coercion errors, metadata/source extraction, environment remapping, serialization views, operation recording, solver-value substitution, and recursive value walking.

Opaque types do not expose the private identity, ownership, or target fields to KCL. Assembly access goes through the named-member interface; component access goes through controlled inspection.

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

Provide a shared internal Rust facility for executing a captured `Block` with caller-supplied additional scope layers and bindings. It must return the completed body-local bindings or the existing error/exit outcome.

The executor must restore all pushed scopes and source/module context on every exit path. A generic block evaluation is not a function call with an implicit return value; assembly uses the resulting locals.

When native standard-library code executes a user-supplied block, temporarily restore the block's captured user-source context. Do not let an `inside_stdlib` flag suppress user diagnostics or grant standard-library privileges to the user's code. Restore the surrounding native-call context afterward.

Keep assembly-specific ownership admission and finalization outside this generic facility.

### 4.5 Ownership and geometry provenance

Add execution-wide ownership/provenance bookkeeping that survives module-local state swaps. Existing module-local consumption state alone is not enough because assemblies can call functions across modules and transfer nested assemblies.

Track the information needed to establish:

- A geometry resource's identity and creation origin.
- Whether a handle generation is still valid.
- Its current owner.
- Whether a solid body is protected as a read-only component.
- Existing consumption state.
- Source information for useful errors.

A copied alias retains the same access token or generation. A transfer issues valid handles for the new owner and invalidates the old generation. Transferring an assembly must invalidate saved descendant handles as well as the outer handle.

Register provenance at trusted geometry creation points. Do not assign local ownership the first time a geometry value is observed, because that would allow external values to acquire local status accidentally.

Creation inside an ordinary helper called during an assembly uses the active assembly construction owner. Module initialization uses module provenance. A nested assembly establishes its own ownership boundary.

This ownership boundary is runtime bookkeeping, not a new execution mode.

Enforcement must be projection-aware. Do not reject `config.width` merely because reading the object to perform the projection temporarily touches an object containing geometry. Validate the value selected by the projection. Validate entire objects and arrays recursively when they are used as values.

Cover name lookup, member access, function arguments, native coercion/adapters, imported values, and other routes through which geometry can be accessed. A function or block value does not trigger recursive scanning of its closure environment; actual geometry access during its execution does.

Enforce mutation admission before queuing the operation that would modify a protected body. Do not reject a legitimate already-admitted construction command merely because a later flush occurs after the body has been finalized into a component.

Integrate with existing consumed-solid and consumed-region behavior without replacing their identities or changing earlier-version semantics. Keep the new ownership behavior gated to KCL 4.

### 4.6 Assembly lifecycle

Implement the same lifecycle for both evaluators:

1. Verify the feature gate and validate ordinary arguments.
2. Substitute an empty components object and empty block where omitted.
3. Preflight all input types, handle validity, consumed descendants, and duplicate identities.
4. Allocate an internal assembly identity; do not allocate an engine object.
5. Prepare the captured lexical context and the scope opened from `std::assemblies`.
6. Transfer inputs and bind their component values into the body scope.
7. Establish the assembly construction ownership boundary.
8. Execute the block.
9. Collect injected input names and directly declared body-local value names.
10. Prepare recursive solid/assembly-to-component conversion without removing consumed members.
11. Validate the complete output for duplicate live identities and unauthorized external geometry.
12. Commit transfers of locally produced live solids and nested assemblies to the returned hierarchy.
13. Invoke the no-op engine finalization hook.
14. Restore the surrounding execution context and return the `Assembly`.

Centralize cleanup so every error or early exit unwinds the same context correctly. Complete all input preflight checks before the first move.

Keep the output-validation stage separate from the transfer-commit stage so a duplicate or invalid output does not partially finalize the returned hierarchy.

### 4.7 Recursive and machine evaluators

Implement feature parity in both the recursive evaluator and the stack-machine evaluator.

The recursive evaluator can use the shared block executor within its existing recursion constraints.

The machine evaluator must use its existing continuation/resumable-standard-library mechanism for assembly body execution and finalization. Do not start an independent nested machine or synchronously recurse through arbitrary nested block calls in a way that defeats the machine's bounded native stack.

The machine must retain the assembly builder and pending finalization state across suspension. Restore them correctly after normal completion, error, or exit.

Both backends must agree on:

- Argument evaluation and capture order.
- Member names and values.
- Ownership and identity behavior.
- Engine command and operation ordering.
- Validation and failure timing.
- Scope and boundary cleanup.

### 4.8 Native operations and component adapters

Do not add a general `Component -> Solid` conversion. Instead, give explicitly supported native operations a narrow input policy, such as read-only solid access or solid cloning.

Initial integration includes:

- `isSolid` and `isSurface` for applicable solid-backed components.
- `faceOf` and `planeOf`.
- `faceId` and `edgeId`.
- `clone`.

For functions accepting a solid directly, extend the effective KCL 4 accepted input type to include a solid-backed `Component` only when the operation's policy permits it. Include the corresponding supported array-element case for cloning.

Reference-taking queries do not need unrelated signature changes. They do need ownership validation and propagation of protected-body lineage through their results.

Adapters may obtain a private read-only view of the target solid. They must not return an unrestricted raw solid to KCL. Derived references must retain enough information to reject later operations that would mutate or consume the protected body.

An assembly-backed component is not a solid. Reject it in solid-only operations. Reject its use with `clone` with the explicit unsupported-assembly-cloning behavior.

Account for eager standard-library signature resolution in older language versions: changing a shared declaration to reference `Component` unconditionally can break older programs before they call the function. Preserve old-version-resolvable written signatures and apply version-aware effective widening through the native-function metadata or equivalent existing signature mechanism.

Use the same metadata for actual validation and displayed/documented KCL 4 signatures. New `added_in = "4.0"` declarations must be excluded for older versions before their parameter types are resolved.

### 4.9 Tooling, value views, caches, and documentation

Update Rust and TypeScript AST consumers and generated bindings for the optional call block. General trailing blocks must be formatted, highlighted, traversed, and diagnosed as ordinary call syntax; do not classify them as sketch blocks.

Existing sketch-selection and source traversal must still reach sketches nested inside a general block where appropriate. This does not authorize adding assembly APIs to `frontend.rs`.

Extend value views and operation-value representations:

- `Block`: an opaque marker or summary, without serializing ASTs or captured environments.
- `Assembly`: a view of its named public members.
- `Component`: a presentation of its target kind and inspectable value, without exposing mutable raw-handle access or private ownership state.

Update all generated union consumers to handle the new variants. Keep ordinary solid operations and their existing engine/artifact behavior. Do not create a new engine scene or artifact kind for an assembly.

Ownership/provenance state must remain consistent with runtime memory when executions are cached or resumed. Carry the associated registry with the relevant runtime state, or invalidate incompatible cached state. A stale cached handle must never become valid merely because its bookkeeping was discarded.

Preserve existing sketch cache behavior and ensure recursive solver-value substitution handles new value containers. Prevent ownership state or new-version values from leaking into earlier-version executions.

Add language and standard-library documentation, including examples and diagnostics. Use the repository's generation workflow for generated standard-library documentation and bindings instead of manually editing generated output.

## 5. Verification, implementation order, and explicit defaults

### 5.1 Acceptance scenarios

Run language-level scenarios against both the recursive and machine backends. Add engine-backed coverage where real geometry identity, command admission, or derived-reference behavior matters.

| Area | Required scenarios |
| --- | --- |
| Version boundary | Default version, 1.x, 2.x, 3.0-preview, and 3.x preserve prior behavior; 4.0-preview enables the feature. Keep the feature boundary suitable for later 4.x versions. |
| Earlier-version names | User-defined names matching the new function, module, and types remain valid where they were valid before. |
| Legacy parsing | Preserve old call-followed-by-object parsing, ordinary conditional bodies, comments, and settings appearing after other syntax. |
| Generic block syntax | Empty and nonempty blocks; comments; multiline calls; qualified calls; function aliases; calls in pipelines; parenthesized condition calls; source ranges and recast round trips. |
| Argument binding | Fixed `block` parameter, explicit forwarding, duplicate explicit-plus-trailing block, wrong block type, and a callee that does not accept the argument. |
| Deferred behavior | Ignored block never executes; stored and returned blocks retain lexical capture; repeated execution creates fresh local bindings. |
| Return and exits | Direct block return is rejected; nested function return works; errors and exits restore state and skip the success hook. |
| Defaults | Empty `assembly()`, components-only calls, block-only calls, and explicit block forwarding; other functions receive no implicit empty block. |
| Scope | Module export precedence, injected component names, same-scope duplicate definitions, local shadowing of outer names, and no captured/module/helper-local leakage into members. |
| Result membership | Numbers, strings, functions, sketches, objects, arrays, raw solids, nested assemblies, and consumed values retain their declared names; recursive conversion updates array typing. |
| Input validation | Reject unsupported property types and components passed as inputs; validate the complete object before any move or body side effect. |
| Input movement | The old solid and all aliases are invalid before the first body statement; injected values are components immediately. |
| Descendant movement | Moving an assembly invalidates a previously saved child handle; the new hierarchy remains usable; copied scalar metadata remains valid. |
| Consumed input assemblies | Moving an assembly with consumed geometry is rejected with the member path. |
| External captures | Reject direct, aliased, imported, container-held, and helper-accessed external geometry; allow fresh helper-created geometry. |
| Module provenance | Lazy top-level module geometry remains external; imported factory-created geometry belongs to the active construction. |
| Mixed objects | Allow a scalar projection from an object that also contains external geometry; reject the geometry projection or whole-container use. |
| Component inspection | Permit supported solid queries and member access without exposing a mutable raw solid. |
| Component protection | Reject mutators, transforms, consuming operations, casts that would unwrap a solid, and indirect mutation through derived faces, edges, sketches, or merge operations. |
| Independent construction | Allow construction of new independent geometry using permitted references to a component. |
| Clone | Cloning a solid component yields an independent mutable solid; preserve collection behavior; reject assembly cloning. |
| Other geometry | Exported non-solid geometry keeps ordinary modeling behavior and cannot bypass component protection. |
| Duplicate inputs and outputs | Reject direct aliases, nested-object/array duplicates, cross-subassembly duplicates, and repeated assembly identity; allow independent clones. |
| Consumed members | Preserve consumed names; fail when used as live geometry; ignore them only for duplicate-live-body checks. |
| Nesting | Nested assemblies, assemblies in sketches, and sketches in assemblies work under existing restrictions; an assembly does not bypass active nested-sketch checks. |
| Sketch replay | No new engine permissions, frontend mode, or cache behavior; solver substitution traverses new value containers. |
| Engine commands | Ordinary commands still execute; finalization sends no new message and adds no mandatory flush; legitimate pending construction commands can flush after finalization. |
| Failure and caching | Setup, body, and finalization errors unwind correctly; failures are not cached as success; stale generations and version changes cannot revive handles. |
| Tooling and views | Formatting, comments, node paths, hover/signatures, highlighting, generated bindings, and value views handle new variants without serializing private environments or ownership fields. |
| Deep execution | The machine evaluator handles deep nesting through continuations and cleans up state without introducing unbounded native recursion. |

Use test-only module fixtures to exercise opening future `std::assemblies` exports and collision behavior while keeping the production module empty.

Use mock execution for language and ownership cases that do not require an engine. Use engine simulations for real movement/identity, cloning, derived-reference mutation rejection, and sketch/assembly interaction.

All new feature examples and fixtures must opt into `4.0-preview`. Earlier-version KCL fixtures are appropriate only where they explicitly test compatibility.

### 5.2 Suggested implementation units

Implement in these dependency-ordered units:

1. **General blocks:** version-aware parsing, optional AST field, lexical capture and forwarding, the `Block` type, generic evaluation support, traversal, and formatting.
2. **Opaque assembly values:** `Assembly` and `Component`, runtime type integration, member access, views, and generated bindings.
3. **Ownership rules:** provenance, generation validation, transfers, descendant invalidation, external-geometry admission, consumption integration, and duplicate helpers.
4. **Assembly standard library:** the declaration, empty `std::assemblies` module, shared lifecycle, recursive and machine support, and no-op engine hook.
5. **Component operations:** narrow read-only adapters, solid-component cloning, and protection of derived references.
6. **Integration and compatibility:** nested sketch behavior, caching, documentation, tooling, and regression coverage.

Keep logically separate changes reviewable. Do not alter CI configuration to make the feature pass.

### 5.3 Verification commands and workflow

During implementation:

- Format Rust using `cargo +nightly fmt`.
- Run `cargo clippy --all-targets`.
- Run focused KCL parser, execution, standard-library, ownership, and tooling tests.
- Exercise the feature tests with both `KCL_EXECUTOR=recursive` and `KCL_EXECUTOR=machine`, using the repository's existing test commands.
- Run relevant engine-backed simulation tests where credentials and infrastructure are available.
- Regenerate bindings with `npm run bindings:update` when the Rust interfaces change.
- Build the required WASM artifacts before testing application integration that depends on them.
- Run affected CodeMirror, LSP, AST/recast, and value-view checks.
- Regenerate standard-library documentation through the repository workflow.

Use unchanged earlier-version tests and snapshots as evidence of compatibility. Do not update historical snapshots merely to conceal changed semantics.

Follow repository Rust error-handling conventions: return errors for invalid states instead of panicking or using `unreachable!()`. Keep Rust comments ASCII.

### 5.4 Explicit defaults and deferred work

The following decisions are settled for this implementation:

- The module is `std::assemblies`; the function is `assembly`; the input parameter is `components`.
- The generic block parameter name is `block`.
- General blocks use lexical capture and an opaque `Block` value.
- Only Rust code executes a block; no general KCL block-evaluation API is added.
- Existing `SketchBlock` AST nodes remain separate.
- Assembly members are accessed with ordinary dot syntax.
- Every injected input and directly declared body-local value becomes a member.
- Objects and arrays are recursively converted where they contain solids or assemblies.
- Consumption does not remove member names.
- Live-body duplication and repeated assembly identity are errors, with deterministic member paths.
- Components are read-only; other exported geometry retains normal modeling behavior.
- A whole-assembly move invalidates saved descendant handles.
- Captured environments are not rewritten after ownership transfers.
- Imports remain subject to the existing top-level-only rule.
- Ordinary failures and engine batching retain existing semantics.
- No assembly execution mode, frontend API, scene grouping, or engine message is added.

Leave three explicit future-work TODOs at the corresponding implementation points:

1. Revisit return-value and `return` semantics for general trailing blocks.
2. Implement cloning of assemblies and assembly-backed components.
3. Accumulate future mating constraints and send the new engine message from the finalization hook once the engine API is available.

The feature is complete when these semantics work in both evaluators, the finalization placeholder runs before a successful return without sending a new engine message, and earlier KCL versions retain their existing behavior.
