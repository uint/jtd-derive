#[derive(jtd_derive::JsonTypedef)]
#[typedef(metadata(foo, bar, foo = 2))]
struct Foo {
    bar: u32,
}

#[derive(jtd_derive::JsonTypedef)]
#[typedef(metadata = "a")]
struct Bar {
    bar: u32,
}

#[derive(jtd_derive::JsonTypedef)]
#[typedef(metadata)]
struct Baz {
    bar: u32,
}

fn main() {}
