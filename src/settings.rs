//! Per datasource configuration loader/handler aiming to enable re-usability as much as possible

quick_error! {
    /// SettingsError contains all the possible Error types returned by the Settings impl
    #[derive(Debug)]
    pub enum SettingsError {
        /// Crate config specific errors
        ConfigError(err: config::ConfigError) {
            display("{}", err)
                from()
        }
        /// Configuration file(s) related I/O errors
        IOError(err: std::io::Error) {
            display("{}", err)
                from()
        }
    }
}

/// Settings struct is empty currently
#[derive(Debug)]
pub struct Settings;

/// We use Arc<> to pass data source specific immutable configuration that has been cloned
use std::sync::Arc;

/// Settings implements the interface to use outside the module
impl Settings {
    /// Load dataset specific settings typically identified by two-letter country code e.g. au
    /// This is expected to conform [ISO 3166-1_alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)
    /// Except lowercase due to [Rust RFC 430](https://rust-lang.github.io/api-guidelines/naming.html)
    /// ```rust
    /// # fn main() {
    ///     let au_config = gis_puller::settings::Settings::load("au");
    /// #
    /// #   println!("AU({:p}) = {:#?}", &au_config, au_config);
    /// # }
    /// ```
    pub fn load(submod: &str) -> Result<Arc<SettingsDerive>, SettingsError> {
        validate_submod("upsert", submod);
        let settings_config = SettingsDerive::merge_all(submod)?;
        SettingsContainer::upsert(submod, settings_config)?;
        SettingsContainer::handle(submod)
    }
}

use config::{Config, Environment, File};
use serde::Deserialize;

#[allow(unused_imports)]
use validator::{Validate, ValidationError};

#[derive(Debug, Default, Validate, Deserialize, Clone, PartialEq)]
/// Used by au submodule currently, to be generalised
pub struct MatcherSettingsDerive {
    #[validate(non_control_character)]
    catalog: String,
    /// URL for the REST API
    #[validate(url)]
    pub rest_url: Option<String>,
    /// query value parameter for the REST call
    #[validate(non_control_character)]
    pub query: Option<String>,
    /// regex string for matching valid fetcher URLs
    #[validate(non_control_character)]
    pub matcher: Option<String>,
}

use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
/// Settings files JSON deserialises over this
pub struct SettingsDerive {
    /// fetcher_matcher settings used in au submod
    pub fetcher_matcher: MatcherSettingsDerive,
}

use std::env;
//use camino::Utf8PathBuf;
// @FEATURE: No enforcing of UTF-8/String due to portable impl so we'll go by OsString with no camino::Utf8PathBuf that would give us lossless Display/Debug
use std::path::PathBuf;

impl SettingsDerive {
    /// Merge all configurations from settings and environment variables on a submod e.g. au
    /// Return SettingsDerive container for the merged settings by the config crate
    ///
    fn merge_all(submod: &str) -> Result<Self, SettingsError> {
        validate_submod("upsert", submod);
        let mut s = Config::default();

        // @TODO: Incorporate some XGD crate in future? - http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html
        let mut path_locator = match env::var_os("GIS_CONFIG_PATH") {
            Some(val) => PathBuf::from(val),
            None => env::current_dir().unwrap().join("config"),
        };

        path_locator.push(submod);

        // Merge defaults from default
        s.merge(File::from(path_locator.join("default")).required(false))?;

        let env_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::from(path_locator.join(env_mode)).required(true))?;

        // Eg.. `AU_XX=1 ./target/app` will set the `xx` key
        s.merge(Environment::with_prefix(submod))?;

        match s.try_into() {
            Ok(val) => Ok(val),
            Err(e) => Err(SettingsError::ConfigError(e)),
        }
    }
}

#[derive(Debug)]
/// Typically RwLock protected container for global settings HashMap key by submod e.g. au
pub struct SettingsContainer {
    settings: HashMap<String, SettingsDerive>,
}

use state::Storage;
use std::sync::RwLock;

static SETTINGS_CONTAINER: Storage<RwLock<SettingsContainer>> = Storage::new();

fn validate_submod(func: &str, submod: &str) {
    assert!(
        (submod.len() == 2) && submod.chars().all(char::is_alphanumeric),
        "Caller bug: {}() submod must be ISO 3166-1_alpha-2 and {} is not.",
        func,
        submod
    );
}

impl SettingsContainer {
    /// Give Arc<> cloned copy of a subset of configuration e.g. au submod
    fn handle(submod: &str) -> Result<Arc<SettingsDerive>, SettingsError> {
        validate_submod("upsert", submod);
        let mut_container = SETTINGS_CONTAINER.get().read().unwrap();
        let sub_config = mut_container.settings.get(submod).unwrap();
        Ok(Arc::new(sub_config.clone()))
    }
    /// Update or Insert submod e.g. au configuration within the global configuration container
    fn upsert(submod: &str, settings: SettingsDerive) -> Result<bool, SettingsError> {
        validate_submod("upsert", submod);
        let mut_container = SETTINGS_CONTAINER.try_get();
        match mut_container {
            Some(_) => {
                let config_all_lock = mut_container.unwrap();
                let mut config_all = config_all_lock.write().unwrap();
                config_all.settings.insert(submod.to_string(), settings);
            }
            None => {
                let mut init_map = SettingsContainer {
                    settings: HashMap::new(),
                };
                init_map.settings.insert(submod.to_string(), settings);
                SETTINGS_CONTAINER.set(RwLock::new(init_map));
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest_derive::Arbitrary;
    use rstest::rstest;
    use rstest_reuse::{self, *};

    #[derive(Arbitrary, Default, Debug)]
    struct MatcherSettingsDeriveTest {
        catalog: String,
        rest_url: Option<String>,
        query: Option<String>,
        matcher: Option<String>,
    }

    fn settings_derive(fetcher_matcher_val: MatcherSettingsDerive) -> SettingsDerive {
        SettingsDerive {
            fetcher_matcher: fetcher_matcher_val,
        }
    }

    fn fetcher_matcher_test(derive_test: MatcherSettingsDeriveTest) -> MatcherSettingsDerive {
        MatcherSettingsDerive {
            catalog: derive_test.catalog,
            rest_url: derive_test.rest_url,
            query: derive_test.query,
            matcher: derive_test.matcher,
        }
    }

    #[template]
    #[rstest(submod,
             case("au"),
             // no way knowing xx !exist so treat as valid for now.
             // could always hardcode the submod list but benefit?
             case("xx")
    )
    ]
    fn upsert_submod_valid_isostr(submod: &str) {
        let res = SettingsContainer::upsert(submod, SettingsDerive::default());
        assert!(res.unwrap());
    }
    #[apply(upsert_submod_valid_isostr)]
    fn handle_submod_valid_isostr(submod: &str) {
        // mock that global static
        let _foo = SettingsContainer::upsert(submod, SettingsDerive::default());
        let _res = SettingsContainer::handle(submod).unwrap();
    }

    #[template]
    #[rstest(submod, case(")("), case("aaa"), case("+++"))]
    #[should_panic]
    fn upsert_submod_invalid_isostr(submod: &str) {
        let test_config =
            settings_derive(fetcher_matcher_test(MatcherSettingsDeriveTest::default()));
        let _res = SettingsContainer::upsert(submod, test_config);
    }
    #[apply(upsert_submod_invalid_isostr)]
    fn handle_submod_invalid_isostr(submod: &str) {
        let test_config =
            settings_derive(fetcher_matcher_test(MatcherSettingsDeriveTest::default()));
        let _res = SettingsContainer::upsert("au", test_config);
        let _res = SettingsContainer::handle(submod);
    }

    // upsert() Proptest with random props on settings
    proptest! {
        #[test]
        // private fn upsert does not validate fields as it's validated upon serde only
        fn upsert_fetcher_matcher(derive_test: MatcherSettingsDeriveTest) {
            let test_config = settings_derive(fetcher_matcher_test(derive_test));
            let res = SettingsContainer::upsert("au", test_config);
            assert!(res.unwrap());
        }
    }
}
