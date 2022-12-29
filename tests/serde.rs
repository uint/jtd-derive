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
