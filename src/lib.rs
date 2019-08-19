use md5::{Md5, Digest};

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
    pub hash: Option<String>,
}
impl Addon {
    pub fn new(name: String) -> Self {
        Self {
            name,
            files: Vec::new(),
            hash: None,
        }
    }
    pub fn line(&mut self) -> String {
        format!("ADDON:{}:{}:{}\n", self.name.clone(), self.files.len(), self.hash())
    }
    pub fn hash(&mut self) -> String {
        if let Some(hash) = &self.hash {
            return hash.to_owned();
        }
        let mut hasher = Md5::new();
        let mut files = self.files.clone();
        files.sort_by(|a,b| a.name.cmp(&b.name));
        for mut file in files {
            hasher.input(&mut file.hash().chars().map(|c| c as u8).collect::<Vec<u8>>()[0..16].to_vec());
        }
        let hash = format!("{:X}", hasher.result());
        self.hash = Some(hash.clone());
        hash
    }
}

#[derive(Clone)]
pub struct SwiftyFile {
    pub name: String,
    pub parts: Vec<FilePart>,
    pub hash: Option<String>,
}
impl SwiftyFile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parts: Vec::new(),
            hash: None,
        }
    }
    pub fn line(&mut self) -> String {
        let mut out = format!("{}:{}:{}:{}:{}\n", if self.name.ends_with(".pbo") {"PBO"} else {"FILE"}, self.name.clone(), self.size(), self.parts.len(), self.hash());
        for part in &self.parts {
            out.push_str(&part.line());
        }
        out
    }
    pub fn hash(&mut self) -> String {
        if let Some(hash) = &self.hash {
            return hash.to_owned();
        }
        let mut hasher = Md5::new();
        for part in &self.parts {
            hasher.input(&mut part.hash.chars().map(|c| c as u8).collect::<Vec<u8>>());
        }
        let hash = format!("{:X}", hasher.result());
        self.hash = Some(hash.clone());
        hash
    }
    pub fn size(&self) -> usize {
        let mut size = 0;
        for part in &self.parts {
            size += part.size;
        }
        size
    }
}

#[derive(Clone)]
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
