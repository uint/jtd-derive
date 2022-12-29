use jtd_derive::{gen::Generator, JsonTypedef};

#[derive(JsonTypedef)]
#[allow(dead_code)]
struct Cstruct {
    bar: u32,
    baz: Option<String>,
}

#[test]
fn cstruct() {
    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<Cstruct>()).unwrap(),
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
struct CstructWithGenerics<'a, T, const N: usize> {
    bar: &'a str,
    baz: [T; N],
}

#[test]
fn cstruct_with_generics() {
    assert_eq!(
        serde_json::to_value(
            Generator::default().into_root_schema::<CstructWithGenerics::<'_, u32, 2>>()
        )
        .unwrap(),
        serde_json::json! {{
            "properties": {
                "bar": { "type": "string" },
                "baz": { "elements": { "type": "uint32" } }
            },
            "additionalProperties": true
        }}
    );
}

#[derive(JsonTypedef)]
#[allow(dead_code)]
struct Newtype(u32);

#[test]
fn newtype_like() {
    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<Newtype>()).unwrap(),
        serde_json::json! {{
            "type": "uint32",
        }}
    );
}

#[derive(JsonTypedef)]
#[allow(dead_code)]
struct Nested {
    inner: Newtype,
}

#[test]
fn nested() {
    assert_eq!(
        serde_json::to_value(Generator::default().into_root_schema::<Nested>()).unwrap(),
        serde_json::json! {{
            "definitions": {
                "r#struct::Newtype": {
                    "type": "uint32",
                },
            },
            "properties": {
                "inner": { "ref": "r#struct::Newtype" }
            },
            "additionalProperties": true,
        }}
    );
}
