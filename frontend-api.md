# Summary

There should be an API that hides the abstract syntax tree (AST) implementation from the frontend TypeScript, allowing it to change internally without the frontend knowing or caring.  We propose a combination of focused APIs and more general query, traverse, and modify APIs.

# Motivation

Up until now, making changes to the AST has been slow and painful because the frontend code makes a lot of assumptions about its representation and structure.

There are several changes that we'd like to make to the KCL interpreter that likely require significant restructuring of the AST.  Some of these are longer-term goals.  But we'd like to make future changes easier and benefit from Rust's guarantees.

To do this, we'd like to stop exposing the internal representation of the AST to TypeScript.

Examples of changes that we may want in the future that likely require significant AST changes.

- Better parser (better error messages, incremental parsing, error recovery)
- Static analysis
- Performance: String interning
- Performance: Packed array of AST nodes, instead of heap-allocated nodes with pointers.
- Performance: Bytecode VM

Another motivating factor in why we're proposing this approach is that we'd like frontend developers to be able to make changes to app features and code-mods without significant Rust changes.

# Guide-level explanation

The AST types including `Program`, `BodyItem`, `Expr`, `VariableDeclaration`, `Identifier`, etc. will not be exposed to TS via ts-rs bindings.

Instead, there will be general opaque types and more specific [facade](https://en.wikipedia.org/wiki/Facade_pattern) types that only contain high-level information relevant to the frontend.

The general opaque types are:

| Current TypeScript type | Proposed opaque type | Description |
| --- | --- | --- |
| `string` | `KclSource` | Unparsed KCL source code or source code fragment |
| `Node<Program>` | `KclProgram` | Parsed KCL program, possibly including parse errors |
| `Node<T>` | `KclNodeInfo` | Facade enum of more details about a node. |
| `PathToNode` | `KclNodeId` | A reference to an AST node in a `KclProgram` |
| `SyntaxType` | `KclNodeKind` | A simple enum of the different types of AST Nodes, used for querying for a certain kind of node. |
| `SourceRange` | `SourceRange` | No functional change proposed to the type itself. |

Some examples of more specific `KclNodeInfo` facade types:

- Common properties that all `KclNodeInfo` have:
    - `id: KclNodeId`
    - `type: KclNodeKind`
    - `range: SourceRange | undefined` Similar to a node digest, the source range of a modified AST cannot be determined until it has been formatted back to a string.  So the API should reflect its possible absence to avoid unnecessary `parse(recast(ast))` all over.
- `VariableDeclaration`
    - `identifier: string`
    - `initializer: KclNodeId`
- `FunctionExpression`
    - `parameters: Array<Parameter>`
    - `body: KclNodeId`
- `Parameter`
    - `name: string`
    - `optional: boolean`
    - `unlabeled: boolean`
- `Call`
    - `callee: KclNodeId`
    - `arguments: Array<CallArg>`
- `CallArg`
    - `label: string | null`
    - `exp: KclNodeId`

## Modify API

In order to accomplish what the frontend needs, some things will have focused APIs that simply do the specific task, with the implementation in the Rust interpreter.  Some examples, many extracted from `modifyAst.ts`, include:

- **Replace Expression Node:** \[`getNodeFromPath()` with `replacement`\] Given a `KclProgram`, `KclNodeId`, and `KclSource`, returns a new `KclProgram` with the given node replaced with a new node representing the KCL source fragment parsed as an expression.  The `KclNodeId` of the new node doesn't change, but its type and everything else about it may change.
- **Insert Variable:** \[`insertNamedConstant()`\] Given a `KclProgram`, variable name prefix, a `KclSource` expression, optional Node ID to insert before or after, returns a new `KclProgram` with the new variable declaration and the `KclNodeId` of the new variable declaration body item.
- **Mutate Keyword Arg:** \[`mutateKwArg()` and `mutateKwArgOnly()`\] Given a `KclProgram`, a `KclNodeId` of a call expression, a string label/parameter name, and `KclSource` expression, returns a new `KclProgram` with the new argument.  There's also a parameter to restrict the update to literals since these are considered fully constrained in KCL 1.0.
- **Remove Keyword Args:** \[`removeKwArgs()`\] Given a `KclProgram`, a `KclNodeId` of a call expression, an array of string labels/parameter names, returns a new `KclProgram` with the new argument.  There's also a parameter to restrict the update to literals since these are considered fully constrained in KCL 1.0.

Note that we use a non-mutating interface of `KclProgram` since the main caller will be the frontend app that has a lot of concurrency.  JS/TS code is async by default.  Presenting an immutable interface rules out a class of bugs caused by referencing data structures across await points, where another actor mutates the data structure during the await. Under the hood, tricks may be used to not always clone the entire AST, but that isn't exposed to the interface.

These still need to be investigated from `modifyAst.ts`.

- \[`insertNewStartProfileAt()`\]
- \[`addSketchTo()`\]
- \[`sketchOnExtrudedFace()`\]
- \[`addOffsetPlane()`\]
- \[`addModuleImport()`\]
- \[`sketchOnOffsetPlane()`\]
- \[`splitPathAtLastIndex()`\]
- \[`splitPathAtPipeExpression()`\]
- \[`replaceValueAtNodePath()`\]
- \[`moveValueIntoNewVariablePath()`\]
- \[`deleteSegmentFromPipeExpression()`\]
- \[`deleteTopLevelStatement()`\]
- \[`removeSingleConstraintInfo()`\]
- \[`getInsertIndex()`\] This is actually a query.
- \[`updateSketchNodePathsWithInsertIndex()`\]
- \[`splitPipedProfile()`\]
- \[`createNodeFromExprSnippet()`\]
- \[`insertVariableAndOffsetPathToNode()`\]
- \[`createVariableExpressionsArray()`\]
- \[`createPathToNodeForLastVariable()`\] This is actually a query.
- \[`setCallInAst()`\]

## Query API

You may have noticed above that some APIs require a reference to a node using `KclNodeId`.  How does the caller obtain such an ID?  There will be a general query API, largely inspired by code that's currently in `queryAst.ts`.

- **Find Node at Source Location:** \[`getNodeFromPath()` and `getNodePathFromSourceRange()`\] Given a `KclProgram` and `SourceRange`, returns the `KclNodeInfo` of the most specific AST node or `undefined` if not found.
- **Look Up Node Info:** \[`getNodeFromPath()`\] Given a `KclProgram` and `KclNodeId`, get extra information of the AST node.  Returns the corresponding `KclNodeInfo` or `undefined` if not found.
- **Find Relevant Parent Node Of Type:** \[`getNodeFromPath()` with `stopAt`\] Given a `KclProgram`, a `KclNodeId`, and a target `KclNodeKind`, find node's parent of the given target kind, limited to relevant nodes, and return its `KclNodeId`.  For example, given a node that's a function call and a target of a variable declaration, return the variable declaration that the function call is in its initializer.  If it's not part of a variable declaration initializer, none will be returned.  If the call is inside a function body that's part of a variable declaration outside the function, none will be returned since the variable declaration outside the function isn't relevant.
- **Is Top-Level Variable Defined?** \[`traverse()` in `ModelingMachineProvider.tsx`\] Given a `KclProgram` and a string identifier, returns true if it's defined.
- **Find Stdlib Calls:** \[`traverse()` in `addEdgeTreatment.ts`\] Given a `KclProgram`, an optional `KclNodeId` representing the subtree to search, and a set of (`string` name?) target stdlib functions, find all call expressions calling any of the stdlib functions and return an array of `KclNodeId`s.  This is a best-effort find since, in the future, we'll be able to call arbitrary expressions, not just identifiers, store stdlib function references in variables, etc., meaning that the only way this can work perfectly is to execute the program.
- **Does Call Define Given Tag Name?:** \[`traverse()` in `addEdgeTreatment.ts`\]
- **Find All Subexpressions with Type:**
    - \[`traverse()` in `addEdgeTreatment.ts`\] Similar to Find Stdlib Call but looking for the first object expression with a specific.
- **Get Name of Tag that Node Defines:** \[`traverse()` in `addEdgeTreatment.ts`\]
- **Does Tags Argument Refer to Tag:** \[`traverse()` in `addEdgeTreatment.ts`\] Not sure whether this one should have its own dedicated function since it's so specific. The current TS code is looking at the `tags` argument to fillet() and chamfer() calls and determining whether `getOppositeEdge()` or `getNextAdjacentEdge()` is called on a specific tag name.

These still need to be investigated from `queryAst.ts`.

- \[`findAllPreviousVariablesPath()`\]
- \[`isNodeSafeToReplacePath()`\]
- \[`isLinesParallelAndConstrained()`\]
- \[`isSingleCursorInPipe()`\]
- \[`findUsesOfTagInPipe()`\]
- \[`hasSketchPipeBeenExtruded()`\]
- \[`doesSceneHaveSweepableSketch()`\]
- \[`doesSceneHaveExtrudedSketch()`\]
- \[`getObjExprProperty()`\]
- \[`isCursorInFunctionDefinition()`\]
- \[`getBodyIndex()`\]
- \[`isCallExprWithName()`\]
- \[`doesSketchPipeNeedSplitting()`\]
- \[`getVariableExprsFromSelection()`\]
- \[`getSelectedPlaneAsNode()`\]
- \[`locateVariableWithCallOrPipe()`\]
- \[`findImportNodeAndAlias()`\]
- \[`findPipesWithImportAlias()`\]
- \[`getPathNormalisedForTruncatedAst()`\]
- \[`findAllChildrenAndOrderByPlaceInCode()`\]
- \[`getLastVariable()`\]

Note: I don't think the `getNodeFromPath()` `returnEarly` parameter needs to exist.

## Traversal API

In order to have a way to implement very specific queries, instead of many one-off functions, there will be a `traverse()` API that does a DFS traversal of the AST, calling a TS callback on each node, providing `KclNodeInfo` of the node.  Crucially, this doesn't expose the internal representation of the nodes, but only their facade.  This is a last resort for callers when a more-focused API doesn't exist (we should assess the situations where this gets used, and see if it signals holes in our query helpers).

## Initial Implementation

In the future, the AST may not be represented as heap-allocated enums with pointers to child nodes.  In that future, we may have different solutions.  But in the interim, we need to be able to implement the same functionality that we do now without re-writing everything.

- **Finding a node:** Until we change the representation, the only way to get a Rust-reference to a node in the AST is to traverse it.  The opaque `KclNodeId` internally will be a modified Rust `NodePath`.  Since `NodePath` isn't resistant to positional changes like inserting a variable at the top of the program body, we need to preserve and port the existing APIs that update a set of `PathToNode`s.
- **Finding a node's parents:** With the current AST representation, it's non-trivial to have parent references.  By having `KclNodeId`s be `NodePath` internally, we can either build up the parent references as we traverse to the node or simply stop traversing early.  The latter is what TS currently does.

Since `KclNodeId` is opaque, the internal representation can be changed in the future when the AST representation changes without affecting callers.  For example, say that we change the AST to be a packed array of nodes, where child references are indexes into the array.

- `KclNodeId` can be an index into the node array.
- `KclProgram` can have a side-table mapping from node index to parent-node index.  Since parent references are not needed during execution, ideally they would not be stored in the nodes themselves for performance.

This last part is hypothetical and the exact details are outside the scope of this doc, but it shows that such a change would be possible using this proposal.

# Reference-level explanation

TODO

# Drawbacks

The reason not to do this is that it's potentially a lot of work.  But we think that the investment will be worth it.

Another potential drawback is that if we ever add plugins implemented in JavaScript, it may limit their ability.  But we don't have any concrete plans to implement this anytime soon.  If the APIs are powerful enough, it will address this concern.

# Rationale and alternatives

## Transitioning to the Proposed API Isn't as Straightforward as High-Level Functions

An alternate approach is to use high-level functions or methods, one for each code-mod from the user's perspective.  An example would be adding a parallel constraint to two selected segments.  In this approach, there would be a Rust function for doing exactly this.  A benefit of this is that transitioning to it might be as simple as porting existing code from TS to Rust, which could be done incrementally.

However, this would probably make it more difficult for frontend developers working in TS to add new frontend features that need a slightly different code-mod.  They would be forced to switch to Rust.  It also wouldn't reduce the burden for making AST changes, only move it to Rust code.  For these reasons, we think that it would be more beneficial if the API allowed high-level decision-making to be implemented in TS and use the refactoring APIs to query and modify in separate lower-level calls, creating a clear boundary and interface.

## Stable Node IDs: Content-Based Addressing Vs. Sequential IDs

This proposal uses AST Node IDs that need to be stable across edits.  Sequential IDs are fine when the edits are done via the API.  IDs can be preserved.

But when KCL source code is edited by the user and the source needs to be re-parsed, it may be challenging or ambiguous to preserve sequential AST node IDs from the previous parse.  This ability could be desirable for collaborative editing, for example, where only textual diffs are sent over the wire while the user could be doing anything else in the app.

For this reason, the idea came up to use content-based addressing of nodes to make AST node identifiers that were resistant to positional changes after node insertions and deletions.

But we don't currently need this feature, so perhaps we shouldn't be worrying about it.  When a user enters a command-bar flow, we don't generally allow the user to also edit code while the app is holding on to node references via `PathToNode`s.  When this does happen and a `PathToNode` is looked up and no node is found, it's a bug.

## Mutable Vs. Immutable Interface

In general, it's easier to implement a mutable interface that's efficient, both in terms of speed and memory usage.  Immutable interfaces that are also reasonably efficient are possible, but they're generally more work to implement.  The most basic and straightforward approach relies on structural sharing, i.e. avoiding cloning or rebuilding of shared subtrees.  Structural sharing is trivial in languages with garbage collection.  Rust can make that a bit more annoying, depending on the details.  But I think the trade-off is worth it to have an immutable interface to avoid bugs that are hard to reason about in the frontend code due to concurrency.

We know that there are a few use cases like dragging segment points in sketch mode that require low latency.  If performance of the immutable API is a problem, we could implement special APIs for these use cases that mutate that are exceptions to the general rule of immutability.

# Prior Art

TODO

# Unresolved questions

TODO

# Future possibilities

## Rust-Owned AST

Currently, when TS parses KCL, the AST is serialized to JSON, transferred to TS, and TS deserializes it to a JavaScript object, that object is manipulated, and finally, it's serialized back through JSON again to execute it in Rust.  There's a lot of overhead.

Part of the benefit of not exposing the AST to TS is that the AST never needs to leave memory owned by Rust code.  The WebAssembly interface can return only a reference or handle pointing to the parsed AST that's stored on the Rust side.

But Rust code has no way of knowing when the `KclProgram` references become unreachable.

In this case, there can be an explicit "free" API for unused `KclProgram`s.  Calling code can use the [`FinalizationRegistry`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry) to detect when a JS object is no longer used and reclaim the memory by calling `free()`.
