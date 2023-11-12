use jtd_derive::{Generator, JsonTypedef};

#[test]
fn top_level() {
    #[derive(JsonTypedef)]
    #[typedef(metadata(x = "\"stuff\"", y = "{ \"inner\": 5 }"))]
    #[allow(unused)]
    struct Foo {
        bar: u32,
    }

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<Foo>().unwrap()).unwrap(),
        serde_json::json! {{
            "properties": {
                "bar": { "type": "uint32" },
            },
            "additionalProperties": true,
            "metadata": {
                "x": "stuff",
                "y": {
                    "inner": 5
                }
            }
        }}
    );
}

#[test]
fn struct_field() {
    #[derive(JsonTypedef)]
    #[allow(unused)]
    struct Foo {
        #[typedef(metadata(x = "\"stuff\"", y = "{ \"inner\": 5 }"))]
        bar: u32,
    }

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<Foo>().unwrap()).unwrap(),
        serde_json::json! {{
            "properties": {
                "bar": {
                    "type": "uint32",
                    "metadata": {
                        "x": "stuff",
                        "y": {
                            "inner": 5
                        }
                    }
                },
            },
            "additionalProperties": true
        }}
    );
}

#[test]
fn variant_field() {
    #[derive(JsonTypedef)]
    #[typedef(tag = "type")]
    #[allow(unused)]
    enum Foo {
        Bar {
            #[typedef(metadata(x = "\"stuff\"", y = "{ \"inner\": 5 }"))]
            baz: u32,
        },
    }

    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<Foo>().unwrap()).unwrap(),
        serde_json::json! {{
            "discriminator": "type",
            "mapping": {
                "Bar": {
                    "properties": {
                        "baz": {
                            "type": "uint32",
                            "metadata": {
                                "x": "stuff",
                                "y": {
                                    "inner": 5
                                }
                            }
                        }
                    },
                    "additionalProperties": true
                },
            }
        }}
    );
}
