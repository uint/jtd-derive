# jtd-derive

Generate [JSON Type Definition](https://jsontypedef.com/) schemas from Rust
types.

## Status

WIP - it doesn't work yet.

## Types `serde` supports, but we don't

- tuples - serialized as potentially heterogenous arrays, but _Typedef_ only
  supports homogenous ones.
- `Bound` - one variant gets serialized as a string, the others as objects.
  Typedef can't support that kind of decadent fancy.
- `Duration` - uses `u64`, which is unsupported by _Typedef_.
- `SystemTime` - same reason as above.
- `PhantomData` - seems silly to try to serialize that! Also no good way to
  specify a null literal in the schema.
- `Result` - `Ok` and `Err` variants could have different forms. Also chances
  are we should discourage leaking Rust error types across API boundaries
  anyway. Right?

## License

Dual licensed under MIT and Apache 2.0 at your option, like most Rust project.
