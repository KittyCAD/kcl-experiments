# Sketcher Requirements Rework

## 1. Overview

This document is intended to outline functional requirements for a 2D 'sketcher', and owes a significant debt to Max M.'s [Advanced Constraint Workflows](https://docs.google.com/spreadsheets/d/1-fuSsJgvXIeSUSZRhWRrSkWRk0RVyI8bTS1WEAGHar4).

The required functionality is discussed primarily from a user requirement perspective, with some implementation related detail given where relevant.

### 1.1 Related Documents

- [This spreadsheet](https://docs.google.com/spreadsheets/d/1YHkqXJOuhxwJFZztyvKRlPVCdAboQ9oqNGhnD6Ukn5M/edit?usp=sharing) gives an exhaustive analysis of the main sketch constraint options offered elsewhere.

## 2. Core Geometric Elements

The user should be able to compose their desired 2D geometry from the following elements:

- `Point`
  - A zero-dimensional location in 2D space, defined by an `(x, y)` coordinate pair.
- `Line`
  - A finite straight segment between two points, typically defined by two `Point` elements: a start point and an end point.
- `Circle`
  - A closed curve of constant radius from a given centre point, typically defined in one of two ways:
    - Centre and radius: a `Point` element defines the circle's centre, and a scalar value describes its radius. Diameter may also be used.
    - Three-point: three `Point` elements are defined on the circle's circumference, from which the remaining geometry can be constructed.
- `CircularArc`
  - A segment of a circle bounded by two end points. May be defined by:
    - A centre `Point`, a scalar radius value, and both start and end angles.
    - Three `Point` elements which lie on the segment; start, intermediate, and end.
  - _Some systems (e.g., Onshape) allow circular arcs to be created with an initial tangency constraint to another element. In this case, the user typically controls the arc’s centre and end point, with the start point determined by the tangency._
- `Ellipse`
  - A closed conic curve defined by a centre point and two orthogonal axes (major and minor). Typically specified by:
    - Centre point.
    - Lengths of major and minor axes.
    - Orientation (angle of major axis relative to something known, e.g., horizontal or vertical).
- `EllipticalArc`
  - A continuous segment of an ellipse, bounded by angular or positional limits. Typically defined by:
    - The same properties as a full `Ellipse`.
    - Start and end points or angular limits along the perimeter.
- `Conic`
  - Non-circular, non-elliptical conic sections, specifically parabolae and hyperbolae, are included here. These are likely to see less frequent use, except in specialised industries such as optics and aerospace. Can be defined by:
    - Focus and directrix.
    - Control points and tangents.
    - Type-specific parameters (e.g. eccentricity).
- `Spline`
  - A piecewise polynomial curve, typically cubic, though degree may be configurable. In this context, 'spline' generally refers to a _fit-point spline_, where the user defines a series of points that the curve must pass through. These are called fit points, and the curve interpolates between them.
    - This contrasts with _control-point splines_ (e.g. B-splines), where the curve is influenced by the points but does not necessarily pass through them.
  - Fit-point splines are particularly important in engineering workflows where users often import point data derived from domain-specific or in-house software, e.g., cams, gears, exhaust paths.
  - See [^1], [^2],[^5].

### 2.1 B-Splines and Béziers

While 'spline' generally refers to fit-point splines (a piecewise polynomial curve that passes through an explicitly defined set of points), control-point splines (like B-splines) are favoured in some instances, particularly in complex or freeform surface design.

Examples from existing systems:

- CATIA offers fit-point spline functionality through the 2D sketcher[^2], but its '3D curve' functionality offers 'through points' (fit-point spline), 'control points' (B-spline), and 'near points' (arc) options [^3], [^4].
- Fusion appears to expose B-spline functionality, referred to as a 'Control Point Spline'[^1].
- Onshape appears to expose B-spline functionality, simply referred to as 'Bezier'[^5].
  - Note that this is not raw Bézier segment functionality.

**Direct exposure of raw quadratic or cubic Bézier curve segments does not appear to be common amongst incumbent CAD systems**, though exposing a `Bezier` element may have some utility.

### 2.2 Interactivity

Interaction with the geometric primitives will differ by type and context For example, an unconstrained `Point` should be freely draggable around the sketch area, while it should be possible to drag an entire `Line` object—or either of its end points. This requirement for interaction with more than one component of a given element also extends to the application of dimensional and geometric constraints, e.g., applying a length constraint or a horizontal constraint.

The following table outlines expected interactive components for each core geometric type, assuming no constraints or dimensions have been applied.

| Geometry        | Interactive Region                    | Behaviour                                                                     |
| --------------- | ------------------------------------- | ----------------------------------------------------------------------------- |
| `Point`         | Point itself                          | Draggable.                                                                    |
| `Line`          | Start point                           | Draggable to move line start.                                                 |
|                 | End point                             | Draggable to move line end.                                                   |
|                 | Line body                             | Draggable to move entire line in x-y plane, maintaining direction and length. |
| `Circle`        | Centre point                          | Draggable to move entire circle.                                              |
|                 | Perimeter (circumference)             | Draggable to scale radius.                                                    |
| `CircularArc`   | Start point                           | Draggable to move arc start.                                                  |
|                 | End point                             | Draggable to move arc end.                                                    |
|                 | Centre point                          | Draggable to move arc centre and one end point.                               |
|                 | Perimeter (circumference)             | Draggable to scale radius, moving both end points equally.                    |
| `Ellipse`       | Centre point                          | Move ellipse.                                                                 |
|                 | Major/minor axis handles (if present) | Adjust size/shape.                                                            |
|                 | Perimeter                             | Draggable to scale ellipse about centre point.                                |
| `EllipticalArc` | Start point                           | Draggable to move arc start.                                                  |
|                 | End point                             | Draggable to move arc end.                                                    |
|                 | Centre point                          | Move ellipse.                                                                 |
|                 | Major/minor axis handles (if present) | Adjust size/shape.                                                            |
|                 | Perimeter                             | Draggable to scale ellipse about centre point.                                |
| `Spline`        | Fit points                            | Draggable points the curve must pass through.                                 |
|                 | Curve body                            | Often used for visual feedback, but not always directly draggable.            |
| `Conic`         | Control points / focus                | If editable, user can adjust via control geometry (e.g., focus or directrix). |

### 2.3 Construction Geometry

It should be possible to toggle any geometric element into 'construction' mode. Construction geometry entities:

- Participate in constraints and dimensions.
- Are omitted from any profile finding/region analysis and downstream features (extrude, revolve, etc.).
- Typically render lower in the visual hierarchy; often dashed and grey or desaturated.

## 3. Editing Tools

Most commercial CAD packages offer `trim` and `extend` tools which can be used to modify geometric elements within a sketch, typically 'trimming' elements to intersection points, or extending them to meet other elements.

Other editing tools include the ability to break and split segments, chamfer or fillet corners, and pattern objects within the sketch.

The following table outlines key editing tools:

| Tool                            | Supported Elements                                                             | Typical Inputs                                   | Expected Result                                                                          | Notes                                                                                                                                            |
| ------------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------ | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Trim**                        | `Line`, `Circle`, `CircularArc`, `Ellipse`, `EllipticalArc`, `Spline`, `Conic` | One or more cutting elements, a target element.  | Deletes the portion of the target element that lies beyond the cutting element/boundary. | Requires robust intersection intersection; honour active constraints (e.g. maintain tangency if possible). Circles and ellipses may become arcs. |
| **Extend**                      | `Line`, `CircularArc`, `EllipticalArc`, `Spline`, `Conic`                      | Target element, boundary element.                | Extends the target element until it meets the boundary.                                  | No length extension if already intersecting.                                                                                                     |
| **Fillet / Chamfer**            | `Line`-`Line`                                                                  | Two element, radius / distance.                  | Replaces the corner with an arc (fillet) or straight edge pair (chamfer).                | Must add coincident/tangent constraints automatically.                                                                                           |
| **Mirror**                      | Any                                                                            | Elements to copy, mirror line.                   | Creates symmetric copy; can optionally add symmetric constraints.                        | If the source already references the mirror line, prefer constraint over copy.                                                                   |
| **Pattern (Linear / Circular)** | Any                                                                            | Element(s), count, spacing / angle.              | Creates copies; instances typically inherit constraints from original element.           | Instances should be grouped for UX, e.g., deleting all patterned objects.                                                                        |
| **Move**                        | Any                                                                            | Element(s), translation vector.                  | Translates unconstrained geometry.                                                       | Geometry must be separately constrained post-op.                                                                                                 |
| **Rotate**                      | Any                                                                            | Element, base point, angle.                      | Rotates unconstrained geometry around base.                                              | Geometry must be separately constrained post-op.                                                                                                 |
| **Scale**                       | `Line`, `Circle`, `CircularArc`, `Ellipse`, `EllipticalArc`, `Spline`, `Conic` | Element(s), base point, factor.                  | Uniformly scales distances from base point for unconstrained elements.                   | Geometry must be separately constrained post-op.                                                                                                 |
| **Break / Split**               | `Line`, `Circle`, `CircularArc`, `Ellipse`, `EllipticalArc`, `Spline`, `Conic` | Element, split point(s) or splitting element(s). | Divides an element into two; shared endpoints are coincident.                            | Constraints maintained.                                                                                                                          |

Implementation of these tools will require access to performant intersection detection methods, though there exists a body of internal knowledge on these already.

## 4. Constraints

At the user-facing level, constraints can be broken into two categories: **geometric** and **dimensional**.

- Geometric constraints control the relationships of objects with respect to each other[^6].
- Dimensional constraints control the distance, length, angle, and radius values of objects[^6].

Practically, these are exposed to the user in different ways. Dimensional constraints typically involve the specification of numeric values, e.g., a line's length or a circle's radius. Geometric constraints typically involve the application of one of a finite selection of possible relationships between objects, e.g., making two lines parallel, or two circles concentric.

For example, a user could apply an angular (dimensional) constraint between two line elements, enforcing an angle, $\theta$, of 45° between $L_1$ and $L_2$. Similarly, they could apply perpendicularity (geometric) constraint between two line elements, enforcing $L_1' \perp L_2'$ . In both cases, this situation could be described by an equality constraint; that the angle between two vectors must, within some tolerance, be equal to 45° and 90° respectively. Though these are exposed to the user through different categories of constraint, they are philosophically equivalent; an angle constraint between two vectors†.

†_With respect to the actual constraint solving problem, it may be possible to make optimisations based on specific cases, e.g., horizontal and vertical constraints, perpendicularity constraints etc., such that the search space can actually be reduced, but this is really an implementation detail. The intent here is to highlight that the differentiation between geometric and dimensional constraints is somewhat artificial and exists for the benefit of the user._

### 4.1 Required Constraints: Geometric

#### 4.1.1 Coincident

- Description: Forces two entities to occupy the same position. For example, point-to-point enforces zero Euclidean distance; point-to-curve enforces that the point lies on the curve.
- Applicable elements/element combinations:
  - `Point`-`Point`
  - `Point`-`Line`
  - `Point`-`Circle`
  - `Point`-`CircularArc`
  - `Point`-`Ellipse`
  - `Point`-`EllipticalArc`
  - `Point`-`Conic`
  - `Point`-`Spline`
  - `Line`-`Line`
  - `Circle`-`Circle`
  - `Circle`-`CircularArc`
  - `Ellipse`-`Ellipse`
  - `Ellipse`-`EllipticalArc`
  - `Conic`-`Conic`
  - `Spline`-`Spline`
  - `B-Spline`-`B-Spline`

#### 4.1.2 Parallel

- Description: Aligns two lines to have the same direction (or exactly opposite). Implemented by constraining their direction vectors to be scalar multiples (zero 2D cross product).
- Applicable elements/element combinations:
  - `Line`-`Line`

#### 4.1.3 Perpendicular

- Description: Forces two lines to intersect at 90°. Can be implemented by enforcing a dot product of zero between direction vectors.
- Applicable elements/element combinations:
  - `Line`-`Line`

#### 4.1.4 Tangent

- Description: Forces two elements to touch at exactly one point with shared tangents. For curves, the tangent vectors at the point of contact must align. For lines, they must be perpendicular to the target's normal at the contact point.
- Applicable elements/element combinations:
  - `Line`-`Circle`
  - `Line`-`Ellipse`
  - `Line`-`CircularArc`
  - `Line`-`EllipticalArc`
  - `Line`-`Conic`
  - `Line`-`Spline`
  - `Circle`-`Circle`
  - `Circle`-`Ellipse`
  - `Circle`-`CircularArc`
  - `Circle`-`EllipticalArc`
  - `Circle`-`Conic`
  - `Circle`-`Spline`
  - `Ellipse`-`Ellipse`
  - `Ellipse`-`CircularArc`
  - `Ellipse`-`EllipticalArc`
  - `Ellipse`-`Conic`
  - `Ellipse`-`Spline`
  - `CircularArc`-`CircularArc`
  - `CircularArc`-`EllipticalArc`
  - `CircularArc`-`Conic`
  - `CircularArc`-`Spline`
  - `EllipticalArc`-`EllipticalArc`
  - `EllipticalArc`-`Conic`
  - `EllipticalArc`-`Spline`
  - `Conic`-`Conic`
  - `Conic`-`Spline`
  - `Spline`-`Spline`

#### 4.1.5 Concentric

- Description: Constrains circular or arc elements to share the same centre point.
- Applicable elements/element combinations:
  - `Point`-`Point`
  - `Point`-`Circle`
  - `Point`-`CircularArc`
  - `Point`-`Ellipse`
  - `Point`-`EllipticalArc`
  - `Circle`-`Circle`
  - `Circle`-`CircularArc`
  - `Circle`-`Ellipse`
  - `Circle`-`EllipticalArc`
  - `CircularArc`-`CircularArc`
  - `CircularArc`-`Ellipse`
  - `CircularArc`-`EllipticalArc`
  - `Ellipse`-`Ellipse`
  - `Ellipse`-`EllipticalArc`
  - `EllipticalArc`-`EllipticalArc`

#### 4.1.6 Collinear

- Description: Forces two lines or points to lie along the same infinite line. Distinct from parallel: collinear lines share a path.
- Applicable elements/element combinations:
  - `Line`-`Line`

_Note: this functionality may be provided via the **Coincident** constraint._

#### 4.1.7 Horizontal

- Description: Constrains a line to lie horizontally (parallel to the x-axis), or two points to share the same Y coordinate.
- Applicable elements/element combinations:
  - `Line`
  - `Point`-`Point`

#### 4.1.8 Vertical

- Description: Constrains a line to lie vertically (parallel to the y-axis), or two points to share the same X coordinate.
- Applicable elements/element combinations:
  - `Line`
  - `Point`-`Point`

#### 4.1.9 Equal

- Description: Forces entities to have equal size; equal length (lines), or equal radius (circles/arcs).
- Applicable elements/element combinations:
  - `Line`-`Line`
  - `Circle`-`Circle`
  - `Circle`-`CircularArc`
  - `CircularArc`-`CircularArc`

#### 4.1.10 Symmetric

- Description: Constrains two elements to be mirrored across a reference line. For lines, also enforces equal length and mirrored angle.
- Applicable elements/element combinations†:
  - `Point`-`Point`
  - `Line`-`Line`
  - `Circle`-`Circle`
  - `CircularArc`-`CircularArc`
  - `Ellipse`-`Ellipse`
  - `EllipticalArc`-`EllipticalArc`
  - `Conic`-`Conic`
  - `Spline`-`Spline`

†Symmetric constraints really involve three elements: two geometric elements and an axis of symmetry, often the X or Y axis—though this could be any line.

#### 4.1.11 Midpoint

- Description: Forces a point to lie at the midpoint of a line segment, or a line to pass through the midpoint of a line segment.
- Applicable elements/element combinations:
  - `Point`-`Line`
  - `Point`-`CircularArc`

### 4.2 Required Constraints: Dimensional

For dimensional constraints, it is also necessary to provide interactive access to three key elements of each sketch:

- `Origin`: an immutable `Point` element that exists at `(0,0)`.
- `XAxis`: an immutable, infinite `Line` element that exists at `y=0`.
- `YAxis`: an immutable, infinite `Line` element that exists at `x=0`.

_Note that all dimensional constraints are typically available via a single 'Dimension' button, with the dimensional constraint type inferred from selected element types and user input._

### 4.2.1 Length

- Description: Specifies the Euclidean distance between a line’s two end-points; generally applied exclusively to line elements.
- Applicable elements/element combinations:

- `Line`

### 4.2.2 Distance

- Description: Specifies a Euclidean distance between two elements.
- Applicable elements/element combinations:
  - `Point`–`Point` (including `Point`-`Origin`)
  - `Point`–`Line` (including `Point`-`XAxis` and `Point`-`YAxis`)
  - `Point`–`Circle`
  - `Point`–`CircularArc`
  - `Point`–`Ellipse`
  - `Point`-`EllipticalArc`
  - `Line`–`Line` (only when the two lines are parallel)

### 4.2.3 Vertical Distance

- Description: Specifies separation along the **Y-axis** only.
- Applicable elements/element combinations:
  - `Point`–`Point` (including `Point`–`Origin`)
  - `Point`–`XAxis`
  - `Point`–`Line` (line must be horizontal)

### 4.2.4 Horizontal Distance

- Description: Specifies separation along the **X-axis** only.
- Applicable elements/element combinations:
  - `Point`–`Point` (including `Point`–`Origin`)
  - `Point`–`YAxis`
  - `Point`–`Line` (line must be vertical)

### 4.2.5 Angle

- Description: Constrains the angle between two lines, or between a line and an axis.
- Applicable elements/element combinations:
  - `Line`–`Line`
  - `Line`–`XAxis`
  - `Line`–`YAxis`

### 4.2.5 Radius

- Description: Fixes the radius of a circular element.
- Applicable elements/element combinations:
  - `Circle`
  - `CircularArc`

### 4.2.6 Diameter

- Description: Sets twice the radius directly (alternative to a radius constraint).
- Applicable elements/element combinations:
  - `Circle`
  - `CircularArc`

<!-- ### 4.2.7 Arc Length

- Description: Specifies the length measured along an arc segment.
- Applicable elements/element combinations:
  - `CircularArc`

### 4.2.8 Arc Angle

- Description: Specifies the sweep angle of an arc, usually in degrees.
- Applicable elements/element combinations:
  - `CircularArc` -->

### 4.3 Constraint Support Matrices

In the interest of readability, the required constraint support is given in matrix form below. Relevant condensed nomenclature:

- **P**: `Point`
- **L**: `Line`
- **C**: `Circle`
- **CA**: `CircularArc`
- **E**: `Ellipse`
- **EA**: `EllipticalArc`
- **S**: `Spline`

#### 4.3.1 Unary Constraints - Geometric

| Constraint \ Geometry | `P` | `L` | `C` | `CA` | `E` | `EA` | `S` |
| --------------------- | :-: | :-: | :-: | :--: | :-: | :--: | :-: |
| **Horizontal**        |     |  x  |     |      |     |      |     |
| **Vertical**          |     |  x  |     |      |     |      |     |

#### 4.3.2 Unary Constraints - Dimensional

| Constraint \ Geometry | `Point` | `Line` | `Circle` | `CircularArc` | `Ellipse` | `EllipticalArc` | `Spline` |
| --------------------- | :-----: | :----: | :------: | :-----------: | :-------: | :-------------: | :------: |
| **Length**            |         |   x    |          |               |           |                 |          |
| **Radius**            |         |        |    x     |       x       |           |                 |          |
| **Diameter**          |         |        |    x     |       x       |           |                 |          |

#### 4.3.3 Binary/Ternary Constraints - Geometric

| Constraint \ Element Pair | P-P | P-L | P-C | P-CA | P-E | P-EA | P-S | L-L | L-C | L-CA | L-E | L-EA | L-S | C-C | C-CA | C-E | C-EA | CA-CA | CA-E | CA-EA | E-E | E-EA | EA-EA | S-S |
| :------------------------ | :-: | :-: | :-: | :--: | :-: | :--: | :-: | :-: | :-: | :--: | :-: | :--: | :-: | :-: | :--: | :-: | :--: | :---: | :--: | :---: | :-: | :--: | :---: | :-: |
| **Coincident** ‡          |  x  |  x  |  x  |  x   |  x  |  x   |  x  |  x  |     |      |     |      |     |  x  |  x   |  x  |  x   |   x   |  x   |   x   |  x  |  x   |   x   |  x  |
| **Parallel**              |     |     |     |      |     |      |     |  x  |     |      |     |      |     |     |      |     |      |       |      |       |     |      |       |     |
| **Perpendicular**         |     |     |     |      |     |      |     |  x  |     |      |     |      |     |     |      |     |      |       |      |       |     |      |       |     |
| **Tangent**               |     |     |     |      |     |      |     |     |  x  |  x   |  x  |  x   |  x  |  x  |  x   |  x  |  x   |   x   |  x   |   x   |  x  |  x   |   x   |     |
| **Concentric**            |     |     |  x  |  x   |  x  |  x   |     |     |     |      |     |      |     |  x  |  x   |  x  |  x   |   x   |  x   |   x   |  x  |  x   |   x   |     |
| **Collinear**             |     |     |     |      |     |      |     |  x  |     |      |     |      |     |     |      |     |      |       |      |       |     |      |       |     |
| **Horizontal**            |  x  |     |     |      |     |      |     |     |     |      |     |      |     |     |      |     |      |       |      |       |     |      |       |     |
| **Vertical**              |  x  |     |     |      |     |      |     |     |     |      |     |      |     |     |      |     |      |       |      |       |     |      |       |     |
| **Equal**                 |     |     |     |      |     |      |     |  x  |     |      |     |      |     |  x  |  x   |     |      |   x   |      |       |  x  |  x   |   x   |     |
| **Symmetric** †           |  x  |     |     |      |     |      |     |  x  |     |      |     |      |     |  x  |  x   |     |      |   x   |      |       |  x  |  x   |   x   |  x  |
| **Midpoint**              |     |  x  |     |  x   |     |  x   |     |     |     |      |     |      |     |     |      |     |      |       |      |       |     |      |       |     |

† `Symmetric` takes a third argument for the axis of reflection.  
‡ For pairings of `Circle`, `CircularArc`, `Ellipse`, and `EllipticalArc`, the `Coincident` constraint is functionally identical to the `Concentric` constraint, enforcing that the centre points are coincident.

#### 4.3.4 Binary Constraints - Dimensional

| Constraint \ Element Pair | P-P | P-L | P-C | P-CA | P-E | P-EA | P-S | L-L† | L-XAxis | L-YAxis |
| ------------------------- | :-: | :-: | :-: | :--: | :-: | :--: | :-: | :--: | :-----: | :-----: |
| **Distance**              |  x  |  x  |  x  |  x   |     |      |     |  x   |         |         |
| **Horizontal Distance**   |  x  | x‡  |     |      |     |      |     |      |         |    x    |
| **Vertical Distance**     |  x  | x‡  |     |      |     |      |     |      |    x    |         |
| **Angle**                 |     |     |     |      |     |      |     |  x   |    x    |    x    |

† `Distance` between two lines is allowed only when the lines are parallel (treated as a shortest-distance dimension); acts as horizontal or vertical distance when the lines are horizontal or vertical respectively.
‡ `Horizontal Distance` and `Vertical Distance` between point and line only when the line is vertical/horizontal respectively.

## 5. Constraint Solver

Incumbent CAD systems generally support relatively inexpressive constraints; largely just those listed above. They do not generally support arbitrary expressions, e.g., enforcing that the length of line $L_{1}$ should be the equal to the square root of the length of line $L_{2}$, for example.

### 5.1 Method Overview

At a high level, the system's objective is to:

1. Translate a set of user-defined geometric and dimensional constraints into a single system of equations.
2. Find a set of values for the relevant variables (e.g., point coordinates, arc radii) that satisfies these equations simultaneously.

Practically, each constraint is converted into one or more residual functions, where that residual function measures how close the given constraint is to being satisfied. Broadly speaking, the solver then works by gathering this set of residuals into an equation of the form:

```math
\mathbf{F}(\mathbf{x}) = \mathbf{0}
```

Where:

- $\mathbf{F}$ is the vector of all residual functions.
- $\mathbf{x}$ is the vector of unknown (free) variables to be solved for: point coordinates, radii, angles, etc.

Formally, this is a **nonlinear multivariate root-finding problem**. To solve this system, iterative numerical methods are employed.

These methods start with an initial guess for the values of $\mathbf{x}$ and progressively refine this guess until the sum of the squares of the residuals is minimised. For most methods, this process involves the Jacobian matrix, $J$, which contains the partial derivatives of each residual function with respect to each variable ($\partial{F}/\partial{\textbf{x}}$).

The Jacobian has dimensions $m \times n$, where $m$ is the number of residual functions and $n$ is the number of free variables.

### 5.1.1 Example Residual Functions

| Constraint          | Example Residual Function, $(f_i)$                       |
| ------------------- | -------------------------------------------------------- |
| $Coincident(P,Q)$   | $\lVert P-Q\rVert = 0$                                   |
| $Parallel(L_1,L_2)$ | $u_x v_y - u_y v_x = 0$                                  |
| $Radius(R)$         | $\lVert P_{\text{onCircPerim}}-C_{Centre}\rVert - R = 0$ |

### 5.2 Structural Pre-processing

Before running the numerical solve, the system of constraints may be possible to analyse and simplify. This structural optimisation can make a significant difference to performance, as it reduces the size and complexity of the problem that the iterative solver must handle.

#### 5.2.1 System Decomposition

The constraint graph (where primitives are nodes and constraints are edges) can often be broken down into several smaller, wholly independent sub-problems. This is equivalent to finding the connected components of the graph. Solving several small systems independently is significantly more efficient than solving one large, combined system. See: https://github.com/KittyCAD/geometric-constraint-playground/blob/a143aa4fdd6c01c5017ac6836c5c7f0c251b5fda/src/newton/solver_base.py#L85C9-L85C34

While more advanced decomposition techniques (e.g., Dulmage–Mendelsohn or block triangular decomposition) can identify quasi-independent subsystems, I'm led to believe sparse solvers are effective at exploiting the inherent sparsity and partial decoupling within a single connected system. Therefore, the primary decomposition step is to separate the problem into its fully disconnected components.

#### 5.2.2 Symbolic Substitution

Some constraints (e.g., two points being coincident) can be used to eliminate variables entirely. If point P2 is constrained to be coincident with P1, all references to P2's coordinates in other constraints can be symbolically replaced with P1's coordinates. This reduces the number of variables (n) in the system, shrinking the Jacobian and further accelerating the numerical solve.

### 5.3 Numerical Solve

After pre-processing, the simplified system(s) of equations are passed to an iterative numerical solver, likely a sparse Newton method. At each iteration, the solver computes an update step $\Delta x$ by solving the linear system $J \Delta x = -F(x_n)$.

### 5.3.1 System Classification and Solution Strategies

The constraint system can be classified by analyzing the Jacobian matrix $J$ at the current state, where $J$ has dimensions $m × n$ ($m$ constraint equations, $n$ variables). The system's solvability depends on both the rank of $J$ and the consistency of the constraint equations.

**Classification based on Jacobian analysis:**

- **Under-constrained (Underdetermined)**
- **Condition:** $rank(J) < n$
- **Meaning:** Fewer independent constraints than variables. The system has degrees of freedom.
- **Solution Strategy:** Use regularised methods (e.g., Tikhonov regularisation) to find the minimum-norm solution that represents the smallest change from the current state. This preserves the system's draggable behavior during sketch construction.

- **Well-constrained**
- **Condition:** $rank(J) = n$ and the system converges to $\lVert F(x*)\lVert \approx 0$
- **Meaning:** The number of independent constraints matches the variables, and the constraints are mutually consistent.
- **Solution Strategy:** Standard Newton-type methods can find the unique solution $x*$.

- **Over-constrained**
- **Condition:** $rank(J) = n$ but the system fails to converge to $\lVert F(x*)\lVert \approx 0$ (residual remains large)
- **Meaning:** The constraints are inconsistent—they cannot all be satisfied simultaneously.
- **Solution Strategy:** Report constraint conflict to the user. The system should identify and highlight the conflicting constraints rather than attempting a least-squares approximation.

**Implementation notes:**

Rank estimation can be performed using QR decomposition or SVD. For numerical stability, use a tolerance-based rank calculation rather than exact zero comparisons.

The distinction between well-constrained and over-constrained systems cannot be determined purely from Jacobian analysis; it requires attempting the solve and examining both convergence behavior and final residual magnitude. A system that appears well-constrained locally (full rank Jacobian) may still be globally inconsistent due to the nonlinear nature of geometric constraints.

**Practical detection strategy:**

1. Compute `rank(J)` to identify under-constrained systems immediately
2. For systems with `rank(J) = n`, attempt the numerical solve
3. Classify as well-constrained if convergence achieves `||F(x*)|| < tolerance`
4. Classify as over-constrained if convergence fails or final residual exceeds tolerance,

This approach acknowledges that constraint consistency in nonlinear systems can only be definitively determined through the solving process itself.

---

As an aside, my mental model for these things is basically that in an $Ax=b$ case, where $A$ is $m \times n$, the system is:

- Underdetermined when $m < n$; fewer equations than unknowns.
- Fully determined when $m = n$; equal numbers of equations and unknowns.
- Overdetermined when $m > n$; more equations than unknowns.

However, it should be noted that this doesn't hold up—because we are interested in the linearly independent components, as are revealed by the matrix's rank.

### 5.3.2 Tikhonov Regularisation

For the underdetermined systems case, where there are infinitely many possible solutions, we need a stable way to choose one. The standard Newton step $J \\Delta x = -F(x)$ is ill-posed. Instead, we solve the corresponding linear least-squares problem using the normal equations, which for underdetermined systems must be regularised.

Tikhonov regularisation modifies the equation to $(J^T J + \\lambda^2 I)\\delta x = -J^T r$, where $r$ is the residual vector $F(x_n)$. The $\\lambda^2 I$ term is a small diagonal matrix that ensures the system is invertible and stable. This pushes the solver toward a solution that has a minimal deviation from its initial state, effectively finding the 'closest' (minimum-norm) valid solution when degrees of freedom exist.

### 5.4 Recommended Solver Approach: Sparse Newton's Method

Given the nature of geometric constraint problems, where any given constraint typically only involves a small subset of the total variables, the resulting Jacobian matrix is very sparse. This sparsity is a critical feature to exploit for performance.

A dense solver would waste significant computation on zero-value entries, whereas a sparse solver will only operate on the non-zero elements. Therefore, a **sparse Newton's method** approach (or a related algorithm like Trust Region Reflective, which handles sparse Jacobians) is recommended. Modern sparse implementations of these algorithms are well-suited to the structure of geometric constraint problems.

### 5.5 Notes on Third Party Solvers

Third party solvers appear to make use of more sophisticated methods, but the above illustrates the general concept. Tools such as DCM, Solvespace, and PlaneGCS (FreeCAD) make mention of:

- [Galois Theory](https://en.wikipedia.org/wiki/Galois_theory)[^10]
- Breaking of systems into equations into soluble subsystems [^7]
- Nullspace projection [^11]
- Damped least squares/Levenberg-Marquardt [^12], [^13]
- BFGS[^12], [^13]
- Dogleg [^12], [^13]

## 6. Constraint Solver Diagnostics & Other UI/UX Considerations

For the user to make effective use of the constraint system, some degree of visual feedback is required. This can help the user understand the current state of their sketch, diagnose problems etc.

Ideally, users should be able to quickly identify which elements are fully constrained and which remain free to move. In the even of the sketch becoming overconstrained, the system should also indicate this to the user, ideally highlighting the cause of the issue.

### 6.1 Visual Constraint State Indication

In incumbent CAD systems, constraint status is typically indicated visually for each element, which the element changing colour between under, fully, and over-determined conditions. Direct indication of the number of remaining degrees of freedom may also be useful.

| Element State          | Visual Indicator           | Description                                                                                             |
| ---------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------- |
| **Fully Constrained**  | Black outline .            | Element has no remaining degrees of freedom; position/orientation completely determined by constraints. |
| **Under-Constrained**  | Blue outline.              | Element retains one or more degrees of freedom; can be dragged or modified.                             |
| **Over-Constrained**   | Red outline or highlight.  | Element participates in conflicting constraints that cannot be simultaneously satisfied.                |
| **Driving Constraint** | Standard dimension colour. | Constraint actively controls geometry.                                                                  |
| **Driven Constraint†** | Greyed out.                | Constraint displays current value but does not control geometry.                                        |

†This seems specific to Onshape, but is a great feature.

### 6.2 Constraint Conflict Resolution

When a sketch becomes overconstrained, the system should provide tools to identify and resolve conflicts:

| Diagnostic Feature        | Purpose                                                                        | Implementation Notes                                                                      |
| ------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| **Conflict Highlighting** | Visually identify all constraints participating in the conflict.               | Highlight conflicting constraints in red; may require constraint dependency analysis.     |
| **Constraint History**    | Show order of constraint application to help identify the breaking constraint. | Track constraint addition sequence; highlight most recently added conflicting constraint. |
| **Suggested Fixes**       | Recommend which constraint(s) to remove or modify.                             | May be possible with constraint dependency analysis.                                      |

In the interest of performance, dependency/conflict analysis need only happen when a sketch becomes overconstrained.

### 6.3 Interactive Constraint Management

Users should have control over constraint visibility and management. They should be able to:

- Toggle display of dimensional values, constraint symbols, and construction geometry.
- View which constraints depend on others (e.g., symmetric constraints requiring a reference axis).

## Appendix A: Useful Links

- [D-Cubed DCM 2D](https://plm.sw.siemens.com/en-US/plm-components/d-cubed/2d-dcm/): Industry standard 2D geometric constraint solver, used in effectively all commercial CAD packages.
- [Algebraic Solution for Geometry from Dimensional Constraints](https://papers.cumincad.org/data/works/att/9ad2.content.pdf), by John Owen, original D-Cubed author.
- [Solvespace](https://solvespace.com/index.pl)
- [Sketchflat, A Constraint-Based Drawing Tool](https://cq.cx/dl/sketchflat-internals.pdf), by author of Solvespace.
- [A Geometric Constraint Solver](https://core.ac.uk/download/pdf/4971979.pdf), with some link to D-Cubed.
- [Solving Geometric Constraint Systems](https://books.google.co.uk/books?id=TxRPYdYqIT0C); I have a copy of this at home, happy to work through it with folks if useful.
- [A Comprehensive Evaluation of the DFP Method for Geometric Constraint Solving
  Algorithm Using PlaneGCS](https://hrcak.srce.hr/file/446424)

## Append B: References

[^1]: https://help.autodesk.com/view/fusion360/ENU/?guid=SKT-CREATE-SPLINES
[^2]: http://catiadoc.free.fr/online/cfyugdys_C2/cfyugdys0309.htm
[^3]: http://catiadoc.free.fr/online/sdgug_C2/sdgugbt0104.htm
[^4]: http://catiadoc.free.fr/online/cfyugfss_C2/cfyugfssut0202.htm
[^5]: https://cad.onshape.com/help/Content/sketch-tools-bezier.htm?TocPath=Part%20Studios%7CSketch%20Tools%7C_____17
[^6]: https://help.autodesk.com/view/ACD/2026/ENU/?guid=GUID-899E008D-B422-4DF2-AC8D-1A4F5701ED4E
[^7]: https://cq.cx/dl/sketchflat-internals.pdf
[^8]: https://solvespace.com/tech.pl
[^9]: https://www.cambridge.org/highereducation/books/data-driven-science-and-engineering/6F9A730B7A9A9F43F68CF21A24BEC339#overview, P16
[^10]: https://papers.cumincad.org/data/works/att/9ad2.content.pdf
[^11]: https://www.graphics.rwth-aachen.de/media/papers/conmod.pdf
[^12]: https://hrcak.srce.hr/file/446424
[^13]: https://github.com/CadQuery/PlaneGCS
