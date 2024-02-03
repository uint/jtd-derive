use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt::Arguments;
use std::ops::{Range, RangeInclusive};
use std::sync::{atomic, Mutex, RwLock};

use crate::schema::{Schema, SchemaType, TypeSchema};
use crate::{Generator, Names};

pub use jtd_derive_macros::JsonTypedef;

/// Types that have an associated [_Typedef_](https://jsontypedef.com/) schema.
pub trait JsonTypedef {
    /// Generate the [`Schema`] for the implementor type, according to how
    /// the [`Generator`] is configured.
    fn schema(generator: &mut Generator) -> Schema;

    /// Returns `true` if this type can appear in the top-level definitions
    /// and be referenced using the ["ref" form](https://jsontypedef.com/docs/jtd-in-5-minutes/#ref-schemas).
    fn referenceable() -> bool {
        true
    }

    /// Returns info about how to refer to this type within the
    /// [_Typedef_](https://jsontypedef.com/) schema.
    /// Mostly used to generate a name for the top-level definitions.
    fn names() -> Names;
}

macro_rules! impl_primitives {
	($($in:ty => $out:ident),*) => {
		$(
            impl JsonTypedef for $in {
                fn schema(_: &mut Generator) -> Schema {
                    Schema {
                        ty: SchemaType::Type {
                            r#type: TypeSchema::$out,
                        },
                        ..Schema::default()
                    }
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    Names {
                        short: TypeSchema::$out.name(),
                        long: TypeSchema::$out.name(),
                        nullable: false,
                        type_params: vec![],
                        const_params: vec![],
                    }
                }
            }
        )*
	};
}

impl_primitives! {
    // actual primitives
    bool => Boolean,
    u8 => Uint8,
    u16 => Uint16,
    u32 => Uint32,
    i8 => Int8,
    i16 => Int16,
    i32 => Int32,
    f32 => Float32,
    f64 => Float64,
    atomic::AtomicBool => Boolean,
    atomic::AtomicU8 => Uint8,
    atomic::AtomicU16 => Uint16,
    atomic::AtomicU32 => Uint32,
    atomic::AtomicI8 => Int8,
    atomic::AtomicI16 => Int16,
    atomic::AtomicI32 => Int32,
    char => String,
    String => String,
    str => String
}

// Distinct types due to additional constraints
macro_rules! impl_wrappers {
	($($($path_parts:ident)::+ => $in:ident => $out:ident),*) => {
		$(
            impl JsonTypedef for $($path_parts)::+::$in {
                fn schema(_: &mut Generator) -> Schema {
                    Schema {
                        ty: SchemaType::Type {
                            r#type: TypeSchema::$out,
                        },
                        ..Schema::default()
                    }
                }

                fn referenceable() -> bool {
                    true
                }

                fn names() -> Names {
                    Names {
                        short: stringify!($in),
                        long: stringify!($($path_parts)::+::$in),
                        nullable: false,
                        type_params: vec![],
                        const_params: vec![],
                    }
                }
            }
        )*
	};
}

impl_wrappers! {
    std::num => NonZeroU8 => Uint8,
    std::num => NonZeroU16 => Uint16,
    std::num => NonZeroU32 => Uint32,
    std::num => NonZeroI8 => Int8,
    std::num => NonZeroI16 => Int16,
    std::num => NonZeroI32 => Int32,

    std::net => IpAddr => String,
    std::net => Ipv4Addr => String,
    std::net => Ipv6Addr => String,
    std::net => SocketAddr => String,
    std::net => SocketAddrV4 => String,
    std::net => SocketAddrV6 => String,

    std::path => Path => String
}

#[cfg(feature = "url")]
impl_wrappers! {
    url => Url => String
}

impl JsonTypedef for std::path::PathBuf {
    fn schema(gen: &mut Generator) -> Schema {
        gen.sub_schema::<std::path::Path>()
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        std::path::Path::names()
    }
}

impl<T: JsonTypedef> JsonTypedef for Option<T> {
    fn schema(gen: &mut Generator) -> Schema {
        let mut schema = gen.sub_schema::<T>();
        schema.nullable = true;
        schema
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        let mut names = T::names();
        names.nullable = true;
        names
    }
}

macro_rules! impl_array_like {
	($($in:ty),*) => {
		$(
            impl<T: JsonTypedef> JsonTypedef for $in {
                fn schema(gen: &mut Generator) -> Schema {
                    Schema {
                        ty: SchemaType::Elements {
                            elements: Box::new(gen.sub_schema::<T>()),
                        },
                        ..Schema::default()
                    }
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    Names {
                        short: "array",
                        long: "array",
                        nullable: false,
                        type_params: vec![T::names()],
                        const_params: vec![],
                    }
                }
            }
        )*
	};
}

impl_array_like!(
    Vec<T>,
    VecDeque<T>,
    std::collections::BTreeSet<T>,
    BinaryHeap<T>,
    HashSet<T>,
    LinkedList<T>,
    [T]
);

impl<T: JsonTypedef, const N: usize> JsonTypedef for [T; N] {
    fn schema(gen: &mut Generator) -> Schema {
        Schema {
            ty: SchemaType::Elements {
                elements: Box::new(gen.sub_schema::<T>()),
            },
            ..Schema::default()
        }
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        Names {
            short: "array",
            long: "array",
            nullable: false,
            type_params: vec![T::names()],
            const_params: vec![],
        }
    }
}

macro_rules! impl_map_like {
	($($in:ty),*) => {
		$(
            impl<K: ToString, V: JsonTypedef> JsonTypedef for $in {
                fn schema(gen: &mut Generator) -> Schema {
                    Schema {
                        ty: SchemaType::Values {
                            values: Box::new(gen.sub_schema::<V>()),
                        },
                        ..Schema::default()
                    }
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    Names {
                        short: "map",
                        long: "map",
                        nullable: false,
                        type_params: vec![V::names()],
                        const_params: vec![],
                    }
                }
            }
        )*
	};
}

impl_map_like!(BTreeMap<K, V>, HashMap<K, V>);

macro_rules! impl_transparent {
	($($in:ty),*) => {
		$(
            impl<T: JsonTypedef> JsonTypedef for $in {
                fn schema(gen: &mut Generator) -> Schema {
                    gen.sub_schema::<T>()
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    T::names()
                }
            }
        )*
	};
}

impl_transparent!(
    std::num::Wrapping<T>,
    Cell<T>,
    RefCell<T>,
    Box<T>,
    Mutex<T>,
    RwLock<T>,
    Reverse<T>
);

macro_rules! impl_transparent_lifetime {
	($($in:ty),*) => {
		$(
            impl<'a, T: JsonTypedef + ?Sized> JsonTypedef for $in {
                fn schema(gen: &mut Generator) -> Schema {
                    gen.sub_schema::<T>()
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    T::names()
                }
            }
        )*
	};
}

impl_transparent_lifetime!(&'a T, &'a mut T);

impl<'a, T: JsonTypedef + Clone> JsonTypedef for Cow<'a, T> {
    fn schema(gen: &mut Generator) -> Schema {
        gen.sub_schema::<T>()
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        T::names()
    }
}

impl<'a> JsonTypedef for Arguments<'a> {
    fn schema(_: &mut Generator) -> Schema {
        Schema {
            ty: SchemaType::Type {
                r#type: TypeSchema::String,
            },
            ..Schema::default()
        }
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        Names {
            short: "string",
            long: "string",
            nullable: false,
            type_params: vec![],
            const_params: vec![],
        }
    }
}

macro_rules! impl_range {
	($($in:ty),*) => {
		$(
            impl<T: JsonTypedef> JsonTypedef for $in {
                fn schema(gen: &mut Generator) -> Schema {
                    Schema {
                        ty: SchemaType::Properties {
                            properties: [("start", gen.sub_schema::<T>()), ("end", gen.sub_schema::<T>())].into(),
                            optional_properties: [].into(),
                            additional_properties: false,
                        },
                        ..Schema::default()
                    }
                }

                fn referenceable() -> bool {
                    true
                }

                fn names() -> Names {
                    Names {
                        short: stringify!($in),
                        long: concat!("std::ops::", stringify!($in)),
                        nullable: false,
                        type_params: vec![T::names()],
                        const_params: vec![],
                    }
                }
            }
        )*
	};
}

impl_range!(Range<T>, RangeInclusive<T>);
