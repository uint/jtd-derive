/// How to refer to a given schema. Used mostly for referring to a schema definition
/// using the ["ref" form](https://jsontypedef.com/docs/jtd-in-5-minutes/#ref-schemas).
///
/// The [`Generator`](crate::gen::Generator) decides how to use this information to
/// generate an actual identifier.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Names {
    /// The short name. Most of the time this is just the ident of the Rust type.
    pub short: &'static str,
    /// The long name. Most of the time this is the full path of the Rust type, starting
    /// with the crate name.
    pub long: &'static str,
    /// Nullability.
    pub nullable: bool,
    /// Names of any type arguments applied to the generic Rust type.
    pub type_params: Vec<Names>,
    /// The values of constant arguments represented as strings.
    pub const_params: Vec<String>,
}
