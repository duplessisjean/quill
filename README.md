<h1 align="center">Quill</h1>

<div align="center">
🪶✍️📜🖋️🦢
</div>
<div align="center">
  <strong>Scope extensions for <code>TOML</code></strong>
</div>
<div align="center">
   A preprocessor for scopes in <code>TOML</code> to permit multiple simultaneous "files" in one. 
</div>


## 📜 Table of Contents
- [<code>🪶 About Quill</code>](#about-quill)
- [<code>🧾 License</code>](#license)
- [<code>🎓 Acknowledgments</code>](#acknowledgments)

<a name="about-quill"></a>
## 🪶 About Quill 

Quill provides an extension to ``TOML`` that allows for *scopes*
these are chunks of files which can be extracted, essentially permitting
multiple TOML files with shared components inside of one.

No ``TOML`` parsing is done by Quill, it simply extracts scopes from the
file as requested by the user of the library and returns the TOML content
with the scopes extracted.

### Scopes

Scopes are defined in the file by prepending a non-whitespaced string that
may only contain ASCII letters, ASCII digits, underscores, and dashes with a ``@``

For example,

```toml
@my_scope
```

Scope's can be used multiple times in one file, each definition refers to the same
scope and will be returned in the output targeting that one scope.

Multiple scope's can be declared on the same line to mark the proceeding content as
under both scopes equivalently, thus for retrieving any of the scope's on the same line,
the proceeding content will be included.

### Global Scopes

By default all of the elements defined prior to a scope being declared, fall into
the ``global`` scope, All ``global`` scope elements will be included on lookup of any
other scope.

# Example

```rust
use quill::{extract_scope, Scope};

let toml = r#"
title = "App"

@dev
debug = true

@prod
optimized = true

@dev @test
extra_checks = true

@global
do_tests = true"#;

let dev_config = extract_scope(toml, Scope::DefinedScope("dev")).unwrap();
assert_eq!(dev_config, r#"
title = "App"


debug = true





extra_checks = true


do_tests = true"#);
```

<a name="license"></a>
## 🧾 License

This repository/``quill`` is under the MIT license. view it in the ``LICENSE`` file in the root directory of the repository.

<a name="acknowledgements"></a>
## 🎓 Acknowledgments

- Thanks to Tom for ``TOML``!
- Thank you for reading this README/Learning about quill! ❤️

<br>

-------------

[Quill Authored/Created by Aurore du Plessis](https://github.com/duplessisaurore/quill)

Love for everyone 💛 
