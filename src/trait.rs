use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fmt::Arguments;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{self, Wrapping};
use std::ops::{Range, RangeInclusive};
use std::path::{Path, PathBuf};
use std::sync::{atomic, Mutex, RwLock};

use crate::schema::{Schema, SchemaType, TypeSchema};

trait JsonTypedef {
    fn schema() -> Schema;
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
            }
        )*
	};
}

impl_primitives! {
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
    num::NonZeroU8 => Uint8,
    num::NonZeroU16 => Uint16,
    num::NonZeroU32 => Uint32,
    num::NonZeroI8 => Int8,
    num::NonZeroI16 => Int16,
    num::NonZeroI32 => Int32,
    char => String,
    String => String,
    str => String,
    CString => String,
    CStr => String,
    OsString => String,
    OsStr => String,
    PathBuf => String,
    Path => String,
    IpAddr => String,
    Ipv4Addr => String,
    Ipv6Addr => String,
    SocketAddr => String,
    SocketAddrV4 => String,
    SocketAddrV6 => String
}

impl<T: JsonTypedef> JsonTypedef for Option<T> {
    fn schema() -> Schema {
        // an argument could be made this should error on `Option<Option<T>>`, but
        // we're trying to follow serde's footsteps
        let mut schema = T::schema();
        schema.nullable = true;
        schema
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
            }
        )*
	};
}

impl_map_like!(
    BTreeMap<K, V>,
    HashMap<K, V>
);

macro_rules! impl_transparent {
	($($in:ty),*) => {
		$(
            impl<T: JsonTypedef> JsonTypedef for $in {
                fn schema() -> Schema {
                    T::schema()
                }
            }
        )*
	};
}

impl_transparent!(
    Wrapping<T>,
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
            impl<'a, T: JsonTypedef> JsonTypedef for $in {
                fn schema() -> Schema {
                    T::schema()
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
            }
        )*
	};
}

impl_range!(Range<T>, RangeInclusive<T>);
