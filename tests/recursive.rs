use jtd_derive::{Generator, JsonTypedef};

#[derive(JsonTypedef)]
#[allow(unused)]
struct Recursive {
    inner: Option<Box<Recursive>>,
}

#[test]
fn recursive() {
    assert_eq!(
        serde_json::to_value(
            Generator::default()
                .into_root_schema::<Recursive>()
                .unwrap()
        )
        .unwrap(),
        serde_json::json! {{
            "definitions": {
                "recursive::Recursive": {
                    "properties": {
                        "inner": {
                            "ref": "recursive::Recursive",
                            "nullable": true,
                        }
                    },
                    "additionalProperties": true,
                },
            },
            "ref": "recursive::Recursive",
        }}
    );
}
