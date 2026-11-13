# KCL functions and their arguments

Proposal to generalize and possibly simplify KCL function calls with
labelled, unlabelled arguments and how they bind to a function's
parameters.

## Why

1. A more unified way to describe labelled and unlabelled will be
    1. easier to explain to new users, especially new to coding
    1. easier to explain to new users with prior coding knowledge
    1. allow more variety to express function calls unambiguously
1. Remove the limitation we have on unlabelled parameters for only the
   first function parameter
1. Remove extra syntax (prepending a `@` to only the first function
   parameter)

### Benefits

1. Allow arguments in the form of a KCL object to be passed to
   functions.

   ```typescript
    fn f(a,b,c) { ... }

    // we can make the following function calls behave the same

 f(a = 1, b = 2, c = 3)

 // extra check for function with 1 argument that is of type KCL object
 f({ a = 1, b = 2, c = 3 })

 // potential syntactic convenience drop the parenthesis and pass
 // the object value directly to the function
 f{ a = 1, b = 2, c = 3 }

    // "deconstructs" the arguments
 fargs = { a = 1, b = 2, c = 3 }
 f(args)

 // used with pipe (|>) to "apply" the arguments
 result = args
 |> f()

   ```

1. Refactor pipe expressions (`|>`) that use `%` to designate argument
   for passed in value to no longer use `%`. You can label all args
   but the one that is passed `%`. Pipe expression defaults to first
   argument for passed in value that will correctly bind the passed in
   value to the correct parameter. I think this refactoring is
   *always* possible but need to prove it.

## Current

Function definitions in KCL allow for parameters to be

* labelled,
  * A label with the same value as the parameter name is created as
      the label, KCL default e.g., `fn f(x) { ... }`. Creates a label
      `x` for the parameter named `x`.
* unlabelled,
  * *only the first function parameter*, e.g., `fn f(@x, y, ...) {
 ... }`

* optional
  * appear *after* all required parameters, e.g., `fn f(a, b, c?,
      d?) { ... }`
* default value
  * Only on optional parameters, e.g., `fn f(a? = 1) { ... }`.

The case analysis on all options gives us

| Labelled | Optional | Default | Valid? | Example         | Notes            |
|----------|----------|---------|--------|-----------------|------------------|
| True     | True     | True    | True   | `fn f(x? = 1)`  |                  |
| True     | True     | False   | True   | `fn f(x?)`      |                  |
| True     | False    | False   | True   | `fn f(x)`       |                  |
| False    | True     | True    | True   | `fn f(@x? = 1)` | First param only |
| False    | True     | False   | True   | `fn f(@x?)`     | First param only |
| False    | False    | False   | True   | `fn f(@x)`      | First param only |

Labelled and unlabelled parameters alter the way a function call is
written.

* labelled arguments

  ```typescript
   f(length = 1)
  ```

* unlabelled arguments

  ```typescript
   f(1)
  ```

At function call the order of labelled arguments is not required to
match the order of the function's parameters.

NOTE: The unlabelled argument case is a function call site
restriction. There is a label for the parameter.

## Proposed

The claim here is that we can drop the notion of unlabelled
parameters, i.e., all parameters are labelled with the label being the
same value as the name of the parameter.

Now we can allow for function call sites to use labelled or unlabelled
arguments but also interleave argument order freely.

The only change to function definitions is that we can now drop the
`@` symbol to denote unlabelled first parameters.

For function definitions

1. No need for `@`
1. All optional parameters are defined *after* required
   parameters. *Same restriction as today*
1. Default values are optionally allowed on optional parameters
   only. *Same restrictiopn as today*

For function calls, arguments can be provided with or without labels
and the interpreter will bind argument values to parameter names using
the following process

1. for all labeled arguments we bind the value to the corresponding
   labelled parameter. We call these bound parameters **filled**.
1. for remaining unlabelled arguments we bind the argument values
   to the **unfilled** parameters in a **left-to-right** order of the
   **parameter list**.

### Use cases

Consider

```typescript
fn f(a, b, c?, d? = 0) { ... }
```

Then we have

1. All labels provided

   ```typescript
   f(a = 1, b = 2, c = 3, d = 4)

   binds

   a -> 1
   b -> 2
   c -> 3
   d -> 4
   ```

1. Required with labels no optionals

   ```typescript

 f(a = 1, b = 2)

 binds

 a -> 1
 b -> 2
 c -> none
 d -> 0

   ```

1. Required provided by position no optional

   ```typescript
 f(1, 2)

 binds

 a -> 1
 b -> 2
 c -> none
 d -> 0

   ```

1. First required by position, rest required by label no optional

   ```typescript

 f(1, b = 2)

 binds

 a -> 1
 b -> 2
 c -> none
 d -> 0

   ```

1. First required with label, second required by position no optional

   ```typescript
 f(a = 1, 2)

 binds

 a -> 1
 b -> 2
 c -> none
 d -> 0

   ```

1. All positional

   ```typescript

 f(1, 2, 3, 4)

 binds

 a -> 1
 b -> 2
 c -> 3
 d -> 4

   ```

1. All position with missing optional

   ```typescript
 f(1, 2, 3)

 binds

 a -> 1
 b -> 2
 c -> 3
 d -> 0

   ```

1. Required as positional, optional labelled and missing optional

   ```typescript

 f(1, 2, d = 10)

 binds

 a -> 1
 b -> 2
 c -> none
 d -> 10

   ```

1. All provided unlabelled and 1 required with label and out of order

   ```typescript
 f(2, 3, 4, a = 100)

 binds

 a -> 100
 b -> 2
 c -> 3
 d -> 4

   ```

The last case comes across as odd. The algorithm however can figure it
out.

### Algorithm

Abstractly, a function's parameter list can be viewed as an ordered
indexed family. Parameter list and argument list are collections that
have 2 functions used to index it

* index: R -> Values, where R is 0..n, and n is the length of the
parameter list and Values the elements of the collection (parameter
for parameter list, argument for argument list)

* label: L -> Values, where L is the set of Labels and Values is the
same as for index: above

A function's argument list while an ordered indexed collection as well
it will require some processing

* if no labels at all then we rely on the index function and we bind
  argument to parameter by position using index: function

* if some labels then

 1. Bind argument to parameter for all labelled arguments using
       label: function
 1. we perform a **stable** filter of the labelled entries from
  both parameter list and argument list. Stable here means indexes are
  not *updated* or *altered* for the elements. The filtered entries are
  dropped and nothing else is altered.  Then fall back to index:
  function and bind args to parameter by index.

* if all labelled then we have a full label: function and index:
  function so we can bind arguments to parameters by label

Computing correct number of arguments, as well as any runtime type
checks remain unchanged.

# Bibliography

* [Garringe, 2001, POPL](https://caml.inria.fr/pub/papers/garrigue-labels-ppl01.pdf)
