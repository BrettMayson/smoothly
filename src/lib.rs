#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod macros;

pub mod error;
pub use error::{IOPathError, SmoothlyError};

pub mod commands;
pub use commands::Command;

mod repo;
pub use repo::{Repo, Server, Mod};

#[derive(PartialEq, Debug, Clone)]
pub enum Transaction {
    Add,
    Update,
    Remove,
    Ignore,
    Existing,
}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    Disabled,
    Enabled,
    OptionalDisabled,
    OptionalEnabled,
}

pub struct Addon {
    pub name: String,
    pub files: Vec<SwiftyFile>,
}
impl Addon {
    pub fn new(name: String) -> Self {
        Self {
            name,
            files: Vec::new(),
        }
    }
    pub fn line(&self) -> String {
        format!("ADDON:{}:{}:{}\n", self.name, self.files.len(), self.hash())
    }
    pub fn hash(&self) -> String {
        let mut hashes = Vec::new();
        for file in &self.files {
            hashes.append(&mut file.name.chars().map(|c| c as u8).collect::<Vec<u8>>());
            //hashes.append(&mut ":".chars().map(|c| c as u8).collect::<Vec<u8>>());
            hashes.append(&mut file.hash().chars().map(|c| c as u8).collect::<Vec<u8>>());
            //hashes.append(&mut "$$".chars().map(|c| c as u8).collect::<Vec<u8>>());
            for part in &file.parts {
                //hashes.append(&mut part.name.chars().map(|c| c as u8).collect::<Vec<u8>>());
                //hashes.append(&mut ":".chars().map(|c| c as u8).collect::<Vec<u8>>());
                //hashes.append(&mut part.hash.chars().map(|c| c as u8).collect::<Vec<u8>>());
            }
        }
        format!("{:X}", md5::compute(&hashes))
    }
}

pub struct SwiftyFile {
    pub name: String,
    pub parts: Vec<FilePart>,
}
impl SwiftyFile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parts: Vec::new(),
        }
    }
    pub fn line(&self) -> String {
        let mut out = format!("{}:{}:{}:{}:{}\n", if self.name.ends_with(".pbo") {"PBO"} else {"FILE"}, self.name, self.size(), self.parts.len(), self.hash());
        for part in &self.parts {
            out.push_str(&part.line());
        }
        out
    }
    pub fn hash(&self) -> String {
        let mut hashes = Vec::new();
        for part in &self.parts {
            hashes.append(&mut part.hash.chars().map(|c| c as u8).collect::<Vec<u8>>());
        }
        format!("{:X}", md5::compute(&hashes))
    }
    pub fn size(&self) -> usize {
        let mut size = 0;
        for part in &self.parts {
            size += part.size;
        }
        size
    }
}

pub struct FilePart {
    pub name: String,
    pub start: usize,
    pub size: usize,
    pub hash: String,
}
impl FilePart {
    pub fn line(&self) -> String {
        format!("{}:{}:{}:{}\n", self.name, self.start, self.size, self.hash)
    }
}
