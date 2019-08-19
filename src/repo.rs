use serde::{Deserialize, Serialize};
use serde_json;

use crate::{SmoothlyError, State};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub repoName: String,
    pub clientParameters: String,
    pub basePath: String,

    pub requiredMods: Vec<Mod>,
    pub optionalMods: Vec<Mod>,
    pub servers: Vec<Server>,

    #[serde(default="default_version")]
    pub version: String,

    #[serde(skip_serializing_if="String::is_empty")]
    #[serde(default="String::new")]
    pub imageChecksum: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    pub modName: String,
    pub Enabled: bool,
    #[serde(skip_serializing_if="String::is_empty")]
    #[serde(default="String::new")]
    pub checkSum: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub name: String,
    pub address: String,
    pub port: String,
    pub password: String,
    pub battleEye: bool,
}

impl Repo {
    pub fn new(name: String) -> Result<Self, SmoothlyError> {
        serde_json::from_str(&std::fs::read_to_string(name)?).map_err(SmoothlyError::from)
    }

    pub fn has_mod(&self, name: &str) -> bool {
        for arma_mod in &self.requiredMods {
            if arma_mod.modName == name {
                return true;
            }
        }
        for arma_mod in &self.optionalMods {
            if arma_mod.modName == name {
                return true;
            }
        }
        false
    }

    pub fn mod_state(&self, name: &str) -> State {
        for arma_mod in &self.requiredMods {
            if arma_mod.modName == name {
                return if arma_mod.Enabled { State::Enabled } else { State::Disabled }
            }
        }
        for arma_mod in &self.optionalMods {
            if arma_mod.modName == name {
                return if arma_mod.Enabled { State::OptionalEnabled } else { State::OptionalDisabled }
            }
        }
        State::Disabled
    }

    pub fn set_hash(&mut self, name: &str, hash: String) {
        for arma_mod in &mut self.requiredMods {
            if arma_mod.modName == name {
                arma_mod.checkSum = hash;
                return;
            }
        }
        for arma_mod in &mut self.optionalMods {
            if arma_mod.modName == name {
                arma_mod.checkSum = hash;
                return;
            }
        }
    }
}

pub fn default_version() -> String {
    "3.0.0.0".to_owned()
}
