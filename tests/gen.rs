use jtd_derive::{GenError, Generator, JsonTypedef};

#[derive(JsonTypedef)]
#[allow(dead_code)]
enum Foo {
    Bar,
}

mod foo {
    #[derive(jtd_derive::JsonTypedef)]
    #[allow(dead_code)]
    pub enum Foo {
        Baz,
    }
}

#[derive(JsonTypedef)]
#[allow(dead_code)]
struct Wrapping {
    foo1: Foo,
    foo2: foo::Foo,
}

#[test]
fn name_collisions() {
    let GenError::NameCollision { type1, type2, id } = Generator::builder()
        .naming_short()
        .build()
        .into_root_schema::<Wrapping>()
        .unwrap_err();

    assert_eq!(id, "Foo");
    assert!([type1.as_str(), type2.as_str()].contains(&"gen::Foo"));
    assert!([type1.as_str(), type2.as_str()].contains(&"gen::foo::Foo"));
}
