use jtd_derive::JsonTypedef;

#[derive(JsonTypedef)]
#[allow(dead_code)]
enum UnitVariants {
    Bar,
    Baz,
}

#[test]
fn enum_unit_variants() {
    assert_eq!(
        serde_json::to_value(UnitVariants::schema()).unwrap(),
        serde_json::json! {{
            "enum": ["Bar", "Baz"]
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
        serde_json::to_value(StructVariants::schema()).unwrap(),
        serde_json::json! {{
            "discriminator": "type",
            "mapping": {
                "Bar": {
                    "properties": {
                        "x": {"type": "uint32"}
                    }
                },
                "Baz": {
                    "properties": {
                        "y": {"type": "string"}
                    }
                }
            }
        }}
    );
}
