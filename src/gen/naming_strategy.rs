use super::Names;
use crate::JsonTypedef as _;

/// The naming strategy. The strategy decides how types are named in definitions/refs
/// in the _Typedef_ schema.
pub struct NamingStrategy(Box<dyn Fn(&Names) -> String>);

impl NamingStrategy {
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

    pub fn short() -> Self {
        fn strategy(names: &Names) -> String {
            let params = names
                .type_params
                .iter()
                .map(strategy)
                .chain(names.const_params.clone())
                .reduce(|l, r| format!("{}, {}", l, r));

            match params {
                Some(params) => format!("{}<{}>", names.short, params),
                None => names.short.to_string(),
            }
        }

        Self(Box::new(strategy))
    }

    pub fn custom<F: Fn(&Names) -> String + 'static>(fun: F) -> Self {
        Self(Box::new(fun))
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

impl std::fmt::Debug for NamingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let example = Names {
            short: "Foo",
            long: "my_crate::Foo",
            nullable: false,
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
