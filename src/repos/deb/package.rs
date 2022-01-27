use anyhow::{self, Result, bail};
use crate::repos::{errors::ConfigError, config::Config};
use std::{collections::HashMap, path::{PathBuf, Path}};
use std::fs;

///
/// Kind of the package
///
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PkgKind {
    Binary,
    Source,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PkgPriority {
    Required,
    Important,
    Standard,
    Optional,
    Extra, // Deprecated, but here for compatibility issues
}

#[allow(dead_code)]
impl PkgPriority {
    fn get_priority(p: &str) -> Self {
        match p {
            "required" => PkgPriority::Required,
            "important" => PkgPriority::Important,
            "standard" => PkgPriority::Standard,
            "optional" => PkgPriority::Optional,
            "extra" => PkgPriority::Extra,
            _ => PkgPriority::Optional
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub conffiles: Option<PathBuf>,
    pub control: Option<PathBuf>,
    pub md5sums: Option<PathBuf>,
    pub preinst: Option<PathBuf>,
    pub postinst: Option<PathBuf>,
    pub prerm: Option<PathBuf>,
    pub postrm: Option<PathBuf>,
}

impl Info {
    pub fn load(from: &Path) -> Result<Self> {
        let mut result = Self {
            conffiles: None,
            control: None,
            md5sums: None,
            preinst: None,
            postinst: None,
            prerm: None,
            postrm: None,
        };

        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let path = entry.path();

            match path.clone().into_os_string().into_string().unwrap().rsplit('/').next().unwrap() {
                "conffiles" => result.conffiles = Some(path.clone()),
                "control" => result.control = Some(path.clone()),
                "md5sums" => result.md5sums = Some(path.clone()),
                "preinst" => result.preinst = Some(path.clone()),
                "postinst" => result.postinst = Some(path.clone()),
                "prerm" => result.prerm = Some(path.clone()),
                "postrm" => result.postrm = Some(path.clone()),
                _ => ()
            }
        }

        Ok(result)
    }
}

///
/// Debian's control file (mandatory fields)
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFile {
    pub package: String,
    pub version: String,
    pub priority: String,
    pub architecture: String,
    pub maintainer: String,
    pub description: String,
    pub depends: Option<Vec<String>>,
    pub recommends: Option<Vec<String>>,
    pub suggests: Option<Vec<String>>,
    pub enhances: Option<Vec<String>>,
    pub pre_depends: Option<Vec<String>>,
    pub breaks: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
    pub conffiles: Option<Vec<String>>,
    pub filename: String,
    pub size: String,
    pub md5sum: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String
}

// TODO: Improve this in the future
impl ControlFile {
    pub fn new(_config: &Config, contents: &str) -> Result<Self> {        
        let mut map: HashMap<String, String> = HashMap::new();

        contents.lines().map(|line| line.trim()).for_each(|line| {
            let values = line.splitn(2, ':').map(|line| line.to_owned()).collect::<Vec<_>>();
            if values.len() == 2 {
                map.insert(values[0].clone(), values[1].clone());
            }
        });

        Ok(
            Self {
                package: Self::try_get(&map, "Package")?,
                version: Self::try_get(&map, "Version")?,
                architecture: Self::try_get(&map, "Architecture")?,
                maintainer: Self::try_get(&map, "Maintainer")?,
                description: Self::try_get(&map, "Description")?,
                // Should be like the others
                // But, when reading /var/lib/dpkg/status it does not have those fields
                priority: Self::try_get(&map, "Priority").unwrap_or_default(),
                depends: Self::split_optional(Some(&Self::try_get(&map, "Depends").unwrap_or_default())),
                recommends: Self::split_optional(Some(&Self::try_get(&map, "Recommends").unwrap_or_default())),
                suggests: Self::split_optional(Some(&Self::try_get(&map, "Suggests").unwrap_or_default())),
                enhances: Self::split_optional(Some(&Self::try_get(&map, "Enhances").unwrap_or_default())),
                pre_depends: Self::split_optional(Some(&Self::try_get(&map, "Pre-Depends").unwrap_or_default())),
                breaks: Self::split_optional(Some(&Self::try_get(&map, "Breaks").unwrap_or_default())),
                conflicts: Self::split_optional(Some(&Self::try_get(&map, "Conflicts").unwrap_or_default())),
                conffiles: None,
                filename: Self::try_get(&map, "Filename").unwrap_or_default(),
                size: Self::try_get(&map, "Size").unwrap_or_default(),
                md5sum: Self::try_get(&map, "MD5sum").unwrap_or_default(),
                sha1: Self::try_get(&map, "SHA1").unwrap_or_default(),
                sha256: Self::try_get(&map, "SHA256").unwrap_or_default(),
                sha512: Self::try_get(&map, "SHA512").unwrap_or_default(),
            }
        )
    }

    pub fn from_info(config: &Config, info: &Info) -> Result<Option<Self>> {
        if let Some(control) = &info.control {
            let mut result = Self::new(config, &fs::read_to_string(&control)?)?;
            
            if let Some(conffiles) = &info.conffiles {
                result.conffiles = Some(fs::read_to_string(conffiles)?.lines().map(|line| line.trim().to_string()).collect::<Vec<_>>());
            }

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    fn try_get(hashmap: &HashMap<String, String>, field: &str) -> Result<String> {
        if let Some(v) = hashmap.get(field) {
            Ok (v.trim().to_owned())
        } else {
            bail!(ConfigError::UnexError { msg: format!("Invalid debain's control file! Missing \"{}\" field", field), err: None });
        }
    }

    fn split_optional(dependencies: Option<&str>) -> Option<Vec<String>> {
        if let Some(val) = dependencies {
            if !val.is_empty() {
                let val = val
                    .split(',')
                    .map(|d| d.trim().to_owned())
                    .collect::<Vec<_>>();
                Some(val)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.filename = filename.to_owned();
    }
}

/// 
/// Debian binary package format structure
///
#[derive(Debug, Clone, PartialEq)]
pub struct DebPackage {
    pub control: ControlFile,
    pub kind: PkgKind,
}

impl DebPackage {
    pub fn new(config:&Config, info: &Info, kind: PkgKind) -> Result<Self> {
        Ok(
            DebPackage {
                control: ControlFile::from_info(config, info)?.unwrap(),
                kind,
            }
        )
    }
}

#[cfg(test)]
mod test {
	use super::*;
    #[test]
    fn package_from_test() {
        let config = crate::repos::setup(None).unwrap();
        let data = r"Package: accountsservice
Architecture: amd64
Version: 0.6.55-0ubuntu11
Priority: standard
Section: gnome
Origin: Ubuntu
Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
Original-Maintainer: Debian freedesktop.org maintainers <pkg-freedesktop-maintainers@lists.alioth.debian.org>
Bugs: https://bugs.launchpad.net/ubuntu/+filebug
Installed-Size: 452
Depends: dbus, libaccountsservice0 (= 0.6.55-0ubuntu11), libc6 (>= 2.4), libglib2.0-0 (>= 2.44), libpolkit-gobject-1-0 (>= 0.99)
Suggests: gnome-control-center
Filename: pool/main/a/accountsservice/accountsservice_0.6.55-0ubuntu11_amd64.deb
Size: 60940
MD5sum: 87a0e27c83950d864d901ceca0f2b49c
SHA1: ce92ea3783ca4ca6cdb5115381379f9c1317566b
SHA256: e34884d71bb98002bf0c775479aa31ee5011ded1abf969ffe6496874de499f42
Homepage: https://www.freedesktop.org/wiki/Software/AccountsService/
Description: query and manipulate user account information
Task: standard
Description-md5: 8aeed0a03c7cd494f0c4b8d977483d7e";
		ControlFile::new(&config, data).unwrap();
	}

}