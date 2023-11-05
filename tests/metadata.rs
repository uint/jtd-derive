use jtd_derive::{Generator, JsonTypedef};

#[derive(JsonTypedef)]
#[typedef(metadata(x = "\"stuff\"", y = "{ \"inner\": 5 }"))]
#[allow(unused)]
struct Foo {
    bar: u32,
}

#[test]
fn top_level() {
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
