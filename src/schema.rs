//! The internal Rust representation of a [_JSON Typedef_](https://jsontypedef.com/)
//! schema.

use std::collections::BTreeMap;

use serde::Serialize;

// All this corresponds fairly straightforwardly to https://jsontypedef.com/docs/jtd-in-5-minutes/
// I'd normally try to separate the serialization logic from the Rust representation, but using
// serde derives makes this so very easy. Damnit.

/// The top level of a [_JSON Typedef_](https://jsontypedef.com/) schema.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct RootSchema {
    /// The top-level
    /// [definitions](https://jsontypedef.com/docs/jtd-in-5-minutes/#ref-schemas).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub definitions: BTreeMap<String, Schema>,
    /// The top-level schema.
    #[serde(flatten)]
    pub schema: Schema,
}

/// A [_JSON Typedef_](https://jsontypedef.com/) schema.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Schema {
    /// The [metadata](https://jsontypedef.com/docs/jtd-in-5-minutes/#the-metadata-keyword).
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
    /// The actual schema.
    #[serde(flatten)]
    pub ty: SchemaType,
    /// Whether this schema is nullable.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub nullable: bool,
}

impl Default for Schema {
    /// Provides an [empty schema](https://jsontypedef.com/docs/jtd-in-5-minutes/#empty-schemas).
    /// Empty schemas accept any JSON data.
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            ty: SchemaType::Empty,
            nullable: false,
        }
    }
}

/// The 8 forms a schema can take. For more info
/// [see here](https://jsontypedef.com/docs/jtd-in-5-minutes/#what-is-a-json-type-definition-schema).
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(untagged)]
pub enum SchemaType {
    Empty,
    Type {
        r#type: TypeSchema,
    },
    Enum {
        r#enum: Vec<&'static str>,
    },
    Elements {
        elements: Box<Schema>,
    },
    #[serde(rename_all = "camelCase")]
    Properties {
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        properties: BTreeMap<&'static str, Schema>,
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        optional_properties: BTreeMap<&'static str, Schema>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        additional_properties: bool,
    },
    Values {
        values: Box<Schema>,
    },
    Discriminator {
        discriminator: &'static str,
        // Can only contain non-nullable "properties" schemas
        mapping: BTreeMap<&'static str, Schema>,
    },
    Ref {
        r#ref: String,
    },
}

/// Typedef primitive types. See [the Typedef docs entry](https://jsontypedef.com/docs/jtd-in-5-minutes/#type-schemas).
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeSchema {
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

impl TypeSchema {
    pub const fn name(&self) -> &'static str {
        match self {
            TypeSchema::Boolean => "boolean",
            TypeSchema::String => "string",
            TypeSchema::Timestamp => "timestamp",
            TypeSchema::Float32 => "float32",
            TypeSchema::Float64 => "float64",
            TypeSchema::Int8 => "int8",
            TypeSchema::Uint8 => "uint8",
            TypeSchema::Int16 => "int16",
            TypeSchema::Uint16 => "uint16",
            TypeSchema::Int32 => "int32",
            TypeSchema::Uint32 => "uint32",
        }
    }
}

/// Schema [metadata](https://jsontypedef.com/docs/jtd-in-5-minutes/#the-metadata-keyword).
///
/// Metadata is a freeform map and a way to extend Typedef. The spec doesn't specify
/// what might go in there. By default, `jtd_derive` doesn't generate any metadata.
#[derive(Default, Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Metadata(BTreeMap<&'static str, serde_json::Value>);

impl Metadata {
    /// Construct a [`Metadata`] object from something that can be converted
    /// to the appropriate hashmap.
    pub fn from_map(m: impl Into<BTreeMap<&'static str, serde_json::Value>>) -> Self {
        Self(m.into())
    }

    /// Returns `true` if there are no metadata entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<A> Extend<A> for Metadata
where
    BTreeMap<&'static str, serde_json::Value>: Extend<A>,
{
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn empty() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Empty,
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(serde_json::to_value(&repr).unwrap(), serde_json::json!({}))
    }

    #[test]
    fn primitive() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Type {
                    r#type: TypeSchema::Int16,
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({"type": "int16"})
        );
    }

    #[test]
    fn nullable() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Type {
                    r#type: TypeSchema::Int16,
                },
                nullable: true,
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({"type": "int16", "nullable": true})
        );
    }

    #[test]
    fn metadata() {
        let repr = RootSchema {
            schema: Schema {
                metadata: Metadata::from_map([
                    ("desc", json!("a really nice type! 10/10")),
                    ("vec", json!([1, 2, 3])),
                ]),
                ty: SchemaType::Type {
                    r#type: TypeSchema::Int16,
                },
                nullable: false,
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({"type": "int16", "metadata": {"desc": "a really nice type! 10/10", "vec": [1, 2, 3]}})
        );
    }

    #[test]
    fn r#enum() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Enum {
                    r#enum: vec!["FOO", "BAR", "BAZ"],
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({ "enum": ["FOO", "BAR", "BAZ" ]})
        )
    }

    #[test]
    fn elements() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Elements {
                    elements: Box::new(Schema {
                        ty: SchemaType::Enum {
                            r#enum: vec!["FOO", "BAR", "BAZ"],
                        },
                        nullable: true,
                        ..Schema::default()
                    }),
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({ "elements": { "enum": ["FOO", "BAR", "BAZ" ], "nullable": true} })
        )
    }

    #[test]
    fn properties() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Properties {
                    properties: [
                        (
                            "name",
                            Schema {
                                ty: SchemaType::Type {
                                    r#type: TypeSchema::String,
                                },
                                ..Schema::default()
                            },
                        ),
                        (
                            "isAdmin",
                            Schema {
                                ty: SchemaType::Type {
                                    r#type: TypeSchema::Boolean,
                                },
                                ..Schema::default()
                            },
                        ),
                    ]
                    .into(),
                    optional_properties: [].into(),
                    additional_properties: false,
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({
                "properties": {
                    "name": { "type": "string" },
                    "isAdmin": { "type": "boolean" }
                }
            })
        )
    }

    #[test]
    fn properties_extra_additional() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Properties {
                    properties: [
                        (
                            "name",
                            Schema {
                                ty: SchemaType::Type {
                                    r#type: TypeSchema::String,
                                },
                                ..Schema::default()
                            },
                        ),
                        (
                            "isAdmin",
                            Schema {
                                ty: SchemaType::Type {
                                    r#type: TypeSchema::Boolean,
                                },
                                ..Schema::default()
                            },
                        ),
                    ]
                    .into(),
                    optional_properties: [(
                        "middleName",
                        Schema {
                            ty: SchemaType::Type {
                                r#type: TypeSchema::String,
                            },
                            ..Schema::default()
                        },
                    )]
                    .into(),
                    additional_properties: true,
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({
                "properties": {
                    "name": { "type": "string" },
                    "isAdmin": { "type": "boolean" }
                },
                "optionalProperties": {
                    "middleName": { "type": "string" }
                },
                "additionalProperties": true
            })
        )
    }

    #[test]
    fn values() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Values {
                    values: Box::new(Schema {
                        ty: SchemaType::Type {
                            r#type: TypeSchema::Boolean,
                        },
                        ..Schema::default()
                    }),
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({ "values": { "type": "boolean" }})
        )
    }

    #[test]
    fn discriminator() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Discriminator {
                    discriminator: "eventType",
                    mapping: [
                        (
                            "USER_CREATED",
                            Schema {
                                ty: SchemaType::Properties {
                                    properties: [(
                                        "id",
                                        Schema {
                                            ty: SchemaType::Type {
                                                r#type: TypeSchema::String,
                                            },
                                            ..Schema::default()
                                        },
                                    )]
                                    .into(),
                                    optional_properties: [].into(),
                                    additional_properties: false,
                                },
                                ..Schema::default()
                            },
                        ),
                        (
                            "USER_PAYMENT_PLAN_CHANGED",
                            Schema {
                                ty: SchemaType::Properties {
                                    properties: [
                                        (
                                            "id",
                                            Schema {
                                                ty: SchemaType::Type {
                                                    r#type: TypeSchema::String,
                                                },
                                                ..Schema::default()
                                            },
                                        ),
                                        (
                                            "plan",
                                            Schema {
                                                ty: SchemaType::Enum {
                                                    r#enum: vec!["FREE", "PAID"],
                                                },
                                                ..Schema::default()
                                            },
                                        ),
                                    ]
                                    .into(),
                                    optional_properties: [].into(),
                                    additional_properties: false,
                                },
                                ..Schema::default()
                            },
                        ),
                        (
                            "USER_DELETED",
                            Schema {
                                ty: SchemaType::Properties {
                                    properties: [
                                        (
                                            "id",
                                            Schema {
                                                ty: SchemaType::Type {
                                                    r#type: TypeSchema::String,
                                                },
                                                ..Schema::default()
                                            },
                                        ),
                                        (
                                            "softDelete",
                                            Schema {
                                                ty: SchemaType::Type {
                                                    r#type: TypeSchema::Boolean,
                                                },
                                                ..Schema::default()
                                            },
                                        ),
                                    ]
                                    .into(),
                                    optional_properties: [].into(),
                                    additional_properties: false,
                                },
                                ..Schema::default()
                            },
                        ),
                    ]
                    .into(),
                },
                ..Schema::default()
            },
            definitions: BTreeMap::new(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({
                "discriminator": "eventType",
                "mapping": {
                    "USER_CREATED": {
                        "properties": {
                            "id": { "type": "string" }
                        }
                    },
                    "USER_PAYMENT_PLAN_CHANGED": {
                        "properties": {
                            "id": { "type": "string" },
                            "plan": { "enum": ["FREE", "PAID"]}
                        }
                    },
                    "USER_DELETED": {
                        "properties": {
                            "id": { "type": "string" },
                            "softDelete": { "type": "boolean" }
                        }
                    }
                }
            })
        )
    }

    #[test]
    fn r#ref() {
        let repr = RootSchema {
            schema: Schema {
                ty: SchemaType::Properties {
                    properties: [
                        (
                            "userLoc",
                            Schema {
                                ty: SchemaType::Ref {
                                    r#ref: "coordinates".to_string(),
                                },
                                ..Schema::default()
                            },
                        ),
                        (
                            "serverLoc",
                            Schema {
                                ty: SchemaType::Ref {
                                    r#ref: "coordinates".to_string(),
                                },
                                ..Schema::default()
                            },
                        ),
                    ]
                    .into(),
                    optional_properties: [].into(),
                    additional_properties: false,
                },
                ..Schema::default()
            },
            definitions: [(
                "coordinates".to_string(),
                Schema {
                    ty: SchemaType::Properties {
                        properties: [
                            (
                                "lat",
                                Schema {
                                    ty: SchemaType::Type {
                                        r#type: TypeSchema::Float32,
                                    },
                                    ..Schema::default()
                                },
                            ),
                            (
                                "lng",
                                Schema {
                                    ty: SchemaType::Type {
                                        r#type: TypeSchema::Float32,
                                    },
                                    ..Schema::default()
                                },
                            ),
                        ]
                        .into(),
                        optional_properties: [].into(),
                        additional_properties: false,
                    },
                    ..Schema::default()
                },
            )]
            .into(),
        };

        assert_eq!(
            serde_json::to_value(&repr).unwrap(),
            serde_json::json!({
                "definitions": {
                    "coordinates": {
                        "properties": {
                            "lat": { "type": "float32" },
                            "lng": { "type": "float32" }
                        }
                    }
                },
                "properties": {
                    "userLoc": { "ref": "coordinates" },
                    "serverLoc": { "ref": "coordinates" }
                }
            })
        )
    }
}
