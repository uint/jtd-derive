#[derive(jtd_derive::JsonTypedef)]
enum Mixed {
    Foo,
    Bar { x: u32 },
}

fn main() {}
