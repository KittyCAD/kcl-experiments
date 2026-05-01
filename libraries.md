# Libraries

The goal is to support importing parts and functions from distributed libraries.

Example:

```kcl
import screws, bolts, lPlate from 'git@github.com:KittyCAD/kcl-parts.git'

lPlate |> scale(x = 0.5)

screws::fourMM(1, 0, 4)
screws::fourMM(1, 1, 4)
screws::fourMM(0, 1, 4)
screws::fourMM(0, 0, 4)

bolts::sixMMg(0.5, 0.5, 3)
```

## User syntax

We use `import` in the same way as we currently do for importing files from the same project, with the difference that the import string is a URI rather than a path, e.g., `import screws from 'git@github.com:KittyCAD/kcl-parts.git'`. There are two extensions to the current syntax:

- more supported import strings (URIs)
- more supported attributes

### Import strings

Initially we support Git repos with URIs starting with `git@` and local URLs starting with `file:`. File URLs aren't expected to adhere strictly to their spec, and we can support relative paths, etc. File URLs must point to a directory, not an individual file.

Note that in both cases the import path points to a library (whether a Git repo or a local directory), not an individual file.

In the future, we might want to support purpose-designed library formats and/or a package manager or library registry, in any case we just need to add another kind of supported URI, e.g., `kcl:`.

I imagine that a subset of URIs (perhaps just Zoo-owned Git repos) are white-listed for support in the web app and others are only supported in the downloaded app.

Downloading a Git URL should use the user's SSH keys, exactly how is not spec'ed for now.

### Attributes

For Git libraries:

- `version`: a string corresponding to a published tag (cannot be specified as well as `commit`). Treated as an exact version, we don't apply semver rules, etc.
- `commit`: a string identifying a specific commit (cannot be specified as well as `version`)

E.g., 

```kcl
@(version = "1.2")
import foo, bar from 'git@github.com:KittyCAD/kcl-parts.git'
```

## Library syntax

A library is a directory which contains `lib.kcl`. It can contain any other files or directories. When KCL accesses a library, it will download the whole library (including subdirectories), but the only file which is used directly is `lib.kcl`.

`lib.kcl` is a regular (though distinguished) KCL file. It may contain any code, including exports and imports which allows re-exporting code from other files in the library. Only code exported from `lib.kcl` is accessible to users of the library. This can include functions, constants, types, and parts in the usual ways.

Metadata about a library (e.g., license, description, website, author, etc) are specified in a `@libMetadata` attribute in `lib.kcl`. The fields of this attribute are left for unspec'ed. This metadata should not include a version.

It is an error for anything exported from `lib.kcl` to depend on another library which is local (defined using a `file:` URI) or does not have a `version`. This means that client code which depends only on versioned libraries is guaranteed not to change over time.

## Rationale

The easiest thing would be to just allow using a URL to a file and not have the concept of libraries at all. However, this has some disadvantages:

- the pointed-to file can change over time, meaning users are surprised by changes and KCL will need to check and download the file on every execution. This problem is solved by versioning, which requires restricting the URLs we can point to (to those like Git which include some notion of versioning). It would be nicer to use the https URL to a GitHub repo to allow for easy copy/pasting, however, that is difficult to verify that the URL points to a Git repo and makes downloading the Git repo more difficult. Furthermore, using Git rather than a generic URL makes it easier to support public and private libraries, etc.
- The pointed-to file might be private/internal to a project, or the author might not intend for it to be publicly used. The author should opt-in to publishing a library and should opt-in to exporting individual items from the project. Rather than adding more visibility syntax (on top of `export` and default privacy), I think requiring `lib.kcl` (and `@libMetadata`) is an easier way to opt-in to being a library and to exporting items. Furthermore, it means users do not need to know about the internal structure of a library.
- It makes white-listing and caching libraries more difficult.

I expect that there will be a single import per library (unless the user wants multiple versions of the same library). That means using attributes should not be too onerous, nor using URLs directly too verbose (as opposed to having the URL and version in a manifest and referring to the library via an identifier). Together with the syntax for submodules and paths currently being implemented, I think this works nicely, e.g.,

```kcl
import screws, bolts, lPlate from 'git@github.com:KittyCAD/kcl-parts.git'

lPlate |> scale(x = 0.5)

screws::fourMM(1, 0, 4)
screws::fourMM(1, 1, 4)
screws::fourMM(0, 1, 4)
screws::fourMM(0, 0, 4)

bolts::sixMMg(0.5, 0.5, 3)
```

A possible extension would be to allow paths in imports rather than just names, to permit `import screws::fourMM from 'git@github.com:KittyCAD/kcl-parts.git'` or even Rust-style import groups, e.g., `import screws::{fourMM, sixMM} from 'git@github.com:KittyCAD/kcl-parts.git'`.
