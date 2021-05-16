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
    let settings_config = SettingsDerive::merge_all(submod)?;
    SettingsContainer::upsert(submod, settings_config)?;
    SettingsContainer::handle(submod)
  }
}

use config::{Config, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MatcherSettingsDerive {
  catalog: String,
  pub rest_url: Option<String>,
  pub query: Option<String>,
  pub matcher: Option<String>
}

use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct SettingsDerive {
  pub fetcher_matcher: MatcherSettingsDerive
}

use std::env;
//use camino::Utf8PathBuf;
// @FEATURE: No enforcing of UTF-8/String due to portable impl so we'll go by OsString with no camino::Utf8PathBuf that would give us lossless Display/Debug
use std::path::{PathBuf};

impl SettingsDerive {
  pub fn merge_all(submod: &str) -> Result<Self, SettingsError> {
    let mut s = Config::default();

    // @TODO: Incorporate some XGD crate in future? - http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html
    let mut path_locator = match env::var_os("GIS_CONFIG_PATH") {
      Some(val) => PathBuf::from(val),
      None => env::current_dir().unwrap()
    };
    
    path_locator.push(submod);

    // Merge defaults from default
    s.merge(File::from(path_locator.join("default")).required(false))?;

    let env_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    s.merge(File::from(path_locator.join(env_mode)).required(true))?;
    //s.merge(File::with_name(&format!("config/{}/{}", submod, env)).required(false))?;

    // Eg.. `AU_XX=1 ./target/app` will set the `xx` key
    s.merge(Environment::with_prefix(submod))?;

    match s.try_into() {
      Ok(val) => Ok(val),
      Err(e) => Err(SettingsError::ConfigError(e))
    }

  }

}

#[derive(Debug)]
pub struct SettingsContainer {
  settings: HashMap<String, SettingsDerive>
}

use state::Storage;
use std::sync::RwLock;

static SETTINGS_CONTAINER: Storage<RwLock<SettingsContainer>> = Storage::new();

impl SettingsContainer {
  pub fn handle(submod: &str) -> Result<Arc<SettingsDerive>, SettingsError> {
    let mut_container = SETTINGS_CONTAINER.get().read().unwrap();
    let sub_config = mut_container.settings.get(submod).unwrap();
    Ok(Arc::new(sub_config.clone()))
  }
  pub fn upsert(submod: &str, settings: SettingsDerive) -> Result<bool, SettingsError> {
    let mut_container = SETTINGS_CONTAINER.try_get();
    match mut_container {
      Some(_) => {
        let config_all_lock = mut_container.unwrap();
        let mut config_all = config_all_lock.write().unwrap();
        config_all.settings.insert(submod.to_string(), settings);
      },
      None => {        
        let mut init_map = SettingsContainer { settings: HashMap::new() };
        init_map.settings.insert(submod.to_string(), settings);
        SETTINGS_CONTAINER.set(RwLock::new(init_map));
      }
    }

    Ok(true)
  }
}

