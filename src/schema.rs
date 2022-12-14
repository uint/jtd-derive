//! Internal Rust representation of a JSON Typedef schema.

use std::collections::HashMap;

use serde::Serialize;

// All this corresponds fairly straightforwardly to https://jsontypedef.com/docs/jtd-in-5-minutes/
// I'd normally try to separate the serialization logic from the Rust representation, but using
// serde derives makes this so very easy. Damnit.

/// The top level of a Typedef schema.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct RootSchema {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub definitions: HashMap<&'static str, Schema>,
    #[serde(flatten)]
    pub schema: Schema,
}

/// A Typedef schema.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Schema {
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub ty: SchemaType,
    #[serde(skip_serializing_if = "is_false")]
    pub nullable: bool,
}

impl Schema {
    pub fn empty() -> Self {
        Self {
            metadata: Metadata::default(),
            ty: SchemaType::Empty,
            nullable: false,
        }
    }
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// The 8 "forms" a schema can take.
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
    Properties {
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        properties: HashMap<&'static str, Schema>,
        #[serde(
            skip_serializing_if = "HashMap::is_empty",
            rename = "optionalProperties"
        )]
        optional_properties: HashMap<&'static str, Schema>,
        #[serde(skip_serializing_if = "is_false", rename = "additionalProperties")]
        additional_properties: bool,
    },
    Values {
        values: Box<Schema>,
    },
    Discriminator {
        discriminator: &'static str,
        // Can only contain non-nullable "properties" schemas
        mapping: HashMap<&'static str, Schema>,
    },
    Ref {
        r#ref: &'static str,
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

/// Schema metadata.
#[derive(Default, Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Metadata(HashMap<&'static str, serde_json::Value>);

impl Metadata {
    pub fn from_map(m: impl Into<HashMap<&'static str, serde_json::Value>>) -> Self {
        Self(m.into())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                metadata: Metadata::default(),
                ty: SchemaType::Type {
                    r#type: TypeSchema::Int16,
                },
                nullable: true,
            },
            definitions: HashMap::new(),
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
            definitions: HashMap::new(),
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
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                        metadata: Metadata::default(),
                        ty: SchemaType::Enum {
                            r#enum: vec!["FOO", "BAR", "BAZ"],
                        },
                        nullable: true,
                    }),
                },
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                                ..Schema::empty()
                            },
                        ),
                        (
                            "isAdmin",
                            Schema {
                                ty: SchemaType::Type {
                                    r#type: TypeSchema::Boolean,
                                },
                                ..Schema::empty()
                            },
                        ),
                    ]
                    .into(),
                    optional_properties: [].into(),
                    additional_properties: false,
                },
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                                ..Schema::empty()
                            },
                        ),
                        (
                            "isAdmin",
                            Schema {
                                ty: SchemaType::Type {
                                    r#type: TypeSchema::Boolean,
                                },
                                ..Schema::empty()
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
                            ..Schema::empty()
                        },
                    )]
                    .into(),
                    additional_properties: true,
                },
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                        ..Schema::empty()
                    }),
                },
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                                            ..Schema::empty()
                                        },
                                    )]
                                    .into(),
                                    optional_properties: [].into(),
                                    additional_properties: false,
                                },
                                ..Schema::empty()
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
                                                ..Schema::empty()
                                            },
                                        ),
                                        (
                                            "plan",
                                            Schema {
                                                ty: SchemaType::Enum {
                                                    r#enum: vec!["FREE", "PAID"],
                                                },
                                                ..Schema::empty()
                                            },
                                        ),
                                    ]
                                    .into(),
                                    optional_properties: [].into(),
                                    additional_properties: false,
                                },
                                ..Schema::empty()
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
                                                ..Schema::empty()
                                            },
                                        ),
                                        (
                                            "softDelete",
                                            Schema {
                                                ty: SchemaType::Type {
                                                    r#type: TypeSchema::Boolean,
                                                },
                                                ..Schema::empty()
                                            },
                                        ),
                                    ]
                                    .into(),
                                    optional_properties: [].into(),
                                    additional_properties: false,
                                },
                                ..Schema::empty()
                            },
                        ),
                    ]
                    .into(),
                },
                ..Schema::empty()
            },
            definitions: HashMap::new(),
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
                                    r#ref: "coordinates",
                                },
                                ..Schema::empty()
                            },
                        ),
                        (
                            "serverLoc",
                            Schema {
                                ty: SchemaType::Ref {
                                    r#ref: "coordinates",
                                },
                                ..Schema::empty()
                            },
                        ),
                    ]
                    .into(),
                    optional_properties: [].into(),
                    additional_properties: false,
                },
                ..Schema::empty()
            },
            definitions: [(
                "coordinates",
                Schema {
                    ty: SchemaType::Properties {
                        properties: [
                            (
                                "lat",
                                Schema {
                                    ty: SchemaType::Type {
                                        r#type: TypeSchema::Float32,
                                    },
                                    ..Schema::empty()
                                },
                            ),
                            (
                                "lng",
                                Schema {
                                    ty: SchemaType::Type {
                                        r#type: TypeSchema::Float32,
                                    },
                                    ..Schema::empty()
                                },
                            ),
                        ]
                        .into(),
                        optional_properties: [].into(),
                        additional_properties: false,
                    },
                    ..Schema::empty()
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
