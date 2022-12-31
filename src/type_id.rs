/// Provide a unique [`TypeId`] for the given concrete type.
///
/// The ID should reliably identify a type across [`type_id`] calls within one
/// run of a binary, but not across runs. This means it's not suitable for exporting
/// in public APIs, but can be useful internally. `jtd-derive` uses it during schema
/// generation to detect name collisions of schema definitions.
///
/// Inspiration: [`GREsau/schemars#178`](https://github.com/GREsau/schemars/pull/178)
pub(crate) fn type_id<T: ?Sized>() -> TypeId {
    TypeId(type_id::<T> as usize)
}

/// An ID uniquely identifying a concrete type.
///
/// The ID should reliably identify a type across [`type_id`] calls within one
/// run of a binary, but not across runs. This means it's not suitable for exporting
/// in public APIs, but can be useful internally. `jtd-derive` uses it during schema
/// generation to detect name collisions of schema definitions.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct TypeId(usize);
