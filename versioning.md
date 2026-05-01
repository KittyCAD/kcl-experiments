# KCL Backwards Compatibility, Breaking Changes, and Versioning

This doc proposes policy for KCL versioning and breaking changes - specifically what constitutes a breaking change and how such changes affect versioning. It also proposes a policy for experimental changes which are special exception to the versioning and breaking change policy.

Note that I talk about libraries in this doc, but they are not yet implemented and the design hasn't been finalised. Hopefully the vague terms used here will fit with any suitable concrete design.


## Summary: key concepts and language features

Language features:

- `kclVersion`: a property of the settings of a KCL file (i.e., declared as part of the `@settings` attribute). Can also be used as an attribute on items in the standard library to denote which versions of KCL can use the item.
- `experimentalFeatures`: a property of the settings of a KCL file, if set to `allow` (or `warn`), then experimental features and library items can be used in the file. A project which includes any files which use experimental features cannot be published as a library.
- `@experimental`: an attribute used in the standard library to mark items as experimental.


Concepts:

- Major breaking change: a breaking change which may affect reasonable, production users
- Minor breaking change: a breaking change which shouldn't affect any reasonable, production users (i.e., only likely to be noticed by contrived code examples, etc.).
- Non-breaking change: a change which cannot break client code.
- Internal KCL version: the version of KCL required by a Zoo program (as specified by its `kclVersion` attributes or defaults).
- Library KCL version: the version of KCL declared in a library's manifest.


## Versioning

I propose that KCL versioning is its own thing, mostly independent of versioning of the app, API, or crates (including the kcl-lib crate - the versioning of that crate reflects the implementation of KCL, not the language itself). We do not follow semver, nor do we necessarily reflect changes in performance or non-breaking changes in KCL version numbers.

A KCL version is a string of the form `X.Y.Z` where each of `X`, `Y`, and `Z` are positive integers, the major, minor, and patch version numbers, respectively. Major breaking changes require a new major version. Minor breaking changes require a new minor or patch version. Significant, non-breaking changes require a change to the patch version (bug fixes, performance changes, changes which don't affect the syntax accepted or the output generated don't count as significant, most other changes do, even if they only make the interpreter slightly more flexible).

Major breaking changes are intended to be rare (annual or, preferably, less frequently), but not unknown.

The KCL version may be specified in a KCL file's settings. The syntax is a string with part of a KCL version, e.g., `""`, `"1"`, `"1.3"`, `"1.2.3"` are all valid version strings. If no version is specified then `""` is assumed as a default for top-level files. For imported files, the declared (or default) version must be compatible with and less than or equally precise than the top-level version. E.g., if the top-level version is `"1.2"` then `""` and `"1"` are valid versions for imported files but `"1.2.1"` (more precise) and `"2"` (incompatible) are invalid. If no version is specified in an imported file, then the version is inherited from the top-level file (i.e., only one version is ever used for a whole project).

Specified parts of the program's KCL version must match the app's supported versions exactly, unspecified parts are not checked. E.g., if the app supports KCL versions 1.2.4 and 1.3.2, then program versions `""`, `"1"`, `"1.2"`, and `"1.3.2"` are supported (not a complete list), but `"2"`, `"0"`, `"1.1"`, `"1.3"`, and `"1.3.1"` are not (also not a complete list).

The version of a library is specified in its manifest. Any declared version in any file in the library must be less specific: major versions must match and minor/patch versions must be strictly older, e.g., if the declared library version is `"1.2"` then `"1.2.0"`, and `"1.1"` are valid internal KCL versions, but `""`, `"1"`, `"2"`, and `"1.2"`, `"1.3.0"` are invalid. If there is a library manifest, but no KCL version setting, then the KCL version of the file is inherited from the library manifest. Similarly, any libraries used by the project must have matching major version and older minor/patch versions.

By the time we release KCL 2.0, we should extend the library version syntax string so that libraries can declare that they support multiple major versions (e.g., 1.2 and 2.0).

When using a library, the internal KCL version must have matching major version and be more recent than the library version of any libraries used. E.g., if a library has version `"1.2"` then it can be used by a program with internal KCL version `""`, `"1"`, `"1.2"`, `"1.3"`, or `"1.2.1"`, but not `"1.1"` or `"2"`. 

Non-experimental items in the standard library may be marked with a `@(kclVersion = "..."")` attribute. Such items are only available in programs with a compatible KCL version. I expect in the future we'll allow a range of versions here to allow deprecating/removing items, rather than just a lower bound.

For example, given

```kcl
@(impl = std_rust, kclVersion = '1.1')
export fn subtract2d(
  /// Which sketch should this path be added to?
  @sketch: Sketch,
  /// The shape(s) which should be cut out of the sketch.
  @(experimental = true)
  tool: [Sketch; 1+],
): Sketch {}
```

A user using version `1.0` cannot use the function at all. A user using `1.1`, `1.2`, `2.0`, etc. can, but the `tool` argument is only available if they opt-in to experimental features (see below).


### Users' perspective

Most users should not have to worry about versioning and will not declare a KCL version anywhere. When we release 2.0, users will opt-in to upgrading by adding `kclVersion = "2"` to any file in their project which is used as a top-level file. Likewise, these versions will need to be updated when upgrading to 3.0, etc. but not with minor releases, which should be mostly transparent to most users. Most non-breaking changes (i.e., most new features) are available as users upgrade their app (without changing declared versions).

Some users may wish to limit the versions or features used in their projects (presumably for some corporate reason they want limited upgrades and/or to ensure compatibility with non-upgrading colleagues). These users can declare a more precise KCL version, e.g., `kclVersion = "1.2.3"`. These users will have to update the version number (or remove it) to use newer KCL features and library items but, not to update their app (within reason).

Users will get useful error messages when trying to use incompatible features, standard library items, or third-party libraries.

Library authors will declare supported library versions and will have to ensure their declared KCL versions and used features are compatible with the library version. I expect this to be a little bit fiddly but not too bad. The best thing to do is for users to use the least specific KCL version setting (i.e., no version, or `"2"` etc., for more recent than version 1 libraries). The interpreter will help by supporting testing with all declared and supported versions, and giving helpful error messages.


## Experimental features

KCL language features and library items may be marked as experimental (language features are 'marked' implicitly in the interpreter implementation). These features and items are subject to change and exempt from the usual versioning and backwards compatibility rules. Users must opt-in to using these features or items using `experimentalFeatures` in each file in which they're used. A project that has any file with `experimentalFeatures` cannot be published as a library. Experimental features do not affect the version of the program.

The `experimentalFeatures` property accepts `allow`, `warn`, and `deny`; `deny` is the default. Using `experimentalFeatures = allow` will generate a warning on the property only. The property applies only to the file in which it is used.

Functions, types, constants, and arguments in the standard library can be marked with an `@(experimental = true)` attribute. Such items can only be used if the user has opted-in to experimental features. It is not possible for a user to label a feature as experimental (at least for now, long-term it should be supported).


## Breaking changes and backwards compatibility

This policy is a guide to help us make decisions around breaking changes that will be optimal for our users. The goal is not to stick to a theoretically perfect definition of breaking changes, the goal is a good user experience. There is never a perfect policy for breaking changes and there are always gray areas. Do not be pedantic about sticking to or creating perfect rules, they do not exist.

Note that there are plenty of changes which are not breaking and are still bad. Just because a change is not breaking doesn't mean there is no reason not to do it. We should not try and make a breaking change policy be the guiding policy for all changes.


### What is a breaking change?

A change to KCL is breaking if any program that runs without any errors before the change, produces any error (including those due to *reasonable* assertions) or produces a different *result* after the change.

There are some important caveats:

- The result of a KCL program is due to an interaction between the KCL interpreter and engine (but not the rest of the frontend, see below). "Errors" includes both interpreter errors and engine errors.
- The "result" of executing a program is the output of exporting the model and the *significant parts* of the rendered video of the scene. It does not include any interpreter-internal representation of the program or intermediate state, even if that can be observed by the user (e.g., in the debug-pane of the frontend, or using asserts or logging functionality, etc.). There may be contrived ways to get internal state into a KCL string (or similar) and then to display it some how, changing output does not count as part of the result of the program for the purposes of determining if a change is breaking.
- The errors or changes to results do not have to have a direct relation to the change in question. If there is a change which is breaking in the case of a program, but that program is impossible to construct before the change without an error, then the change is not breaking.
- A reasonable assertion is one with tolerance which excludes minor floating point changes, does not include any internal state, and is not clearly contrived.
- We don't mention warnings, see below.

The following are examples of breaking changes:

- Code which ran and produced a result produces a significantly different result or no result or an error (even if the result is the same).
- Code which ran and produced a warning now produces an error.
- Code which ran now triggers a reasonable assertion.

The following are examples of changes which are not breaking:

- Code which always caused a KCL error, no longer causes an error and produces some output.
- Code which always caused one error, now causes a different error.
- Code which ran correctly now causes a warning.
- Code which previously caused an engine error, now produces a KCL error (or vice versa).
- Code which ran with an error but still produces an output, now produces a different output or no output at all.
- Code which produces a result, produces a different result in some non-significant way, e.g., a different colour when the colour isn't specified, with different camera properties, or helper decorations, etc. (yes, "significant" is doing heavy lifting here, it's outside the scope of this doc. We're grown-ups, we can work it out).
- Code which ran with no errors, but didn't produce a result, now produces a result or now produces an error.
- Any change to performance (better or worse), unless the change is so negative that a reasonable program that used to run in a reasonable time now takes an unreasonable amount of time to run.

If there are changes to both the KCL interpreter and the engine, if the user should always be using 'matching' versions and all code is non-breaking if using the new versions of both, then the set of changes is non-breaking, even if using one version or the other would be breaking. Note that there is work that must be done to ensure that users always get those matching versions.

Here is a non-exhaustive list of things which are never breaking changes:

- Any change to any tooling which does not directly affect the result of the program (e.g., a change to how an error is presented to the user is never breaking). 
- Any change to the source range of a function or other item.
- Any change to documentation, signature help, or other presentation of a program's content.
- Any change to the ordering of named arguments, or other changes to ordering which do not affect the result of program execution.
- Adding an optional argument to a function with a default value which preserves the previous behaviour.


### Exceptions

#### Bug fixes

Behaviour which is obviously a bug (from the perspective of a user) can be fixed and considered non-breaking.

#### Trivial breaking changes

Some changes are technically breaking, but are unlikely to affect any users (note *any users*, a change which affects a very small number of users is a different category of change). For example, it might require a contrived combination of syntax, or require an unreasonable program or inputs. Such changes do not count as breaking. However, be aware that tools (including but not limited to our AI tools) can create programs that no reasonable human would, and those programs must still not be broken by changes.

#### Name clashes

Introducing a name to a namespace is a breaking change since it would cause a duplicate name error in a program which already uses that name. Note that adding a name to the prelude is not a breaking change since local names take priority over names in the prelude. We can also assume that any name starting with `__` or a name which is long and random (e.g., a UUID) is not used.

#### Warnings

Just because there is a warning in a program does not make a change non-breaking (as an error would). However, if there has been a specific warning that a change will happen and that warning has been there for the whole duration of the version, then we *may* make the change despite it being breaking and still not be violating our backwards compatibility guarantee. Similarly, we might consider an otherwise major breaking change as minor in this case (preferable to considering a change non-breaking).

For example, if there was a warning that first appeared in version 1.0, and was maintained throughout 1.1, then that warning could be changed to an error in 1.2 and it would not be considered a violation with respect to 1.1 (but it would with respect to 1.0). Note that even with a warning, if users are still using a feature, then the change could be negative for our users and we should take that into account when decision making. If the change is more complex than simply changing a warning into an error, then a lot more thought is required.


### When should we make breaking changes?

Ideally, we do not make breaking changes! However, often we can offer a better user experience in the long-term by making breaking changes. However, there is a trade-off with the negatives caused to existing users in the short-term. We should always try to find non-breaking alternatives or to make the breakage as painless as possible.

Not all breaking changes are equal. Consider:

- How many users are likely to be affected?
- How obvious is the change? Generally, more obvious changes are easier for users to find and fix. Silently changing output is generally bad.
- How much effort is required to fix the breakage? Can this be reduced with tooling?
- How much thought is required to fix or recognise the breakage?
- Can the breakage be effectively communicated?
- Is the breakage opt-in? For how long can users not opt-in and what happens after that time?
- Can the user opt-out of the change? For how long?
- Is the benefit of the change obvious?
- Is there an alternative? Is it already supported? Can users be moved to the alternative before the breaking change happens?

To simplify, consider classifying breaking changes according to: severity in terms of user impact (taking into account mitigations such as tool support), whether the breaking change is opt-in or not (note that this document does not specify any mechanism for opting-in to breaking changes other than the major version number, however, we may add some additional mechanism in the future).

There is clearly a spectrum of severity for breaking changes, but for the sake of versioning there is a binary differentiation between major and minor breaking changes. Generally, a minor breaking change should not affect any real, production users, any breaking change which is likely to affect users is major. Note that this distinction is similar to that for 'trivial breaking change' above. A trivial breaking change may be regarded as non-breaking or minor. The correspondence is not perfect because this is ultimately a judgement call rather than a technical property. A change where we are very confident it won't affect any users is trivial and we can consider it non-breaking for versioning. A change where we are less confident, or where there is a very small exception[^exception] we may consider minor but not trivial.

We should be intentional and precise around breaking changes. They should be documented and communicated to users. We should document the changes required to migrate code from earlier versions to later versions  (and tools where possible). We must avoid a situation where a user has code from an earlier version and the migration path to the current version is unclear for them or us.

[^exception]: To be a bit more precise about this, if we expect that a change will affect a very small number of real programs, then it should be considered  major breaking change, even if the number of real programs is very small. If we consider that there is a small number of potential real programs which would be broken, but it is unlikely that any such programs exist, then we may consider it minor. If we consider that there a no real programs, but some contrived ones, then we many consider it trivial.

