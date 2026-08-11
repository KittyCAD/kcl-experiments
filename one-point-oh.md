# 1.0 plan and priorities

We don't have time for big changes or big features (dev time + time to marinate). We know vaguely where we're going (see other design PRs).

High level priorities:

- Stability and polish
  - bugs, performance, glitches, etc.
- User experience
  - Goal: KCL is easy to read and comprehend
  - Feedback is useful (though perhaps not pretty)
    - IDE experience
    - Error messages (parser in particular)
- Supporting core new features in engine, as required
- Future compatibility
- Not making tech debt worse
  - And ideally a little bit better

Related:

- [Modelling app roadmap](https://github.com/orgs/KittyCAD/projects/35)
- [1.0 milestone](https://github.com/KittyCAD/modeling-app/milestone/2)
- [CAD primitives roadmap](https://github.com/KittyCAD/modeling-app/issues/729)

Priorities in **bold**, dependencies in *italics*, task size in t-shirt sizes (taking into account breaking change management; m-l means uncertainty about which, not 'somewhere in between').

## As required

### Bug fixing

**P0**:

- high priority bugs as they emerge
- plan to seek out and address lower-priority bugs during feature freeze


### Non-language feature work

Priority follows from other team's priorities.

New or more flexible std lib functions and code mods are expected as side effects of other work.


### Pre-paying technical debt

**P1**:

- Don't make things worse by adding hacks, especially around the API of KCL

**P3**:

- Supplying user-side IDs to engine
- code mod API

## Feature work

**P1** (priority ordered):

- reserve keywords (s)
  - https://github.com/KittyCAD/modeling-app/issues/4486
  - and give meaning to underscore in identifiers
- assemblies (xl?) - see below
- function args
  - named and optional args (m; *$0*)
  - only one non-named arg (m; *$1*)
    - ideally we make it the first arg, but doesn't have to be
    - could require it be called `self`, but I don't think we need to and that makes back-compat harder
- simple tidying changes
  - automatic conversion of round floats to ints, remove the `int()` function (s)
  - record init: `:` -> `=`  (s)
  - function decl: remove `=` and `=>`, `:` for type annotations (s)
    - return type syntax?
  - convert special strings (e.g., `END`, `XY`) to constants (s)
- std lib
  - organise into modules (for docs, at least) (l)
    - potential issues: namespacing, collisions, receivers, name shadowing for std
  - ensure consistency, remove unnecessary or uncertain functions, check for anything we're badly missing (m)
    - *depends somewhat on $3, we should do this in any case, but how we do it will depend on whether $3 is achieved*
- support optionally-implicit `%` in pipelines (m; *$2; depends on $1*)


**P2** (priority ordered):

- std lib
  - remove use of anon records (l; *depends on $0*)
- immutable model
  - well-defined rules for when engine calls cause a rendered object, add `construction` flag where necessary (m-l)
- tagging
  - support `as` in pipelines (m)
  - remove `$` syntax (m)
  - where std lib functions take tags, ensure they're unambiguous and work with `as` variables (m)


**P3** (priority ordered):

- std lib
  - reduce number of functions by using optional args (m-l; *$3; depends on $0*)
- `var`/`roughly` syntax for unconstrained points/numbers (m-l)
  - changes how we do 'fixed point' constraints

**P-Unknown**

- [Multiple profiles in Sketches](https://github.com/KittyCAD/modeling-app/issues/1876) (m-l)

### Assemblies

TODO what are the 1.0 requirements? How must we interact with the frontend? Is module-as-part the right abstraction for assemblies?

(I'm using 'render' as short-hand for sends commands to the engine)

Requirements:

- extend `import` syntax to allow importing the whole module (**P1**)
  - replace `import()` for non-KCL objects (**P3**)
- collect the module into an object to represent the part (**P1**)
  - without rendering (**P1**)
- Syntax to cause rendering of a part object (**P1**)
- Manipulation of a part
  - Access to tags and variables (**P2**)
    - export tags/variables
  - transformation (**P2**)
  - extension (**P3**)
    - e.g., if the module is a sketch, can we extrude it?
- check code mods (don't) work, etc. (**P1**)

Implementation plan:

See also caching in performance section, below.

Note: I don't think we have time to change the fundamental model enough to do this nicely with modules as proper objects with functions being side-effect free and returning a single object, etc.


- treat modules as their own data type
  - somewhat object-like, since they support field access
  - somewhat function-like, for rendering
- import:
  - `import 'foo.kcl' as bar`/`import 'bar.kcl'`
  - brings `bar` name into scope
    - exported functions and variables of `bar` can be accessed using `bar.whatever`
      - QUESTION: or `::`? Note that module is not a `self` for dispatch, but we do want to treat modules like structs?
    - using `bar` must implicitly clone because the module cannot be modified by side-effects
      - both in the engine and KCL
      - variable/fn access doesn't need to clone, but assignment or pipeline use does
    - using or assigning `bar` at the top-level causes rendering
- for rendering, we 'replay' the whole module top-level *as if it were a function*
  - what object does this create? An array of shapes?
    - Can it be used in pipelines for transforms, etc.?
    - This object needs to have the variables/tags/`tags` field, i.e., it *is* the part object, but with a UUID
    - Does the engine support grouping of objects like this?
  - Does treating it as a function work? Would multiple objects be rendered by side-effect in a function today?
- `tags` of module contains all exported variables
  - I think we need `export as`, given how much we rely on tags, which bumps the priority of fixing tags
  - alternative: put all variables and tags in `tags` - back compat issue

## Performance

- Caching (**P2**)
  - Incremental re-execution
  - Goal
    - sub-tree of AST and it's reverse-deps are identified as unchanged (even if transformed)
    - program memory representation is preserved, no new calls to engine
      - program memory repr must be complete (i.e., we've got all the info we need without engine calls and their side-effects)
      - QUESTION: how is program memory GC'ed?
      - QUESTION: What other compiler/interpreter state would need to be preserved/invalidated?
    - can we use copy-on-write/ref counting for sharing program memory repr within the same run of a program?
      - only valuable if we have a lot of instantiations of the same thing and the thing is large
  - for 1.0
    - per-file/module granularity
      - invalidation and dependency tracking is easier
        - hash a whole file for slightly more sophisticated change tracking rather than just a valid bit
        - means we can skip parsing too, but would mean comment changes would affect us
          - could hash the token stream (could use AST digest, but then we can't skip parsing. Would just need to compute it, not the stored version, easy to map a module to an AST digest)
    - ref counting/cow? (**P3**)
      - probably not a huge benefit if we are caching between runs
    - still need the same changes to preserve/invalidate program memory/interpreter state
 
## User feedback

TODO I don't really know what needs doing here, how it should be prioritised, or how it should be prioritised relative to the above language work.

- Error messages
  - Parser
- IDE functionality
  - Suggestions
  - ...

## Docs and education material

Planned for feature freeze period (**P3** until then). We should do some planning before-hand (**P0**).

- std docs
- guide
- reference
- videos and other tutorial stuff


## Open questions

- Tags (these might be things we want to change for 1.0), in particular the current approach of collecting all tags into a `tags` field and allowing users of the geometry to refer to internal tags seems sub-optimal:
  - source names are reified at runtime
  - construction (including tag names) of an object should be private by default
  - having a `tags` field violates the principle of tags being just variables
  - a more principled way to do it is `export` of variables, but this 'works' for block syntax sketches, not so much for pipelines, and not for deeply nested tags
- Any other precautions we should take for future compatibility? Including (but not limited to):
  - higher order functions
    - used in `patternTransform` pattern function, `reduce` and `map`
    - are we happy committing to supporting higher order functions?
    - how would they fit into a type system?
  - Units of Measure
    - syntax (is `num id` valid syntax today?)
    - modules with units? (shouldn't be an issue since we only support imports within a project)
  - Are there things we want to 'tighten up'?
    - Indexing records (`foo['fieldName']`)
    - Implicit conversion from array to record (pretty sure this is unintentional and should not work)
    - Make arrays uni-typed?
    - `KclValue` top type? (IMO this is the internals of the implementation leaking into the surface syntax and should be removed)


## Postponed

- [over-eager mutation](https://github.com/KittyCAD/modeling-app/issues/2728)
- Performance or tech-debt work
- point syntax (perhaps just change from `[]` to `()`? Or require `pt`/`vec`?)
- [construction geometry](https://github.com/KittyCAD/modeling-app/issues/1553), implicit or explicit `show`
- extensions to tagging
  - 'dynamic tagging', e.g., for edges of a `polygon`
