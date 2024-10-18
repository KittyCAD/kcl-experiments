# Module system extensions and assemblies

This doc describes possible extensions to the current `import`/`export` syntax for modules. This includes support for assemblies.

I don't think much of this is high priority, but we might want to include some parts in 1.0. I would like to have a good idea of the preferred direction here since it informs other language design issues.

This doc is fairly detailed, but that is mostly to prove out the designs and ensure internal consistency, etc. Consider all syntax to be a strawman for iteration and the whole doc to be a starting point for discussing the design rather than a complete proposal.

The doc is ordered by requirements, rather than a logical breakdown of the complete feature. Much is left unspecified for now, especially where I don't think it will affect other design choices. What is spec'ed and what isn't should be understood as a comment on overall prioritisation.

Previous discussion:

- [Initial modules proposal (`use` statements)](https://github.com/KittyCAD/modeling-app/issues/4080)
- [Implementation of initial module system](https://github.com/KittyCAD/modeling-app/pull/4149)

## Import all items from a module

Importing all items (aka glob imports) is primarily a convenience so users don't need to make a long list of names in an `import` expression. I think this will get more important with some ideas I have for modularising std, but more on that elsewhere.

The basic idea is we allow `import * from "foo.kcl"` which imports all `export`ed names from foo.kcl into the namespace of the current file.

There are downsides to allowing this: primarily if an exported name is added to foo.kcl then that can cause an error in the importing file if the name already exists (which may be from another import, e.g., `import * from "bar.kcl"`). It's also a bit harder to follow the code without tooling (you can't tell at a glance where a name is defined).

Syntax: `*` is common in many PLs, but is a bit opaque for new programmers. A keyword like `all`, `everything`, or `any` would be better, but that is one less name we can all use for programming (we can't use a contextual keyword here). I think it would be a bit too subtle to use no names to mean all names (e.g., `import from "foo.kcl"`).

Renaming: one of the drawbacks of glob imports is that you can't rename items in the glob. To facilitate that, I propose that `*` (or whichever keyword) is allowed at the end of a list names and imports all exported names which haven't appeared in the list. So `import a, b, * from "foo.kcl"` is allowed (assuming `a` and `b` are defined in foo.kcl) and has the same meaning as `import * from "foo.kcl"`; more usefully `import a as c, b as d, * from "foo.kcl"` imports the same items but renames `a` to `c` and `b` to `d` (`a` and `b` are not available). There would be an error if the names `c` or `d` are used elsewhere, including in the `*`.

Optional extension: name clashes from glob imports are only an error if the name is actually used, not just imported.

Alternative: explicit names take priority over implicitly imported names (from globs). I do not like this. It is what is done in Rust and it causes issues (it's also pretty subtle to explain).

## Import constants from a module

We permit `export` on constant declarations (e.g., `export foo = 42`) and those exported constants can be imported in the same way as functions. The difficulty is around side effects, design there is postponed, blocked on discussing side-effects and the KCL model of computation.

## Import the module itself as a name

There's an important question about what this actually means in terms of the side-effects, let's postpone that, blocked on discussing side-effects and the KCL model of computation. For now assume there is some (unnamed) geometry exposed from a module, and also constants and functions which we wish to name via the module.

`import "foo.kcl"` imports the module of foo.kcl with the name `foo`. We also permit renaming: `import "foo.kcl" as bar` imports the module of foo.kcl with the name `bar`. If there are characters in the filename which are not permitted in identifier names (e.g., spaces, `-`, or `.` other than separating the extension), then renaming is required (there is no 'clever' renaming such as mapping `-` to `_`).

We may wish to allow the module itself to be imported as well as specific names: `import a, self from "foo.kcl"` (I'm reusing the `self` keyword from Rust, but there may be a better one for KCL) would import `a` from `foo.kcl` and the module itself as `foo`. Renaming (`self as bar`) would have the obvious meaning.

We may also want to allow an assignment style rename: `bar = import "foo.kcl"` with the same meaning as `import "foo.kcl" as bar`. This would be more natural for assemblies. Although it provides two ways to do the same thing (generally bad), I think both ways are natural (and I am thinking of using `as` elsewhere with the same meaning to implement tagging, but that's way out of scope for this doc). If we do allow this we *may* also consider it for single name imports,e.g., `bar = import baz from "foo.kcl"`.

### Assemblies

A module may or may not contain top-level geometry (as well as zero or more named constants or functions). If it does, then the name of the module (whether renamed, assigned or whatever) should work as an assembly. E.g., assuming foo.kcl contains top-level geometry, then after `import "foo.kcl"`, `foo` can be used in the same way as if the geometry from foo.kcl were defined in the importing file as a constant called `foo`.

Obviously there is a massive caveat about design questions around side effects.

There is an open question about how the UI can modify assemblies from other files (note that the same feature will be used for assemblies from other projects which must be immutable to the UI).

### Foreign formats

`import` can be used to import objects defined in other CAD formats, replacing the `import` function in the standard library. E.g., `model = import("tests/inputs/cube.obj")` is replaced by `import "tests/inputs/cube.obj" as model`. Such objects are treated in the same way as other assemblies, but they are always opaque (no AST, etc.) and read-only.

### Naming items within a module

We require some format to name items (constants or functions) within a module, e.g., assume that `a` is defined in foo.kcl, then following `import "foo.kcl"`, we should permit `foo::a` to name `a` in `foo`. This would have exactly the same semantics as using `a` following `import a from "foo.kcl"`, only the naming is different. All names must be `export`ed to be named from other modules, whether that is via an `import` or via `::`.

Syntax: I've used `::` above as a strawman syntax which is common in other languages. An alternative is `.`. This reduces the number of symbols in the language and is likely to be intuitive to some users (since we use it in a similar way for field access), however it also implies modules are just like objects. I think this is mostly true, so it might be a good choice. However, there might be some advantage to users or the implementation to distinguish runtime lookup (of fields or maybe methods) and simple naming. I'm not sure.

Note that there are no submodules, so this naming syntax only makes sense with imported modules. Even with subdirectories and so forth, there is no way to name another module without explicitly importing it.

## Re-exporting imported names

We permit `export import ...` (and `export x = import ...`, if we support that form, see above). These make the imported names (in their renamed form, as appropriate) visible outside the current module in the same was as an exported constant or function (as well as making them available within the module).

## Import graphs

It is permitted to import a name multiple times and there is no name clash if the name points at exactly the same item (even if the path of imports to the item is different). It is permitted to import an item multiple times with different names.

Cycles of imports are permitted (we could offer a lint which detects these). When tracing imports, a file is never read twice; there is no fixpoint computation required.

E.g.,

```
// In foo.kcl

import * from "bar.kcl"
export x = ...

// In bar.kcl

import * from "foo.kcl"
export y = ...
```

The above is allowed and `x` and `y` can be named in both files. If the import in bar.kcl is changed to `import x from "foo.kcl"`, there is no change. If it is changed to `import y from "foo.kcl"` there is no error (though we might warn that the import does nothing), `y` is visible in both modules and `x` is only visible in foo.kcl. If the import in bar.kcl is changed to `import from "foo.kcl"`, `x` is visible in both files, `y` is visible in foo.kcl, `foo` and `foo::x` are visible in bar.kcl. Note that `foo` is not visible in `foo.kcl`.

## Non-top-level imports

Imports may appear anywhere within a file (including within conditionals, etc). A name may only be used following its import and at the same or narrower scope.

E.g.,

```
// At top level
import "foo.kcl"
a = foo // ok

b = bar // error
import "bar.kcl"

x = ... {
  import "baz.kcl"
  c = baz // ok
  d = bar // ok
}

e = baz // error
```

Caveat: postponed design work around side-effects.

## Importing from other directories within the project

Paths (as well as filenames) may be used inside the quoted string of an `import`. Paths are always relative to the top-level of the project, and may not include `.` or `..` (alternative: we could relax this requirement and allow relative paths as long as traversing the path never leaves the project directory). Paths are ignored in the name of the module as imported E.g., `import "foo.kcl"` brings in `foo` from the root directory of the project, `import "baz/bar/foo.kcl"` brings in `foo` from `foo.kcl` found at `baz/bar`, `import a from "baz/bar/foo.kcl"` brings in `a` from the same file. The path of the file makes no difference to the imported name or to privacy.

All modules within a project are nameable, there is no need for `export` anywhere on a module. If all items in a module are private (i.e., none are `export`ed), the module itself can still be named, but it is useless (like an empty object) and a glob import would import nothing.

## Importing from other projects

This is mostly postponed for future work. We would use the same syntax and semantics as the rest of the system, however the string in the import would include some indicator that the target is outside the project and how to locate it.

Example with very, very strawman syntax: `import a from "cargo://some-library/foo.kcl"` would import `a` from the file `foo.kcl` in the project `some-library` located in the `cargo` (lol) repository. `import a from "local://../some-library/foo.kcl` would import `a` from the file `foo.kcl` from a sibling directory of the current project called `some-library`. The point of these examples is not the syntax for identifying other projects or the places where an external project might live, just to show how the `import` statement might be extended to support other projects.

Privacy across multiple projects is postponed, but I would like to keep it simple, e.g., just keeping the single `export` keyword to expose names to other projects as well as other modules in the same project.

### Versioning

Versioning info and other extensions should all be specified within the file identifier string. Details of the design postponed.

### Cycles between projects

Design postponed.

## Importing data

`import` can also be used to import data, e.g., `import "foo.json"` would create a `foo` object with the json data. Exactly how that works is postponed design, the interesting questions I see are how arbitrary data is represented in our quite limited object data structure, and how we specify what format to treat data as (is it just implied by the extension or can it be overridden? Can we support arbitrary data formats by allowing the user to provide a decoder, etc.)

## Support for units of measure in modules

Design postponed, blocked on discussing UoM types first.
