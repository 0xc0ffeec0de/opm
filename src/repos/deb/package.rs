use std::collections::HashMap;
use std::fs;
use std::io::Error;

///
/// Kind of the package
///
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PkgKind {
    Binary,
    Source,
}

/**
 * 
    Package (mandatory)
    Source
    Version (mandatory)
    Section (recommended)
    Priority (recommended)
    Architecture (mandatory)
    Essential
    Depends et al
    Installed-Size
    Maintainer (mandatory)
    Description (mandatory)
    Homepage
    Built-Using
 */

///
/// Debian's control file (mandatory fields)
///
#[derive(Debug, Clone)]
pub struct ControlFile { // We could improve by using lifetimes
    pub package: String,
    pub version: String,
    pub architecture: String,
    pub maintainer: String,
    pub description: String,
    pub depends: Vec<String>,
    pub filename: String,
}

// TODO: Improve this in the future
impl ControlFile {
    pub fn new(file: &str) -> Result<Self, Error> {
        let contents = fs::read_to_string(file)?;

        let mut map: HashMap<String, String> = HashMap::new();

        for line in contents.lines() {
            let values = line.split(":").collect::<Vec<&str>>();
            map.insert(String::from(*values.get(0).unwrap_or(&"NONE")), String::from(*values.get(1).unwrap_or(&"NONE")));
        };

        Ok(
            Self {
                package: map.get("Package").unwrap_or(&String::from("NONE")).trim().to_owned(),
                version: map.get("Version").unwrap_or(&String::from("NONE")).trim().to_owned(),
                architecture: map.get("Architecture").unwrap_or(&String::from("NONE")).trim().to_owned(),
                maintainer: map.get("Maintainer").unwrap_or(&String::from("NONE")).trim().to_owned(),
                description: map.get("Description").unwrap_or(&String::from("NONE")).trim().to_owned(),
                depends: Self::parse_dependencies(map.get("Depends").unwrap_or(&String::from("NONE")).trim()),
                filename: map.get("Filename").unwrap_or(&String::from("NONE")).trim().to_owned(),
            }
        )
    }

    pub fn from(contents: &str) -> Result<Self, Error> {
        let mut map: HashMap<String, String> = HashMap::new();

        for line in contents.lines() {
            let values = line.splitn(2, ":").collect::<Vec<&str>>();
            map.insert(String::from(*values.get(0).unwrap_or(&"NONE")), String::from(*values.get(1).unwrap_or(&"NONE")));
        };

        Ok(
            Self {
                package: map.get("Package").unwrap_or(&String::from("NONE")).trim().to_owned(),
                version: map.get("Version").unwrap_or(&String::from("NONE")).trim().to_owned(),
                architecture: map.get("Architecture").unwrap_or(&String::from("NONE")).trim().to_owned(),
                maintainer: map.get("Maintainer").unwrap_or(&String::from("NONE")).trim().to_owned(),
                description: map.get("Description").unwrap_or(&String::from("NONE")).trim().to_owned(),
                depends: Self::parse_dependencies(map.get("Depends").unwrap_or(&String::from("NONE")).trim()),
                filename: map.get("Filename").unwrap_or(&String::from("NONE")).trim().to_owned(),
            }
        )
    }

    // TODO: Make this better to read/understand
    fn parse_dependencies(dependencies: &str) -> Vec<String> {
        let dependencies = dependencies
            .split(",")
            .map(|d| d.trim().to_owned())
            .collect::<Vec<_>>();
            
        dependencies
    }
}

/// 
/// Debian binary package format structure
///
#[derive(Debug, Clone)]
pub struct DebPackage {
    pub control: ControlFile,
    pub signature: String,
    pub kind: PkgKind,
}

impl DebPackage {
    pub fn new(file: &str, kind: PkgKind, signature: String) -> Result<Self, Error> {
        Ok(
            DebPackage {
                control: ControlFile::new(file)?,
                signature,
                kind
            }
        )
    }
}
