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
        serde_json::to_value(Cstruct::schema(&mut Generator::default())).unwrap(),
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
        serde_json::to_value(CstructWithGenerics::<'_, u32, 2>::schema(
            &mut Generator::default()
        ))
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
        serde_json::to_value(Newtype::schema(&mut Generator::default())).unwrap(),
        serde_json::json! {{
            "type": "uint32",
        }}
    );
}
