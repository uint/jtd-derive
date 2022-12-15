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

#[derive(JsonTypedef)]
#[allow(dead_code)]
struct FooWithGenerics<'a, T, const N: usize> {
    bar: &'a str,
    baz: [T; N],
}

#[test]
fn foo2() {
    assert_eq!(
        serde_json::to_value(FooWithGenerics::<'_, u32, 2>::schema()).unwrap(),
        serde_json::json! {{
            "properties": {
                "bar": { "type": "string" },
                "baz": { "elements": { "type": "uint32" } }
            },
            "additionalProperties": true
        }}
    );
}
