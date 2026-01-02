# Summary
KCL math operations on numbers and their units should have a simple mental model with a short spec, such that anyone who reads the spec should be able to reason about a KCL expression and know what it evaluates to. This attempts to write down the spec of math version 2.

The main change in math version 2 is that math operations no longer attempt to infer pseudo-dimensional-analysis. Instead, each unit kind, like Length or Angle, is treated as a separate space. Math works within that space and simply propagates the unit of measure, doing unit conversions when necessary. Units never cancel or become unknown because it would have resulted in a different power dimension in dimensional analysis, like when lengths are multiplied to get an area.

The second change is making the `PI` constant have the same Count units as `TAU`.

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

**default units** - A unit of measure in KCL that's a member of multiple unit kinds. This is a convenience often used to omit units of measure suffixes from KCL literals. The actual units of measure depend on the module in which it statically appears, so two defaults may not be equal. Details will be explained below.

**clear units** - A unit of measure is said to be clear when it is a concrete unit of measure of Length, Angle, or Count. All other units of measure, including default, are unclear. (In the code, this is referred to as "known", but that's a misnomer because there are different levels of knowledge. Default units are more known than `unknown`, but still ambiguous.)

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

The literal `3` (with no suffix) has default units. In the context of this module, it means that `x` has the value `3` with units of measure that are the intersection of Count and millimeters, the latter because of `@settings(defaultLengthUnit = mm)`. In contrast, if the module used `@settings(defaultLengthUnit = in)`, the units of measure would be the intersection of Count and inches. (It's also the intersection of a third unit of measure, degrees, but usage of defaults for the Angle unit kind is discouraged due to ambiguity and results in a warning when used. So we will not discuss it further.)

It is the intersection, not the union, because the value can be used both as a Length _and_ as a Count. A single value is not restricted to one or the other the way a union is.

If `defaultLengthUnit` specification is omitted from a module, `mm` is assumed.

## Math Operations
Now that we have literal numbers, we can combine them using math operations.

### Combining Terms
In the expressions `a + b` and `a - b`, `a` and `b` are called terms. Before an operation is computed, the units of measure of the terms are analyzed.

This is unchanged from math version 1.

1. If units of measure of both terms are equal, the result is the same.
2. If either term is `unknown`, the result is `unknown`. `unknown` is infectious.
3. If either term is `any`, the result is the unit of measure of the other term. `any` is flexible.
4. If either term is clear:
	1. If the other term is ambiguous (like default), it's first narrowed to the unit kind of the clear term. If this is not possible, the result is `unknown`.
	2. If the other term has a different unit kind, the result is `unknown`.
	3. Since the two terms have the same unit kind, the resulting units are the same as the clear term. If both terms are clear, we use the units of the first term. The quantity is unit-converted to the units of the term that we use for the result so that the operation can be performed.
5. Anything else results in `unknown`.

Note: The actual algorithm is more complicated because this document intentionally omits any talk of generics.
### Combining Factors
In the expressions `a * b`, `a / b`, and `a % b`, `a` and `b` are called factors. Before an operation is computed, the units of measure of the factors are analyzed.

This is _very different_ from math version 1, where each op `*`, `/`, and `%` has a subtly different algorithm, and no unit-conversion was ever done.

1. If one factor is Count and the other is default, the result is default.
2. If one factor is non-Count clear and the other is Count, the result is the non-Count clear.
3. Anything else is the same as the algorithm for combining terms.

The rules mean that for multiplying and dividing, default dominates Count.

In contrast, for adding and subtracting, Count dominates default.

The intuitive motivation behind this difference is that usually when Count is used as a factor, it's a unitless number or ratio like PI modifying a quantity like a Length. A Count cannot be used in a coordinate, so yielding to the other unit is useful. But when Count is used as a term in addition, it's more likely to be an array index or other primary quantity that should override the ambiguity of defaults.

Without data, this is somewhat arbitrary, and personally, I'd be happy to remove this difference. If I had to choose one, I'd choose the factor algorithm. But this would mean even more breaking changes to change the term algorithm. The reason I think the factor algorithm is preferable is that Counts are rarely used, only for array indexing and unitless stdlib parameters like scale factors.

The main tension is between the ambiguity of default units and the desire for convenience of omitting units from literals.

If it were purely up to me, default units would resolve _only_ to Length, not the intersection of Length and Count. We already made this change for Angles, and people seemed fine with the change. I would make the same requirement of being explicit for Count units. This would simplify a lot of things by removing the ambiguity, while still allowing users to omit units suffixes for the vast majority of things, including all coordinates.

## PI Units
Math version 2 changes the units of measure of the `PI` constant in the stdlib from `unknown` to Count. This reduces the number of places where users get an `unknown` result and need to be explicit about the units they intend.

This also makes the units of the `PI` constant match the units of the `TAU` constant. Up until now, using `PI` would oftentimes result in a warning, but changing a KCL program to use `TAU / 2` instead would make the warning go away. That was confusing.

The original motivation behind using unknown units for `PI` was that people and LLMs were unaware of how KCL default units and automatic unit conversion worked, and they frequently used `PI` to attempt to manually convert units of angles. But this resulted in bugs in KCL programs. The `unknown` forced the user to be explicit about what they meant. But for a long time now, we've required explicit units for Angles. So the original motivation for the `unknown` unit of `PI` no longer applies.
