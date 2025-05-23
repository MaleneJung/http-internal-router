use std::{
    collections::HashMap, 
    env, 
    fs
};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Router {
    pub port: u16
}

pub type Firewall := HashMap<String, String>;

#[derive(Deserialize)]
pub struct Config {
    pub router: Router,
    pub firewall: Firewall
}

impl Config {

    pub fn from_default_location() -> Result<Self, toml::de::Error> {

        let mut raw: String = String::new();

        if let Ok(mut default_location) = env::current_exe() {
            default_location.pop();
            default_location.push("Config.toml");
            if let Ok(default_read) = fs::read_to_string(default_location) {
                raw = default_read;
            }
        }

        toml::from_str(&raw)

    }

    pub fn apply_firewall_rules(&self, path: &str) -> Option<String> {

        if let Some((path_from, path_to)) = path.split_once('/') {
            for (rule_from, rule_to) in &self.firewall {
                if rule_from.eq_ignore_ascii_case(&path_from) {
                    return Some(rule_to.clone() + "/" + path_to);
                }
            }
        }

        None

    }

}


