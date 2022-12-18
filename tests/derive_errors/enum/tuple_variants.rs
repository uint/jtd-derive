#[derive(jtd_derive::JsonTypedef)]
enum Tuple {
    Foo(u32),
    Bar(),
}

#[derive(jtd_derive::JsonTypedef)]
enum MixedWithTuple {
    Foo { x: String },
    Bar(u32),
}

fn main() {}
