# Attributes syntax

We're currently using `@` and some ad hoc following stuff.

Proposed changes:

- Inner vs outer attribute semantics
  - Attachment in AST
- Inner attributes with no identifier
- Tool attributes

## Proposal

Goal: to be flexible and extensible.

### Syntax

Inner/outer is not meant to be user-facing jargon.

```
item ::= ... | fn_decl | const_decl | import | inner_attr | outer_attr item
stmt ::= ... | const_decl | outer_attr stmt

inner_attr ::= `@` `(` (annot_item,)* `)`
outer_attr ::= `@` id (`(` (annot_item,)* `)`)?
annot_item ::= id `=` expr
```

Note: `expr` in `annot_item` is a parsing requirement but not a semantic one, so e.g., `@(defaultLengthUnit = mm)` is ok and `mm` is interpreted as a unit, not an identifier.

Inner attributes cannot be applied to outer attributes, but multiple attributes (inner or outer) can be applied to a single item, attributes combine rather than override, precisely how this is handled depends on the attribute handler for the specific attribute.

### Semantics

`outer_attr` applies to the surrounding scope, e.g., an outer attributes at the top level of a file applies to the whole module.

`inner_attr` applies to the following item.

Values are specified below or may be specified by tools. Incorrect values are warnings (not errors).

### Initial values

#### Outer attributes

- `settings`
  - `defaultLengthUnit`: a length unit, `mm`, ...
  - `defaultAngleUnit`: an angle unit, `deg`, `rad`
  - `attrs`: identifier[], to register consumers of attributes
- `metadata`
  - `kclVersion`: a semver string
  - `title`: string
  - `description`: string
  - any other `ident: string` values
- `no_std` - no implicit import of the std prelude

Possibly:

- `warnings`
  - `allow`: identifier | identifier[], silence warnings (or lints)
  - `deny`: identifier | identifier[], make warnings (or lints) into errors

#### Inner attributes

- `import` of non-KCL files
  - `format`: ident, one of `fbx`, `gltf`, `glb`, `obj`, `ply`, `sldprt`, `step`, `stl`
  - `lengthUnit`: a length unit, `mm`, ...
  - `coords`: ident, one of `zoo`, `opengl`, `vulkan`
- functions
  - `impl`: `kcl` (default) or `std_rust` (body must be empty, implemented in Rust as part of KCL interpreter)


### Examples

```
@settings(defaultLengthUnit = in, attrs = [fmt])

import "foo.kcl"
@(lengthUnit = mm)
import "foo.obj"

@(impl = std_rust)
fn bar(@sketch, radius: number): Circle {}

@(fmt = ignore)
fn baz() {
  @settings(...)
  @fmt(ignore = true)
}
```


## Alternatives

- `#` and `#!` like Rust but without `[]`
  - `#!settings(defaultLengthUnit = in)`, `#(lengthUnit = mm)`
- `#` instead of `@`
  - `#settings(defaultLengthUnit = in)`, `#(lengthUnit = mm)`

```
#!settings(defaultLengthUnit = in)

import "foo.kcl"
#(lengthUnit = mm)
import "foo.obj"

#(impl = std_rust)
fn bar(@sketch, radius: number): Circle {}

fn baz() {
  #!settings(...)
}
```

## Issues

- "Attributes" or "annotations"? I prefer the former, we've been using a mix, but mostly the latter. Other languages use either or both, no consensus.
- Syntax overlap with `@` for 'self'
  - Remove `@` on self-arg once we've finished the kwarg migration
- Discoverability and docs
  - settings and attributes on imports are now documented, not sure if we want to document attributes in one place or in many places based on what they do. Some attributes designed for use in the standard library are probably better left undocumented.
