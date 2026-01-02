# Summary
KCL math operations on numbers and their units should have a simple mental model with a short spec, such that anyone who reads the spec should be able to reason about a KCL expression and know what it evaluates to. This attempts to write down the spec of math version 2.

The main change in math version 2 is that math operations no longer attempt to infer pseudo-dimensional-analysis. Instead, each unit kind, like Length or Angle, is treated as a separate space. Math works within that space and simply propagates the unit of measure, doing unit conversions when necessary. Units never cancel or become unknown because it would have resulted in a different power dimension in dimensional analysis, like when lengths are multiplied to get an area.

The second change is making the `PI` constant have the same Count units as `TAU`.

The third change is that bare number literals without a unit suffix no longer represent Count. This removes ambiguity in units, which greatly simplifies both the mental model and the implementation.

# Motivation
Users and implementers of KCL are frequently confused by how numbers and units behave. It's frequently unclear why certain expressions result in warnings. When adding new stdlib functions, it's frequently unclear what the function signature should be because of subtleties in units.

No one has a clear mental model of how KCL units work.

The longer we wait to fix KCL math, the harder it will be and the more people it will affect.

# Human-Readable Spec
## Definitions
**Unit Kind** - A group of concrete units of measure that are compatible with each other for addition, subtraction, and comparison (called commensurable). This is similar to a [kind](https://en.wikipedia.org/wiki/Physical_quantity#Kind) in dimensional analysis, where two quantities of the same kind can be added together or compared. We call this a unit kind. For example, Length, Angle, and Count are distinct unit kinds in KCL. They are called "unit kinds" and not simply "kinds" to differentiate with type kinds.

**Unit of Measure** - The concrete unit of the quantity. Each unit of measure is a member of a single unit kind. For example, millimeters are a unit of measure, and their unit kind is Length. Inches are a unit of measure, and their unit kind is also Length. Degrees are a unit of measure, and their unit kind is Angle. Because millimeters and inches have the same unit kind, the quantities can be compared. In contrast, millimeters and degrees cannot be compared. In KCL, Count is a unit of measure. Since there are no other units of measure with the unit kind, the unit kind has the same name.

`unknown` - A unit of measure in KCL that's part of an unknown unit kind. It's the result of combining multiple units of measure in a way that doesn't make sense.

`any` - A unit of measure in KCL that's a member of all unit kinds. It is useful in functions when the unit of measure doesn't matter.

**default units** - A unit of measure in KCL that's a convenience used to omit units of measure suffixes from KCL literals. The actual units of measure depend on the module in which it statically appears, so two defaults may not be equal. Details will be explained below.

**clear units** - A unit of measure is said to be clear when it is a concrete unit of measure of Length, Angle, or Count. Default units are not themselves clear, but can be unambiguously resolved to a clear Length unit of measure on the context of a module because default units are unambiguous in this proposal. All other units of measure are unclear. (In the code, this is referred to as "known".)

## Literals
When you write a literal number in KCL, it has a numeric value and a unit of measure. Example literals, and their unit of measure:

- `2.5mm` millimeters
- `1in` inches
- `90deg` degrees
- `2rad` radians
- `5_` count
- `3` default

When a suffix is omitted, the units of measure of the quantity depends on the source location of the literal. The `@settings()` in the module determines the mapping from default to possible units of measure.

For example, say that the module has this:

```
@settings(defaultLengthUnit = mm)

x = 3
```

The literal `3` (with no suffix) has default units. In the context of this module, it means that `x` has the value `3` with units of measure that are millimeters because of `@settings(defaultLengthUnit = mm)`. In contrast, if the module used `@settings(defaultLengthUnit = in)`, the units of measure would be inches.

If `defaultLengthUnit` specification is omitted from a module, `mm` is assumed.

## Math Operations
Now that we have literal numbers, we can combine them using math operations.

### Combining Terms
In the expressions `a + b` and `a - b`, `a` and `b` are called terms. Before an operation is computed, the units of measure of the terms are analyzed.

This is mostly unchanged from math version 1, except for the handling of Count combined with non-Count clear units.

0. Resolve any default units to clear units of measure
1. If units of measure of both terms are equal, the result is the same.
2. If either term is `unknown`, the result is `unknown`. `unknown` is infectious.
3. If either term is `any`, the result is the unit of measure of the other term. `any` is flexible.
4. If either term is clear:
    1. If one term is non-Count clear and the other is Count, the result is the non-Count clear.
	2. If the other term has a different unit kind, the result is `unknown`.
	3. Since the two terms have the same unit kind, the resulting units are the same as the clear term. If both terms are clear, we use the units of the first term. The quantity is unit-converted to the units of the term that we use for the result so that the operation can be performed.
5. Anything else results in `unknown`.

Note: The actual algorithm is more complicated because this document intentionally omits any talk of generics.
### Combining Factors
In the expressions `a * b`, `a / b`, and `a % b`, `a` and `b` are called factors. Before an operation is computed, the units of measure of the factors are analyzed.

The algorithm is the same as for combining terms.

This is _very different_ from math version 1, where each op `*`, `/`, and `%` has a subtly different algorithm, and no unit-conversion was ever done.

The rules mean that, in this proposal, default units always dominate Count.

The intuitive motivation behind this difference is that usually when Count is used as a factor, it's a unitless number or ratio like PI modifying a quantity like a Length. A Count cannot be used in a coordinate, so yielding to the other unit is useful.

## PI Units
Math version 2 changes the units of measure of the `PI` constant in the stdlib from `unknown` to Count. This reduces the number of places where users get an `unknown` result and need to be explicit about the units they intend.

This also makes the units of the `PI` constant match the units of the `TAU` constant. Up until now, using `PI` would oftentimes result in a warning, but changing a KCL program to use `TAU / 2` instead would make the warning go away. That was confusing.

The original motivation behind using unknown units for `PI` was that people and LLMs were unaware of how KCL default units and automatic unit conversion worked, and they frequently used `PI` to attempt to manually convert units of angles. But this resulted in bugs in KCL programs. The `unknown` forced the user to be explicit about what they meant. But for a long time now, we've required explicit units for Angles. So the original motivation for the `unknown` unit of `PI` no longer applies.

## Explicit Suffix for Count Units

The third change in this proposal is that numbers with Count units must use the `_` suffix. When you omit a units suffix on a number literal, its unit of measure resolves unambiguously to a unit of measure with unit kind Length.

This means that in order to index arrays or use stdlib functions that require a Count, explicit `_` or ascription must be used.

The motivation for this is that it makes all numbers unambiguous. This _greatly_ simplifies both the implementation and the mental model for users.

The mental model is now:

> 1. When you see no explicit suffix, its units are the module units like `mm`. It's _always_ a Length.
> 2. Non-Count dominates Count.

It's _way_ simpler for everyone.

In contrast, there is no written description of the math v1 mental model because it's so complicated. It was even worse when defaults could be Angles also. This change does the same for Count.

A programming language is primarily a tool for communication between _people_. If human experts cannot read the code and understand it, something is very wrong. This needs to be fixed, and the sooner we fix it, the better.
