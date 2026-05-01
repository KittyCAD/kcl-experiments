## State

The frontend is the source of truth for source code. The interpreter has a copy too and keeps it up to date. The interpreter builds AST and other data structures based on it's copy of the program text.

A key concept is the scene graph. This is not a shared data structure and is morally transient, but kept statefully as an optimisation (thus, the API uses `SceneGraphDelta` in it's returns. I imagine this is useful for the frontend if it is retaining information between frames of animation, etc. but it may be a premature optimisation and we could just send the whole scene graph). The frontend and interpreter each have a scene graph, but there is no symmetry between them. They represent the same data in different ways. E.g., the interpreter scene graph keeps extra data around for mapping between the scene graph and AST or other internal data structures, and the frontend keeps extra data in the graph to map from the rendered scene to the graph (i.e., the frontend scene graph subsumes today's artefact graph) and to optimise rendering, etc. The scene graph abstracts the scene as rendered, not the whole program. The frontend displays everything (except the source code and engine video stream) based on its scene graph. The scene graph is based on durable ids called `ObjectId`s. These are preserved across most updates. When the interpreter cannot preserve ids, they are invalidated as part of the `SceneGraphDelta`.

## API

### Project

```
openProject(ProjectId, [File], openFile: FileId) -> Result<SceneGraph>
addFile(ProjectId, File) -> Result<SceneGraphDelta>
removeFile(ProjectId, FileId) -> Result<SceneGraphDelta>
// File changed on disk, etc. outside of the editor or applying undo, restore, etc.
updateFile(ProjectId, FileId, String) -> Result<SceneGraphDelta>
switchFile(ProjectId, FileId) -> Result<SceneGraph>
refresh(ProjectId) -> Result<SceneGraph>

File {
  id: FileId,
  path: Path,
  text: String,
}
```

### Code editing

```
codeEdit(ProjectId, FileId, Version, SrcDelta) -> Result<(Option<SceneGraphDelta>, [Error])>

// Only replace is necessary but having the others is convenient. It might not even be worth using these diffs and we just
// send the whole new source code.
enum SrcDelta {
  Append(String),
  Insert(Loc, String),
  Replace(Loc, Loc, String),
}
```

`Version` is an unsigned int, for the current file, `codeEdit` and `updateFile` increment it on success, new versions are included in `SceneGraphDelta`.

### Sweeps

```
addExtrude(ProjectId, FileId, Version, profile: ObjectId, args: ExtrudeArgs) -> Result<(SrcDelta, SceneGraphDelta)>
editExtrude(ProjectId, FileId, Version, extrude: ObjectId, profile: ObjectId, args: ExtrudeArgs) -> Result<(SrcDelta, SceneGraphDelta)>
deleteExtrude(ProjectId, FileId, Version, extrude: ObjectId) -> Result<(SrcDelta, SceneGraphDelta)>
```

### Sketch mode

```
// Returned ObjectId is the id of the in-progress sketch.
enterSketchModeNew(ProjectId, FileId, Version, plane: Plane) -> Result<(SrcDelta, SceneGraphDelta, ObjectId)>
enterSketchModeNewEdit(ProjectId, FileId, Version, sketch: ObjectId) -> Result<SceneGraphDelta>
exitSketchMode(ObjectId) -> Result<SceneGraph>

addLine(sketch: ObjectId, Version, from: (int, int), to: (int, int)) -> Result<(SrcDelta, SceneGraphDelta)>
addLineToPath(sketch: ObjectId, Version, path: ObjectId, to: (int, int)) -> Result<(SrcDelta, SceneGraphDelta)>
deleteLine(sketch: ObjectId, Version, line: ObjectId) -> Result<(SrcDelta, SceneGraphDelta)>

// For dragging using the mouse or using entered values. Frontend should take into account grid-snapping, etc.,
// transform from the screen coordinate space to scene coordinates, and other event handling stuff.
movePoint(sketch: ObjectId, Version, point: ObjectId, (int, int)) -> Result<(SrcDelta, SceneGraphDelta)>

addFixedPointConstraint(sketch: ObjectId, Version, point: ObjectId, (int, int)) -> Result<(SrcDelta, SceneGraphDelta)>
removeFixedPointConstraint(sketch: ObjectId, Version, point: ObjectId, XorYorBoth) -> Result<(SrcDelta, SceneGraphDelta)>
```

## Evaluation

This proposal is a big chunk of work, so we should be sure the cost is outweighed by the benefits. We also want to be sure that we can mitigate risk by implementing incrementally.


### Implementation cost

- Porting existing sketch mode to the new API would be a lot of work and hard to make incremental.
- However, we plan to remove this code. Implementing the new sketch mode using this API should be significantly easier than using the existing API because code mods can be implemented within the interpreter.
- Implementing the project lifecycle API should be fairly easy for the frontend (at first duplicating project state in the frontend, ignoring returned data, and using dummy file ids, versions, etc.). In the interpreter, keeping a mirror of the project and using this for code mods (but not direct execution) is fairly easy.
- Porting sweeps and similar actions should be fairly straightforward and incremental. It does not need to block any other work and can be done later or in parallel with higher priority work.


### Benefits

I looked at items on the KCL [roadmap](https://github.com/KittyCAD/modeling-app/discussions/8040). I identified the following items which would (or might) benefit from the proposed refactoring.

- constraints and new sketch mode - should be much easier with the high-level API since we can write code mods in the interpreter. The low-level API alternative would not help.
- Improved tags and tagging - might be a bit easier since dealing with back compat issues will mean parallel systems which will be easier to handle within the interpreter. Low-level doesn't help.
- More function functionality - easier since different function kinds are hidden from the frontend by the API. Low-level might help a bit, depending on design, but not as much.
- Interfaces - might help by encapsulating change to the interpreter. Low-level might help, depending on the design.
- Helper functions - might help to make helper functions transparent to the frontend (e.g., a call to `rectangle` and four lines could be treated the same). Low-level wouldn't help.
- More caching - would help since this will require changes to the AST and exec modes which would be encapsulated. Low-level API might help, but only if mapping from new data structures to the API is not too hard.
- Improving arg names - might help with back compat workarounds. Low-level wouldn't help.
- Improvements to the parser, static checking - likely to help a lot since the AST is encapsulated. Low-level API might help, but only if mapping from new data structures to the API is not too hard.

I think there is likely to be some benefit for day-to-day work like bug fixing and performance work, but not a huge amount now that we are focussed on backwards compatibility.


### Incremental delivery

This is touched on in the implementation cost section, but to make it clear, the plan would be:

- Implement the project lifecycle API in the frontend and interpreter, but don't use it for anything until required to.
  - Use `updateFile` for all code edits, `codeEdit` later.
- All initial implementations avoid diff/delta optimisations.
- Implement `SceneGraph` in Rust, including generating it as part of execution.
- The frontend mostly ignores the scene graph, the only thing that is required initially is to get the `ObjectId` to start or edit a sketch. Question: how hard is this?
- Develop the new API for sketch mode in parallel with the new sketch mode implementation.
- Port non-sketch mode functionality to the new API.
- Remove the old sketch mode and any other non-API interaction between frontend and interpreter.

I believe the biggest risk in terms of taking more than expected time and blocking incremental delivery is the design of the scene graph and its implementation in the interpreter. Therefore, we should prioritise starting that work and making it as incremental as possible.