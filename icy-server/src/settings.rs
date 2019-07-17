use serde::Deserialize;
use config::{Config, ConfigError, File};
use std::collections::{HashMap, BTreeMap};

lazy_static! {
   pub static ref SETTINGS: Settings = Settings::new().unwrap();
   pub static ref ROCKET_CONFIG: rocket::config::Config = init_rocket_config();
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub mqtt: MqttSettings,
    database: DBSettings,
}

#[derive(Debug, Deserialize)]
pub struct MqttSettings {
    pub server: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct DBSettings {
    url: String,
}

impl Settings {
    // On development, load config from current directory
    // on production same location as apt installs it.
    pub fn new() -> Result<Self, ConfigError> {
        let mut c = Config::new();

        let env = rocket::config::Environment::active()
            .expect("Unable to detect rocket env");

        let conf_path = match env {
            rocket::config::Environment::Production =>
                "/usr/local/etc/icy-server/icy-server.toml",
            _ => "icy-server.toml"
        };

        c.merge(File::with_name(conf_path))?;

        c.try_into()
    }
}

fn init_rocket_config() -> rocket::config::Config {
    let mut db_config = BTreeMap::new();
    let mut measurement_db_config = HashMap::new();
    measurement_db_config.insert("url", SETTINGS.database.url.clone());
    db_config.insert("measurements", measurement_db_config);

    let env = rocket::config::Environment::active().expect("Unable to detect rocket env");
    let template_dir = match env {
        rocket::config::Environment::Production => "/usr/local/share/icy-server/",
        _ => "static"
    };

    let config = rocket::config::Config::build(env)
        .extra("databases", db_config)
        .extra("template_dir", template_dir)
        .extra("static_dir", template_dir)
        .finalize()
        .expect("invalid rocket config");

    config
}
