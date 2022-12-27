# jtd-derive &emsp; ![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/uint/jtd-derive/rust.yml?branch=main) ![Crates.io](https://img.shields.io/crates/l/jtd-derive) ![Crates.io](https://img.shields.io/crates/v/jtd-derive) ![docs.rs](https://img.shields.io/docsrs/jtd-derive)

Generate [JSON Type Definition](https://jsontypedef.com/) schemas from Rust
types.

# Status

Sort of usable, but lacking important features like better `serde` support.

The API is unstable. Expect breaking changes between minor version bumps.

# Why?

Because _Typedef_ seems really nice in how minimal and unambiguous it is. In
particular, systems that generate JSON-based APIs and related
[IDL](https://en.wikipedia.org/wiki/Interface_description_language) files (with
the expectation those will be used for code generation) could use something like
this. Feature bloat is arguably not a good idea in those sensitive spots.

This crate hopefully makes it a little nicer in that Rust projects can keep
language-agnostic type definitions as Rust code rather than a separate thing
with a different syntax.

# Alternatives

## JSON Schema

JSON Schema is often tauted as the more universally accepted solution. The thing
is, it's a solution to a different problem. JSON Schema is meant to be very
expressive and good for validating JSON data against complex constraints.

If you expect codegen to be a major need for you but want to provide JSON
Schemas as well, consider using _Typedef_ and writing a `Typedef -> JSON Schema`
generator. That way codegen consumers can still benefit from _Typedef_'s
simplicity.

## OpenAPI

`OpenAPI` serves a similar purpose, but is complex and meant to describe
specifically APIs built on top of HTTP (often called "RESTful APIs", though
[that's usually quite silly](https://medium.com/@andrea.chiarelli/please-dont-call-them-restful-d2465527b5c)),
with its paths and methods and all the doodads. In that way, it already has a
way of describing your API's endpoints, whereas if you want to use _Typedef_,
you'll want to embed it in some custom
[IDL](https://en.wikipedia.org/wiki/Interface_description_language) of your
design.

If you're building a "web" api, `OpenAPI` might be worth a look. It seems
complex, but maybe it will make sense for your use case.

If you're not building a "web" API and aren't constrained by the HTTP
vocabulary, you'll probably get more value out of _Typedef_.

# Types supported by `serde`, but not by `jtd_derive`

- unit structs like `struct Foo;`
- tuple structs like `struct Foo(u32, u32)` or `struct Foo()`
  - Newtype structs are an exception. They are represented as the inner value in
    JSON, and as the inner schema in _Typedef_. A struct is considered a newtype
    simply if it has exactly one unnamed field, e.g. `struct Foo(u32)`
- structs in the C struct style, but with no fields, e.g. `struct Foo {}`
- enums with mixed variant "kinds", e.g.
  ```rust
  enum Foo {
      Bar,            // unit variant
      Baz { x: u32 }, // struct variant
  }
  ```
- enums with tuple variants, e.g.
  ```rust
  enum Foo {
      Bar(u32),
      Baz(String),
  }
  ```
- enums with any other `serde` representation than
  [internally tagged](https://serde.rs/enum-representations.html#internally-tagged) -
  that's how _Typedef_ insists enums are represented
- tuples - serialized as potentially heterogenous arrays, but _Typedef_ only
  supports homogenous ones.
- `Bound` - one variant gets serialized as a string, the others as objects.
  Typedef can't support that kind of decadent fancy.
- `Duration` - uses `u64`, which is unsupported by _Typedef_.
- `SystemTime` - same reason as above.
- `PhantomData` - seems silly to try to serialize that! Also no good way to
  specify a null literal in the schema.
- `Result` - `Ok` and `Err` variants usually have different forms, which can't
  be expressed in Typedef.
- `OsStr`, `OsString`, `Path`, `PathBuf` - I don't fully understand the
  subtleties around these types. I'm not sure if it's smart to encourage people
  to use these types at API boundaries other than the Rust FFI. If you'd like to
  discuss, feel free to open an issue describing your use case and thoughts.

This may all seem quite restrictive, but keep in mind the point of _Typedef_
isn't to be vastly expressive and capable of describing anything that can be
described with the Rust type system. The idea is to encourage APIs that are
universal and schemas that are suitable for code generation.

Every bit of expressiveness you're missing here is a breath of relief for your
consumers.

# License

Dual licensed under MIT and Apache 2.0 at your option, like most Rust project.
