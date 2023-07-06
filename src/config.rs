/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: ConfigServer,
    pub rules: Vec<ConfigRule>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigServer {
    pub bind: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigRule {
    pub name: String,
    pub method: Option<String>,
    pub path: Option<String>,
    pub headers: Option<Vec<ConfigRuleHeader>>,
    pub action: String,
    pub redirect_to: Option<String>,
    pub redirect_status: Option<u16>,
    pub proxy_url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigRuleHeader {
    pub name: String,
    pub value: Option<String>,
}

impl Config {
    pub fn create() -> Config {
        let filename = std::env::args().nth(1);

        if filename.is_none() {
            tracing::error!(target: "Config", "no configuration file");
            std::process::exit(1);
        }

        Config::create_from_filename(&filename.unwrap())
    }

    pub fn create_from_filename(filename: &str) -> Config {
        let content = match std::fs::read_to_string(&filename) {
            Ok(c) => c,
            Err(_) => {
                tracing::error!(target: "Config", filename=filename, "could not read file");
                std::process::exit(1);
            }
        };
        let config: Config = match serde_yaml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!(target: "Config", filename=filename, error = e.to_string(), "unable to load data");
                std::process::exit(1);
            }
        };

        config
    }
}
