use jtd_derive::{gen::Generator, JsonTypedef};
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
        serde_json::to_value(Generator::default().into_root_schema::<StructVariants>()).unwrap(),
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
        serde_json::to_value(Generator::default().into_root_schema::<DenyStruct>()).unwrap(),
        serde_json::json! {{
            "properties": {
                "x": { "type": "uint32" }
            }
        }}
    );

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<DenyEnum>()).unwrap(),
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
        serde_json::to_value(Generator::default().into_root_schema::<DenyStruct>()).unwrap(),
        serde_json::to_value(Generator::default().into_root_schema::<DenyStruct2>()).unwrap(),
    );

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<DenyEnum>()).unwrap(),
        serde_json::to_value(Generator::default().into_root_schema::<DenyEnum2>()).unwrap(),
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
        serde_json::to_value(Generator::default().into_root_schema::<Transparent>()).unwrap(),
        serde_json::json! {{ "type": "uint32" }}
    );
}
