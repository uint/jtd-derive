use jtd_derive::{Generator, JsonTypedef};
use serde::Deserialize;

#[derive(JsonTypedef, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum StructVariants {
    Bar { x: u32 },
    Baz { y: String },
}

#[test]
fn enum_respects_serde_tag_attr() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<StructVariants>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "discriminator": "type",
            "mapping": {
                "Bar": {
                    "properties": {
                        "x": {"type": "uint32"}
                    },
                    "additionalProperties": true
                },
                "Baz": {
                    "properties": {
                        "y": {"type": "string"}
                    },
                    "additionalProperties": true
                }
            }
        }}
    );
}

#[derive(JsonTypedef, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DenyStruct {
    x: u32,
}

#[derive(JsonTypedef)]
#[typedef(deny_unknown_fields)]
#[allow(dead_code)]
struct DenyStruct2 {
    x: u32,
}

#[derive(JsonTypedef, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
#[allow(dead_code)]
enum DenyEnum {
    Bar { x: u32 },
}

#[derive(JsonTypedef)]
#[typedef(tag = "type", deny_unknown_fields)]
#[allow(dead_code)]
enum DenyEnum2 {
    Bar { x: u32 },
}

#[test]
fn deny_unknown_fields() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<DenyStruct>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "properties": {
                "x": { "type": "uint32" }
            }
        }}
    );

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<DenyEnum>().unwrap()).unwrap(),
        serde_json::json! {{
            "discriminator": "type",
            "mapping": {
                "Bar": {
                    "properties": {
                        "x": { "type": "uint32" }
                    }
                }
            }
        }}
    );

    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<DenyStruct>()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<DenyStruct2>()
                .unwrap()
        )
        .unwrap(),
    );

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<DenyEnum>().unwrap()).unwrap(),
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<DenyEnum2>()
                .unwrap()
        )
        .unwrap(),
    );
}

#[derive(JsonTypedef, Deserialize)]
#[serde(transparent)]
#[allow(dead_code)]
struct Transparent {
    x: u32,
}

#[test]
fn transparent() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<Transparent>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{ "type": "uint32" }}
    );
}

#[derive(JsonTypedef, Deserialize)]
#[serde(from = "Transparent")]
#[allow(dead_code)]
struct FromStruct {
    x: Transparent,
}

impl From<Transparent> for FromStruct {
    fn from(x: Transparent) -> Self {
        Self { x }
    }
}

#[test]
fn from() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<Transparent>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{ "type": "uint32" }}
    );
}

#[derive(Default, JsonTypedef, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
struct FromDefault {
    x: bool,
}

#[test]
fn default() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<FromDefault>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "optionalProperties": {
                "x": { "type": "boolean" }
            },
            "additionalProperties": true,
        }}
    );
}

#[derive(JsonTypedef, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct RenameStruct {
    foo_bar: bool,
}

#[derive(JsonTypedef, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
#[allow(dead_code)]
struct RenameStruct2 {
    foo_bar: bool,
}

#[test]
fn rename_all() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameStruct>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "properties": {
                "fooBar": { "type": "boolean" }
            },
            "additionalProperties": true,
        }}
    );
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameStruct>()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameStruct2>()
                .unwrap()
        )
        .unwrap(),
    );
}

#[derive(JsonTypedef, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
#[allow(dead_code)]
struct RenameStructSerialize {
    foo_bar: bool,
}

#[test]
fn rename_all_serialize_gets_ignored() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameStructSerialize>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "properties": {
                "foo_bar": { "type": "boolean" }
            },
            "additionalProperties": true,
        }}
    );
}

#[derive(JsonTypedef)]
#[typedef(rename_all = "SCREAMING-KEBAB-CASE")]
#[allow(dead_code)]
struct RenameStruct3 {
    foo_bar: bool,
}

#[test]
fn rename_all_typedef_attr() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameStruct3>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "properties": {
                "FOO-BAR": { "type": "boolean" }
            },
            "additionalProperties": true,
        }}
    );
}

#[derive(JsonTypedef)]
#[typedef(rename_all = "SCREAMING-KEBAB-CASE")]
#[allow(dead_code)]
enum RenameEnum {
    FooBar,
}

#[test]
fn rename_all_enum() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameEnum>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "enum": ["FOO-BAR"],
        }}
    );
}

#[derive(JsonTypedef)]
#[typedef(rename_all = "SCREAMING-KEBAB-CASE", tag = "type")]
#[allow(dead_code)]
enum RenameEnum2 {
    FooBar { x: u32 },
}

#[test]
fn rename_all_enum_struct_variants() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<RenameEnum2>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "discriminator": "type",
            "mapping": {
                "FOO-BAR": {
                    "properties": {
                        "x": { "type": "uint32" }
                    },
                    "additionalProperties": true,
                }
            }
        }}
    );
}
