use jtd_derive::{Generator, JsonTypedef};

#[derive(JsonTypedef)]
#[allow(dead_code)]
enum UnitVariants {
    Bar,
    Baz,
}

#[test]
fn enum_unit_variants() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<UnitVariants>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "enum": ["Bar", "Baz"]
        }}
    );
}

#[derive(JsonTypedef)]
#[typedef(tag = "kind")]
#[allow(dead_code)]
enum UnitVariantsTagged {
    Bar,
    Baz,
}

#[test]
fn enum_unit_variants_tagged() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<UnitVariantsTagged>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "properties": {
                "kind": {"enum": ["Bar", "Baz"]}
            },
            "additionalProperties": true,
        }}
    );
}

#[derive(JsonTypedef)]
#[typedef(tag = "type")]
#[allow(dead_code)]
enum StructVariants {
    Bar { x: u32 },
    Baz { y: String },
}

#[test]
fn enum_struct_variants() {
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
