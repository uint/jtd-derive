//! Schema generator and its settings.

mod naming_strategy;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;

use self::naming_strategy::NamingStrategy;
use crate::schema::{RootSchema, Schema, SchemaType};
use crate::type_id::{type_id, TypeId};
use crate::{JsonTypedef, Names};

/// A configurable schema generator. An instance is meant to produce one
/// [`RootSchema`] and be consumed in the process.
///
/// If you want to just use the sane defaults, try [`Generator::default()`].
///
/// Otherwise, you can configure schema generation using the builder.
///
/// # Examples
///
/// Using the default settings:
///
/// ```
/// use jtd_derive::{JsonTypedef, Generator};
///
/// #[derive(JsonTypedef)]
/// struct Foo {
///     x: u32,
/// }
///
/// let root_schema = Generator::default().into_root_schema::<Foo>().unwrap();
/// let json_schema = serde_json::to_value(&root_schema).unwrap();
///
/// assert_eq!(json_schema, serde_json::json!{ {
///     "properties": {
///         "x": { "type": "uint32" }
///     },
///     "additionalProperties": true,
/// } });
/// ```
///
/// Using custom settings:
///
/// ```
/// use jtd_derive::{JsonTypedef, Generator};
///
/// #[derive(JsonTypedef)]
/// struct Foo {
///     x: u32,
/// }
///
/// let root_schema = Generator::builder()
///     .top_level_ref()
///     .naming_short()
///     .build()
///     .into_root_schema::<Foo>()
///     .unwrap();
/// let json_schema = serde_json::to_value(&root_schema).unwrap();
///
/// assert_eq!(json_schema, serde_json::json!{ {
///     "definitions": {
///         "Foo": {
///             "properties": {
///                 "x": { "type": "uint32" }
///             },
///             "additionalProperties": true,
///         }
///     },
///     "ref": "Foo",
/// } });
/// ```
#[derive(Default, Debug)]
pub struct Generator {
    naming_strategy: NamingStrategy,
    /// Types for which at least one ref was created during schema gen.
    /// By keeping track of these, we can clean up unused definitions at the end.
    refs: HashSet<TypeId>,
    definitions: HashMap<TypeId, (Names, DefinitionState)>,
    inlining: Inlining,
}

impl Generator {
    /// Provide a `Generator` builder, allowing for some customization.
    pub fn builder() -> GeneratorBuilder {
        GeneratorBuilder::default()
    }

    /// Generate the root schema for the given type according to the settings.
    /// This consumes the generator.
    ///
    /// This will return an error if a naming collision is detected, i.e. two
    /// distinct Rust types produce the same identifier.
    pub fn into_root_schema<T: JsonTypedef>(mut self) -> Result<RootSchema, GenError> {
        let schema = self.sub_schema_impl::<T>(true);
        self.clean_up_defs();

        fn process_defs(
            defs: HashMap<TypeId, (Names, DefinitionState)>,
            ns: &mut NamingStrategy,
        ) -> Result<BTreeMap<String, Schema>, GenError> {
            // This could probably be optimized somehow.

            let defs = defs
                .into_iter()
                .map(|(_, (n, s))| (ns.fun()(&n), (n, s.unwrap())));

            let mut map = HashMap::new();

            for (key, (names, schema)) in defs {
                if let Some((other_names, _)) = map.get(&key) {
                    return Err(GenError::NameCollision {
                        id: key,
                        type1: NamingStrategy::long().fun()(other_names),
                        type2: NamingStrategy::long().fun()(&names),
                    });
                } else {
                    map.insert(key, (names, schema));
                }
            }

            Ok(map
                .into_iter()
                .map(|(key, (_, schema))| (key, schema))
                .collect())
        }

        Ok(RootSchema {
            definitions: process_defs(self.definitions, &mut self.naming_strategy)?,
            schema,
        })
    }

    /// Generate a [`Schema`] for a given type, adding definitions to the
    /// generator as appropriate.
    ///
    /// This is meant to only be called when implementing [`JsonTypedef`] for
    /// new types. Most commonly you'll derive that trait. It's unlikely you'll
    /// need to call this method explicitly.
    pub fn sub_schema<T: JsonTypedef + ?Sized>(&mut self) -> Schema {
        self.sub_schema_impl::<T>(false)
    }

    fn sub_schema_impl<T: JsonTypedef + ?Sized>(&mut self, top_level: bool) -> Schema {
        let id = type_id::<T>();
        let inlining = match self.inlining {
            Inlining::Always => true,
            Inlining::Normal => top_level,
            Inlining::Never => false,
        };

        let inlined_schema = match self.definitions.get(&id) {
            Some((_, DefinitionState::Finished(schema))) => {
                // we had already built a schema for this type.
                // no need to do it again.

                (!T::referenceable() || (inlining && !self.refs.contains(&id)))
                    .then_some(schema.clone())
            }
            Some((_, DefinitionState::Processing)) => {
                // we're already in the process of building a schema for this type.
                // this means it's recursive and the only way to keep things sane
                // is to go by reference

                None
            }
            None => {
                // no schema available yet, so we have to build it
                if T::referenceable() {
                    self.definitions
                        .insert(id, (T::names(), DefinitionState::Processing));
                    let schema = T::schema(self);
                    self.definitions
                        .get_mut(&id)
                        .unwrap()
                        .1
                        .finalize(schema.clone());

                    (inlining && !self.refs.contains(&id)).then_some(schema)
                } else {
                    Some(T::schema(self))
                }
            }
        };

        inlined_schema.unwrap_or_else(|| {
            let schema = Schema {
                ty: SchemaType::Ref {
                    r#ref: self.naming_strategy.fun()(&T::names()),
                },
                ..Schema::default()
            };
            self.refs.insert(id);
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

#[derive(Debug, Clone, Copy, Default)]
enum Inlining {
    Always,
    #[default]
    Normal,
    Never,
}

/// Builder for [`Generator`]. For example usage, refer to [`Generator`].
#[derive(Default, Debug)]
pub struct GeneratorBuilder {
    inlining: Inlining,
    naming_strategy: Option<NamingStrategy>,
}

impl GeneratorBuilder {
    /// Always try to inline complex types rather than provide them using
    /// definitions/refs. The exception is recursive types - these cannot
    /// be expressed without a ref.
    pub fn prefer_inline(&mut self) -> &mut Self {
        self.inlining = Inlining::Always;
        self
    }

    /// Where possible, provide types by ref even for the top-level type.
    pub fn top_level_ref(&mut self) -> &mut Self {
        self.inlining = Inlining::Never;
        self
    }

    /// A naming strategy that produces the stringified name
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
    /// Then the concrete type `Foo<u32, 5>` will be named `"Foo<uint32, 5>"`
    /// in the schema.
    ///
    /// Please note that this representation is prone to name collisions if you
    /// use identically named types in different modules or crates.
    pub fn naming_short(&mut self) -> &mut Self {
        self.naming_strategy = Some(NamingStrategy::short());
        self
    }

    /// Use the `long` naming strategy. This is the default.
    ///
    /// The `long` naming strategy produces the stringified full path
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
    ///
    /// This representation will prevent name collisions under normal circumstances,
    /// but it's technically possible some type will manually implement `names()`
    /// in a weird way.
    pub fn naming_long(&mut self) -> &mut Self {
        self.naming_strategy = Some(NamingStrategy::long());
        self
    }

    /// Use a custom naming strategy.
    pub fn naming_custom(&mut self, f: impl Fn(&Names) -> String + 'static) -> &mut Self {
        self.naming_strategy = Some(NamingStrategy::custom(f));
        self
    }

    /// Finalize the configuration and get a `Generator`.
    pub fn build(&mut self) -> Generator {
        Generator {
            inlining: self.inlining,
            naming_strategy: self.naming_strategy.take().unwrap_or_default(),
            ..Generator::default()
        }
    }
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

/// Schema generation errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub enum GenError {
    /// A name collision was detected, i.e. two distinct types have the same
    /// definition/ref identifiers.
    #[error("definition/ref id \"{id}\" is shared by types `{type1}` and `{type2}`")]
    NameCollision {
        type1: String,
        type2: String,
        id: String,
    },
}
