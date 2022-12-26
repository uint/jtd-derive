use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt::Arguments;
use std::ops::{Range, RangeInclusive};
use std::sync::{atomic, Mutex, RwLock};

use crate::schema::{Names, Schema, SchemaType, TypeSchema};

pub use jtd_derive_macros::JsonTypedef;

pub trait JsonTypedef {
    fn schema() -> Schema;

    fn referenceable() -> bool {
        true
    }

    fn names() -> Names;
}

macro_rules! impl_primitives {
	($($in:ty => $out:ident),*) => {
		$(
            impl JsonTypedef for $in {
                fn schema() -> Schema {
                    Schema {
                        ty: SchemaType::Type {
                            r#type: TypeSchema::$out,
                        },
                        ..Schema::empty()
                    }
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    Names {
                        short: TypeSchema::$out.name(),
                        long: TypeSchema::$out.name(),
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
                fn schema() -> Schema {
                    Schema {
                        ty: SchemaType::Type {
                            r#type: TypeSchema::$out,
                        },
                        ..Schema::empty()
                    }
                }

                fn referenceable() -> bool {
                    true
                }

                fn names() -> Names {
                    Names {
                        short: stringify!($in),
                        long: stringify!($($path_parts)::+::$in),
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

impl JsonTypedef for std::path::PathBuf {
    fn schema() -> Schema {
        std::path::Path::schema()
    }

    fn referenceable() -> bool {
        std::path::Path::referenceable()
    }

    fn names() -> Names {
        std::path::Path::names()
    }
}

impl<T: JsonTypedef> JsonTypedef for Option<T> {
    fn schema() -> Schema {
        let mut schema = T::schema();
        schema.nullable = true;
        schema
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        T::names()
    }
}

macro_rules! impl_array_like {
	($($in:ty),*) => {
		$(
            impl<T: JsonTypedef> JsonTypedef for $in {
                fn schema() -> Schema {
                    Schema {
                        ty: SchemaType::Elements {
                            elements: Box::new(T::schema()),
                        },
                        ..Schema::empty()
                    }
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    Names {
                        short: "array",
                        long: "array",
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
    fn schema() -> Schema {
        Schema {
            ty: SchemaType::Elements {
                elements: Box::new(T::schema()),
            },
            ..Schema::empty()
        }
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        Names {
            short: "array",
            long: "array",
            type_params: vec![T::names()],
            const_params: vec![],
        }
    }
}

macro_rules! impl_map_like {
	($($in:ty),*) => {
		$(
            impl<K: ToString, V: JsonTypedef> JsonTypedef for $in {
                fn schema() -> Schema {
                    Schema {
                        ty: SchemaType::Values {
                            values: Box::new(V::schema()),
                        },
                        ..Schema::empty()
                    }
                }

                fn referenceable() -> bool {
                    false
                }

                fn names() -> Names {
                    Names {
                        short: "map",
                        long: "map",
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
                fn schema() -> Schema {
                    T::schema()
                }

                fn referenceable() -> bool {
                    T::referenceable()
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
                fn schema() -> Schema {
                    T::schema()
                }

                fn referenceable() -> bool {
                    T::referenceable()
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
    fn schema() -> Schema {
        T::schema()
    }

    fn referenceable() -> bool {
        T::referenceable()
    }

    fn names() -> Names {
        T::names()
    }
}

impl<'a> JsonTypedef for Arguments<'a> {
    fn schema() -> Schema {
        Schema {
            ty: SchemaType::Type {
                r#type: TypeSchema::String,
            },
            ..Schema::empty()
        }
    }

    fn referenceable() -> bool {
        false
    }

    fn names() -> Names {
        Names {
            short: "string",
            long: "string",
            type_params: vec![],
            const_params: vec![],
        }
    }
}

macro_rules! impl_range {
	($($in:ty),*) => {
		$(
            impl<T: JsonTypedef> JsonTypedef for $in {
                fn schema() -> Schema {
                    Schema {
                        ty: SchemaType::Properties {
                            properties: [("start", T::schema()), ("end", T::schema())].into(),
                            optional_properties: [].into(),
                            additional_properties: false,
                        },
                        ..Schema::empty()
                    }
                }

                fn referenceable() -> bool {
                    true
                }

                fn names() -> Names {
                    Names {
                        short: stringify!($in),
                        long: concat!("std::ops::", stringify!($in)),
                        type_params: vec![T::names()],
                        const_params: vec![],
                    }
                }
            }
        )*
	};
}

impl_range!(Range<T>, RangeInclusive<T>);
