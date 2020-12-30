mod default;
mod deserializer;

pub use self::deserializer::from_env;

pub mod common;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(untagged)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Debug {
        All,
        LLB,
        Frontend,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "kebab-case")]
    struct CustomOptions {
        filename: Option<PathBuf>,
        verbosity: u32,

        #[serde(default)]
        debug: Vec<Debug>,

        #[serde(default)]
        cache_imports: Vec<common::CacheOptionsEntry>,
    }

    #[test]
    fn custom_options() {
        let env = vec![
            (
                "BUILDKIT_FRONTEND_OPT_0".into(),
                "filename=/path/to/Dockerfile".into(),
            ),
            (
                "BUILDKIT_FRONTEND_OPT_1".into(),
                "debug=llb,frontend".into(),
            ),
            (
                "BUILDKIT_FRONTEND_OPT_2".into(),
                r#"cache-imports=[{"Type":"local","Attrs":{"src":"cache"}}]"#.into(),
            ),
            (
                "BUILDKIT_FRONTEND_OPT_3".into(),
                "verbosity=12345678".into(),
            ),
        ];

        assert_eq!(
            from_env::<CustomOptions, _>(env.into_iter()).unwrap(),
            CustomOptions {
                filename: Some(PathBuf::from("/path/to/Dockerfile")),
                verbosity: 12_345_678,

                debug: vec![Debug::LLB, Debug::Frontend],

                cache_imports: vec![common::CacheOptionsEntry {
                    cache_type: common::CacheType::Local,
                    attrs: vec![("src".into(), "cache".into())].into_iter().collect()
                }],
            }
        );
    }

    #[test]
    fn env_variable_names() {
        let env = vec![
            (
                "ANOTHER_OPT_0".into(),
                "filename=/path/to/Dockerfile".into(),
            ),
            (
                "ANOTHER_OPT_2".into(),
                r#"cache-imports=[{"Type":"local","Attrs":{"src":"cache"}}]"#.into(),
            ),
            ("BUILDKIT_FRONTEND_OPT_1".into(), "debug=all".into()),
            (
                "BUILDKIT_FRONTEND_OPT_2".into(),
                "verbosity=12345678".into(),
            ),
        ];

        assert_eq!(
            from_env::<CustomOptions, _>(env.into_iter()).unwrap(),
            CustomOptions {
                filename: None,
                verbosity: 12_345_678,
                debug: vec![Debug::All],
                cache_imports: vec![],
            }
        );
    }

    #[test]
    fn empty_cache() {
        let env = vec![
            ("BUILDKIT_FRONTEND_OPT_1".into(), "cache-imports=".into()),
            (
                "BUILDKIT_FRONTEND_OPT_2".into(),
                "verbosity=12345678".into(),
            ),
        ];

        assert_eq!(
            from_env::<CustomOptions, _>(env.into_iter()).unwrap(),
            CustomOptions {
                filename: None,
                verbosity: 12_345_678,
                debug: vec![],
                cache_imports: vec![],
            }
        );
    }
}
