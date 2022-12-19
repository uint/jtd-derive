# Integration tests

There are broadly two kinds of tests here.

## Pass tests

These currently just use the vanilla Rust toolchain, but that might change. Most
`*.rs` files contain these. Refer to the
[Integration Tests section](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)
of the Rust book for info on how these work.

## Compile-fail tests

The cases can be found under [`derive_errors/`](derive_errors). They're
collected in [`derive_errors.rs`](derive_errors.rs). They use the
[`trybuild`](https://github.com/dtolnay/trybuild) test harness - refer to its
documentation for more info.
