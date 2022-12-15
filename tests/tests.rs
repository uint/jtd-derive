use jtd_derive::JsonTypedef;

#[derive(JsonTypedef)]
#[allow(dead_code)]
struct Foo {
    bar: u32,
    baz: Option<String>,
}

#[test]
fn foo() {
    assert_eq!(
        serde_json::to_value(Foo::schema()).unwrap(),
        serde_json::json! {{
            "properties": {
                "bar": { "type": "uint32" },
                "baz": { "type": "string", "nullable": true }
            },
            "additionalProperties": true
        }}
    );
}
