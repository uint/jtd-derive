use crate::{schema::Schema, JsonTypedef};

#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Generator {}

impl Generator {
    pub fn sub_schema<T: JsonTypedef>(&mut self) -> Schema {
        // TODO: if not inlining, produce a ref schema and store the definition
        // inside `Generator` for later

        T::schema(self)
    }
}
