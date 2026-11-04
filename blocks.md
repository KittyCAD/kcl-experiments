# KCL Blocks

## Summary

Allow KCL functions to declare a block parameter. Callers can then call the function using a trailing block of code, which is syntax sugar for passing the block as the block argument. The block of code is like an anonymous module. How the block is used is determined by the declaring function, which can only be done in Rust for now.

For example:

```
// Define a function in the standard library that accepts a block.
@(impl = std_rust)
fn littleLang(code: Block): number {}

// Call the function, giving it a block argument.
answer = littleLang() {
  width = 10mm
  height = 22mm
}
```

## Motivation

We made a block syntax for sketch-solve. But it's super special. We'd like to generalize it so that it works with assemblies, CAM, drawings, and other future things with their own embedded DSL.

## High-Level Spec

Every function call, in addition to its positional and named parameters inside `()`, can additionally accept a block of code after it inside `{}`, as in `foo() { }`. Inside the curly braces is any statement level KCL code. The block is not executed at the call site.

The function declares that it accepts a code block by declaring a parameter with the built-in type `Block`. The `Block` type is a new opaque type. In the initial release, the only thing that KCL can do with a value of type `Block` is pass it as a value to another function. It can only be called or otherwise used from Rust code.

```
fn augmentedAssembly(name, constraints: Block) {
  setup()

  asm = assembly(block = constraints)

  teardown()
  return asm
}

// Call it.
augmentedAssembly("Car") {
  // constraints here...
}
```

It's an error to call with both a named argument (`constraints` in this case) and a trailing block since it would be equivalent to calling the function with two arguments of the same name.

A block parameter can be optional, like other parameters, using `constraints?: Block`. But currently, specifying a default value is an error due to the fact that KCL currently limits default arguments to literals (strings, numbers, and booleans).

If a function is declared with more than one parameter of type `Block`, then the block argument will be bound to the first one in source order.

If the type `Block` is aliased to another name, e.g. `Code`, then parameters with a type using that new name will also be treated as block parameters. So this will work:

```
type Code = Block

fn augmentedAssembly(constraints: Code) {
  // constraints is a value of type Block.
}

augmentedAssembly() {
  // constraints here...
}
```

A function call with a block argument cannot be called in an if-condition. The `{` would be ambiguous whether it's for the block argument or the then-part of the conditional. The workaround is to assign the result to a temporary variable and use that as the condition. Future work may add parsing of the block inside the call parentheses.

Returning from a block with `return` is an error. This restriction may be lifted in the future.

## Drawbacks

Requiring a block parameter to have a declared type would be a departure from truly optional types in KCL. It would make parameter lists somewhat inconsistent. Types are optional for all other parameters. IMO, this is fine because only advanced KCL code like that in the standard library will be accepting block parameters. Even when user code starts to be more complex, parameter types will generally be encouraged to help tooling and users use the library code.

Making `return` in a block an error would be a breaking change for sketch blocks. We may need to special-case sketch blocks if this isn't added in a major KCL version. But this seems like it would be very rarely used.

## Rationale and Alternatives

My goal is to basically take what we did for sketch blocks, extract it, and generalize it, not invent a brand new thing.

Similar to how other languages like Ruby work, a block is not an anonymous function. This can be seen by looking at how sketch blocks work today in KCL 3. The code block of a sketch block doesn't have a return value. Instead, its outputs are the bindings inside it. In this way, they are closer to anonymous modules.

This is why returning from a block with `return` is an error. Also, see future work for other possible uses of `return`.

Another notable property is that code blocks close over identifiers that they reference, similar to anonymous functions. But unlike functions, their scope can be overridden by executing it in the context of another module, the way sketch blocks are executed in the `std::solver` module. So identifier resolution lookup works in this order. Lookup continues down the list until the name is found.

1. local bindings
2. imported bindings from the opened module (e.g. `std::solver` exports in the case of sketch blocks)
3. lexical parent scope bindings

### Alternative: Parameter Position

Julia, for example, has a `do` block syntax that is a purely syntactic transform from:

```
foo(a, b) do x
  body
end
```

which converts to the first positional argument, as in:

```
foo(x -> body, a, b)
```

The function (`foo` in this case) isn't defined any differently. It just declares a parameter as its first positional parameter.

I don't think this fits with KCL because KCL reserves the first parameter for the unlabeled receiver parameter with the `@` prefix, and other named parameters are not supposed to have an order. KCL could use the last parameter. But KCL supports optional parameters that must be declared after required parameters. How would you declare a required block parameter with other optional parameters?

### Alternative: Parameter Name

Another alternative considered was to go based off of parameter name. But this seemed counter to users defining their own domain language. For example, if we required the block parameter to be called `block`, this would allow users to omit the type. But it would mean that the parameter would not be able to convey self-documenting information about what the parameter represented. This is especially important in cases where code has to deal with multiple blocks, pass them to other functions, etc.

### Alternative: Sigil

Ruby allows either an implicit unnamed block that can be called with `yield` or naming the block parameter with the `&` sigil. It must be the final declared parameter (including after rest args) and prefixed with `&`. There can only be one block parameter.

```
def foo(a, b, &my_block)
end
```

IMO, we generally want to avoid too many sigils in KCL. I have no concrete evidence for this, but my claim is that too many sigils make code unapproachable. We already have `@` for the unlabeled parameter, and it's generally considered a mistake since it collides with annotations like `@settings()`. Adding a sigil would require parser changes. The sigil makes the parameter special. Considering the future possibilities, I generally don't want block parameters to be special, other than some syntactic sugar.

## Prior Art

A lot of the syntax design is based off of how Swift works. Swift doesn't have a special syntax for declaring block parameters. It goes based off the declared parameter type. It allows more than one block parameter. When calling it, the first block can be after the parentheses and unlabeled. This is bound to the first function parameter. Subsequent blocks must be labeled with the parameter name.

For example, a function that accepts two function parameters, `onSuccess` and `onFailure`:

```
foo(a, b) {
  // on success block
} onFailure: {
  // on failure block
}
```

I already discussed how a lot of the semantics are based off of Ruby blocks.

## Future Possibilities

In the future, we will add a way for KCL code to call a block in a module context. This is how we'll allow things like `sketch()` to be implemented, where the block is evaluated with the `std::solver` module opened (i.e. with exported items in scope of the code block). How exactly is left unspecified, but because `Block` is an opaque type, we should be able to define a function that does it.

In the future, we could add a way for functions to control whether engine communication is allowed. `sketch()` disallows engine communication while executing its given block. It would be nice if arbitrary functions, or at least those defined in the standard library, could control this, without it being so special. Perhaps this could be done with an argument to whatever executes the block's code.

In the future, we may also want to add a way for a block to accept arguments. This is out of scope for now. We'll need a way for the block to declare its parameters. We may also need syntax for parameterized types so that the `Block` type can specify its parameter types.

In the future, we may want to allow non-local control flow from within a block using `return`, `continue`, `break`, or other keywords. Similar to Ruby, this allows you to do things like implement for-each in the language, where `break` and `continue` work. It also allows you to refactor in such a way that early `return`s don't change their meaning since `return` can return from the enclosing function, not the block. This can only be added in a backward-compatible way if `return` is disallowed in the initial release.

In the future, we may want to add the ability to call a function with a block argument inside the call parentheses to avoid parsing ambiguity with an if-then conditional. For example:

```
if foo(a, b, block = {
  // block argument
}) {
  // then
} else {
  // ...
}
```

In the future, we may want to allow syntax for passing multiple blocks to a function. This is great for if-then-else-style functions that have both a "then" and an "else" or event style callbacks like "on-success", "on-failure", "always", etc. For example:

```
foo(a, b, onSuccess = {
  // ...
}, onFailure = {
  // ...
})
```

To make it slightly nicer, similar to Swift, block arguments could have optional labels, so the parentheses could be moved and parsed like this:

```
foo(a, b) onSuccess = {
  // ...
} onFailure = {
  // ...
}
```

In the future, we could allow imports that bring into scope bindings from other modules. This may require changes to how imports work. Concurrent module execution as it exists today makes this challenging.
