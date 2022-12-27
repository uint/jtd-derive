//! Schema generator and its settings.

use std::collections::HashMap;

use crate::{
    schema::{RootSchema, Schema},
    JsonTypedef,
};

/// A configurable schema generator. An instance is meant to produce one
/// [`RootSchema`] and be consumed in the process.
///
/// For now, the generator is not configurable and the only way to
/// construct one is by calling [`Generator::default()`].
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Generator {}

impl Generator {
    /// Generate the root schema for the given type according to the settings.
    /// This consumes the generator.
    ///
    /// For now, schemas are always inlined.
    pub fn into_root_schema<T: JsonTypedef>(mut self) -> RootSchema {
        let schema = T::schema(&mut self);

        RootSchema {
            definitions: HashMap::new(),
            schema,
        }
    }

    /// Generate a [`Schema`] for a given type, adding definitions to the
    /// generator as appropriate.
    ///
    /// This is meant to only be used when implementing [`JsonTypedef`] for
    /// new types. Most commonly you'll derive that trait. It's unlikely you'll
    /// need to call this method explicitly.
    pub fn sub_schema<T: JsonTypedef>(&mut self) -> Schema {
        // TODO: if not inlining, produce a ref schema and store the definition
        // inside `Generator` for later

        T::schema(self)
    }
}
