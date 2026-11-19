# Should KCL be imperative?

Currently KCl tries to keep all its variables immutable. Should we allow mutation?

## Why immutable

 - Fewer chances of bugs when variables cannot be updated
 - Encourages users to use higher-level constructs like patterns, that can be optimized in both KCL and the engine much more easily than a for loop.
 - Easy to map a specific geometric feature (e.g. a 2D polygon) to its definition in the IDE because a geometric feature is created once and then never changed, so it's fully defined at its source range. If geometry could be updated, we'd need the IDE to show where it was defined AND where the relevant change was made.
 - KCL is about defining geometry, not about modeling a changing system. Mutation doesn't seem necessary.

## Why mutation

 - Mechanical engineers (MEs) will be coming from Python or JS. They will be used to imperative programming with loops and variables. The functional programming patterns (e.g. map and reduce) they'll need to engage with will be unfamiliar.
 - Given that we're already asking MEs to go outside their comfort zone (by coding rather than using the GUI), we should make the language as intuitive as possible. This probably means for loops.
 - To keep KCL performant will require implementing persistant data structures. For example, to keep KCL immutable, Array.push currently clones the array into a new variable, and appends to the cloned array. It'd be much faster to just mutate the array in-place, but that'd require mutation. There are alternative high-performance immutable data structures but we'd need to understand and implement them. Certainly doable, but less immediately simple than reusing Rust's mutable datatypes.
 - Currently the engine is mutable and KCL is not. This leads to mismatches like <https://github.com/kittycad/modeling-app/issues/2728> which will be complicated to solve if we keep the language immutable, and trivial if we allow mutation.
 - Many math formulae that MEs will try to implement are easier as imperative algorithms (citation needed)

 ## Implementation problems

  - Should we allow any immutable variables? E.g. a difference between `const` and `var`?
  - Currently KCL doesn't require any variable declaration i.e. you can do `x = 2`. This is fine in an immutable language, but with mutation it means you can't tell if `x = 2` is reassigning `x` or creating a new variable named `x`. We should consider making `const` and `var` keywords meaningful again, so it's clear if you're creating a new variable or not (and whether it is mutable).
