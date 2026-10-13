macro_rules! catalog {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident describes $meta:ident {
            $( $field:ident: $fty:ty ),+ $(,)?
        }
        {
            $( $variant:ident { $( $vfield:ident: $value:expr ),+ $(,)? } ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $( $variant, )+
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis struct $meta {
            $( $field: $fty, )+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[ $( $name::$variant, )+ ];

            const fn describe(&self) -> $meta {
                match self {
                    $( Self::$variant => $meta { $( $vfield: $value, )+ }, )+
                }
            }

            $(
                pub const fn $field(&self) -> $fty {
                    self.describe().$field
                }
            )+
        }
    };
}

pub(crate) use catalog;

#[cfg(test)]
mod tests {
    use super::catalog;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EnvSourceKind {
        Local,
        Vault,
    }

    catalog! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum EnvOption describes EnvOptionMeta {
            key: &'static str,
            source: EnvSourceKind,
            secret: bool,
        }
        {
            BindAddr { key: "ADDR", source: EnvSourceKind::Local, secret: false },
            DatabaseUrl { key: "DATABASE_URL", source: EnvSourceKind::Local, secret: false },
            ApiKey { key: "API_KEY", source: EnvSourceKind::Vault, secret: true },
        }
    }

    #[test]
    fn all_lists_every_variant_in_declaration_order() {
        assert_eq!(
            EnvOption::ALL,
            &[EnvOption::BindAddr, EnvOption::DatabaseUrl, EnvOption::ApiKey]
        );
    }

    #[test]
    fn accessors_read_the_declared_metadata() {
        assert_eq!(EnvOption::BindAddr.key(), "ADDR");
        assert_eq!(EnvOption::ApiKey.source(), EnvSourceKind::Vault);
        assert!(EnvOption::ApiKey.secret());
        assert!(!EnvOption::DatabaseUrl.secret());
    }

    #[test]
    fn keys_are_unique() {
        let mut keys: Vec<_> = EnvOption::ALL.iter().map(EnvOption::key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), EnvOption::ALL.len());
    }
}
