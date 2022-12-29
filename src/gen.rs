//! Schema generator and its settings.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use crate::{
    schema::{Names, RootSchema, Schema, SchemaType},
    JsonTypedef,
};

/// A configurable schema generator. An instance is meant to produce one
/// [`RootSchema`] and be consumed in the process.
///
/// For now, the generator is not configurable and the only way to
/// construct one is by calling [`Generator::default()`].
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Generator {
    naming_strategy: NamingStrategy,
    refs: HashSet<Names>,
    definitions: HashMap<Names, DefinitionState>,
    inlining: bool,
}

#[derive(Debug, Clone)]
enum DefinitionState {
    Finished(Schema),
    Processing,
}

impl DefinitionState {
    fn unwrap(self) -> Schema {
        if let Self::Finished(schema) = self {
            schema
        } else {
            panic!()
        }
    }

    fn finalize(&mut self, schema: Schema) {
        match self {
            DefinitionState::Finished(_) => panic!("schema already finalized"),
            DefinitionState::Processing => *self = DefinitionState::Finished(schema),
        }
    }
}

impl Default for DefinitionState {
    fn default() -> Self {
        Self::Processing
    }
}

impl Generator {
    /// Generate the root schema for the given type according to the settings.
    /// This consumes the generator.
    pub fn into_root_schema<T: JsonTypedef>(mut self) -> RootSchema {
        let schema = self.sub_schema_impl::<T>(true);
        self.clean_up_defs();

        let definitions = self
            .definitions
            .into_iter()
            .map(|(n, s)| (self.naming_strategy.fun()(&n), s.unwrap()))
            .collect();

        RootSchema {
            definitions,
            schema,
        }
    }

    /// Generate a [`Schema`] for a given type, adding definitions to the
    /// generator as appropriate.
    ///
    /// This is meant to only be called when implementing [`JsonTypedef`] for
    /// new types. Most commonly you'll derive that trait. It's unlikely you'll
    /// need to call this method explicitly.
    pub fn sub_schema<T: JsonTypedef>(&mut self) -> Schema {
        self.sub_schema_impl::<T>(false)
    }

    fn sub_schema_impl<T: JsonTypedef>(&mut self, top_level: bool) -> Schema {
        let names = T::names();
        let inlining = top_level || self.inlining;

        let inlined_schema = match self.definitions.get(&names) {
            Some(DefinitionState::Finished(schema)) => {
                // we had already built a schema for this type.
                // no need to do it again.

                (!T::referenceable() || (inlining && !self.refs.contains(&names)))
                    .then_some(schema.clone())
            }
            Some(DefinitionState::Processing) => {
                // we're already in the process of building a schema for this type.
                // this means it's recursive and the only way to keep things sane
                // is to go by reference

                None
            }
            None => {
                // no schema available yet, so we have to build it
                if T::referenceable() {
                    self.definitions
                        .insert(names.clone(), DefinitionState::Processing);
                    let schema = T::schema(self);
                    self.definitions
                        .get_mut(&names)
                        .unwrap()
                        .finalize(schema.clone());

                    (inlining && !self.refs.contains(&names)).then_some(schema)
                } else {
                    Some(T::schema(self))
                }
            }
        };

        inlined_schema.unwrap_or_else(|| {
            let schema = Schema {
                ty: SchemaType::Ref {
                    r#ref: self.naming_strategy.fun()(&names),
                },
                ..Schema::default()
            };
            self.refs.insert(names);
            schema
        })
    }

    fn clean_up_defs(&mut self) {
        let to_remove: Vec<_> = self
            .definitions
            .keys()
            .filter(|names| !self.refs.contains(names))
            .cloned()
            .collect();

        for names in to_remove {
            self.definitions.remove(&names);
        }
    }
}

struct NamingStrategy(Box<dyn Fn(&Names) -> String>);

impl NamingStrategy {
    /// A naming strategy that produces the stringified full path
    /// of the type with type parameters and const parameters in angle brackets.
    ///
    /// E.g. if you have a struct like this in the top-level of `my_crate`:
    ///
    /// ```
    /// #[derive(jtd_derive::JsonTypedef)]
    /// struct Foo<T, const N: usize> {
    ///     x: [T; N],
    /// }
    /// ```
    ///
    /// Then the concrete type `Foo<u32, 5>` will be named `"my_crate::Foo<uint32, 5>"`
    /// in the schema.
    pub fn long() -> Self {
        fn strategy(names: &Names) -> String {
            let params = names
                .type_params
                .iter()
                .map(strategy)
                .chain(names.const_params.clone())
                .reduce(|l, r| format!("{}, {}", l, r));

            match params {
                Some(params) => format!("{}<{}>", names.long, params),
                None => names.long.to_string(),
            }
        }

        Self(Box::new(strategy))
    }

    pub fn fun(&self) -> &dyn Fn(&Names) -> String {
        &self.0
    }
}

impl Default for NamingStrategy {
    fn default() -> Self {
        Self::long()
    }
}

impl Debug for NamingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let example = Names {
            short: "Foo",
            long: "my_crate::Foo",
            type_params: vec![u32::names()],
            const_params: vec!["5".to_string()],
        };
        let result = self.fun()(&example);

        f.write_fmt(format_args!(
            "NamingStrategy(Foo<u32, 5> -> \"{}\")",
            result
        ))
    }
}
