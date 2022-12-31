//! This crate provides a framework for generating [_JSON Typedef_](https://jsontypedef.com)
//! schemas based on Rust types and how they'd be serialized by
//! [`serde_json`](https://docs.rs/serde_json).
//!
//! In order to be able to generate a schema for a type, it must implement the
//! [`JsonTypedef`] trait. Many types from the Rust standard library already do.
//! To implement [`JsonTypedef`] for your own types, you'll probably
//! want to derive it.
//!
//! Generating a schema is done by creating a [`Generator`](gen::Generator),
//! calling [`Generator::into_root_schema`](gen::Generator::into_root_schema),
//! and finally serializing the resulting [`RootSchema`](schema::RootSchema) object.
//!
//! # Example
//!
//! ```
//! use jtd_derive::{JsonTypedef, gen::Generator};
//!
//! #[derive(JsonTypedef)]
//! struct Foo {
//!     x: u32,
//! }
//!
//! let root_schema = Generator::default().into_root_schema::<Foo>();
//! let json_schema = serde_json::to_value(&root_schema).unwrap();
//!
//! assert_eq!(json_schema, serde_json::json!{ {
//!     "properties": {
//!         "x": { "type": "uint32" }
//!     },
//!     "additionalProperties": true,
//! } });
//! ```

pub mod gen;
pub mod schema;
mod r#trait;
mod type_id;

pub use r#trait::JsonTypedef;
