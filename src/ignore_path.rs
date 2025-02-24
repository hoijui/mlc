use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    /// Ignore path '{:?}' not found: {:?}.
    FailedToCanonicalize(PathBuf, std::io::Error),
    /// Ignore path '{path:?}' is neither a dir nor a regular file; Do not know how to use it.
    UnknownPathType(PathBuf),
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum Type {
    /// Matches the whole path, so basically a full, canonical, absolute path to a file
    Whole,
    /// Matches only a prefix of the path.
    Prefix,
}

#[derive(Debug, Deserialize)]
pub struct IgnorePath {
    pub r#type: Type,
    pub path: PathBuf,
}

impl IgnorePath {
    #[must_use]
    pub fn matches(&self, abs_path: &Path) -> bool {
        match self.r#type {
            Type::Whole => self.path == abs_path,
            Type::Prefix => abs_path.starts_with(&self.path),
        }
    }
}

impl TryFrom<&Path> for IgnorePath {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path =
            fs::canonicalize(path).map_err(|err| Error::FailedToCanonicalize(path.into(), err))?;
        let r#type = if path.is_file() {
            Type::Whole
        } else if path.is_dir() {
            Type::Prefix
        } else {
            return Err(Error::UnknownPathType(path));
        };
        Ok(Self { r#type, path })
    }
}

impl TryFrom<&str> for IgnorePath {
    type Error = Error;

    fn try_from(path_str: &str) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(path_str))
    }
}

impl TryFrom<&String> for IgnorePath {
    type Error = Error;

    fn try_from(path_str: &String) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(&path_str))
    }
}

impl IgnorePath {
    #[inline]
    #[must_use]
    pub fn to_str(&self) -> Option<&str> {
        self.path.to_str()
    }
}

pub fn is_valid(path_str: &str) -> Result<(), String> {
    IgnorePath::try_from(path_str)
        .map(|_| ())
        .map_err(|err| format!("{err:?}"))
}

pub fn is_valid_string(path_str: String) -> Result<(), String> {
    is_valid(path_str.as_ref())
}
