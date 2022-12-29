use jtd_derive::{gen::Generator, JsonTypedef};

#[derive(JsonTypedef)]
#[allow(unused)]
struct Foo {
    bar: Bar,
    recursive: Recursive,
}

#[derive(JsonTypedef)]
#[allow(unused)]
struct Bar {
    bar: u32,
}

#[derive(JsonTypedef)]
#[allow(unused)]
struct Recursive {
    inner: Option<Box<Recursive>>,
}

#[test]
fn prefer_inline() {
    assert_eq!(
        serde_json::to_value(
            Generator::builder()
                .prefer_inline()
                .build()
                .into_root_schema::<Foo>()
        )
        .unwrap(),
        serde_json::json! {{
            "definitions": {
                "inlining::Recursive": {
                    "properties": {
                        "inner": {
                            "ref": "inlining::Recursive",
                            "nullable": true,
                        }
                    },
                    "additionalProperties": true,
                },
            },
            "properties": {
                "bar": {
                    "properties": { "bar": { "type": "uint32" } },
                    "additionalProperties": true,
                },
                "recursive": { "ref": "inlining::Recursive" },
            },
            "additionalProperties": true,
        }}
    );
}

#[test]
fn prefer_ref() {
    assert_eq!(
        serde_json::to_value(
            Generator::builder()
                .top_level_ref()
                .build()
                .into_root_schema::<Foo>()
        )
        .unwrap(),
        serde_json::json! {{
            "definitions": {
                "inlining::Foo": {
                    "properties": {
                        "bar": {"ref": "inlining::Bar"},
                        "recursive": { "ref": "inlining::Recursive" },
                    },
                    "additionalProperties": true,
                },
                "inlining::Bar": {
                    "properties": { "bar": { "type": "uint32" } },
                    "additionalProperties": true,
                },
                "inlining::Recursive": {
                    "properties": {
                        "inner": {
                            "ref": "inlining::Recursive",
                            "nullable": true,
                        }
                    },
                    "additionalProperties": true,
                },
            },
            "ref": "inlining::Foo",
        }}
    );
}
