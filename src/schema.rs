//! Internal Rust representation of a JSON Typedef schema.

use std::collections::HashMap;

use serde::Serialize;

// All this corresponds fairly straightforwardly to https://jsontypedef.com/docs/jtd-in-5-minutes/

/// The top level of a Typedef schema.
#[derive(Debug, PartialEq, Eq, Clone)]
struct RootSchema {
    definitions: HashMap<String, Schema>,
    schema: Schema,
}

/// A Typedef schema.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Schema {
    metadata: Metadata,
    ty: SchemaType,
    nullable: bool,
}

/// The 8 "forms" a schema can take.
#[derive(Debug, PartialEq, Eq, Clone)]
enum SchemaType {
    Empty,
    Type(TypeSchema),
    Enum(Vec<String>),
    Elements(Box<Schema>),
    Properties {
        properties: HashMap<String, Schema>,
        optional: HashMap<String, Schema>,
        additional: bool,
    },
    Values(Box<Schema>),
    Discriminator {
        discriminator: String,
        // Can only contain non-nullable "properties" schemas
        mapping: HashMap<String, Schema>,
    },
    Ref(String),
}

/// Typedef primitive types. See [the Typedef docs entry](https://jsontypedef.com/docs/jtd-in-5-minutes/#type-schemas).
#[derive(Debug, PartialEq, Eq, Clone)]
enum TypeSchema {
    Boolean,
    String,
    Timestamp,
    Float32,
    Float64,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
}

/// Schema metadata.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Metadata(HashMap<String, serde_json::Value>);
