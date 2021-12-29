use anyhow::{self, Result, Context};
use crate::repos::config::Config;
use crate::repos::errors::CacheError;
use std::{fs, io::Write};

use super::{
	package::{ControlFile, DebPackage, PkgKind}
};

const DEBIAN_CACHE: &str = "/var/lib/apt/lists/";
struct Cache<'a> {
	cache: &'a str
}

struct CacheResult {
	pkg: Option<DebPackage>,
	pkgs: Option<Vec<DebPackage>>
}

impl<'a> Cache<'a> {
	fn get_cache(config: &'a Config) -> Result<Self> {
		if config.use_pre_existing_cache {
			if !std::path::Path::new(DEBIAN_CACHE).exists() {
				anyhow::bail!(CacheError { msg: format!("{} was not found", DEBIAN_CACHE) });
			}
			
			Ok(
				Cache {
					cache: DEBIAN_CACHE
				}
			)
		} else {
			if !std::path::Path::new(&config.cache).exists() {
				anyhow::bail!(CacheError { msg: format!("{} was not found", config.cache) });
			}

			Ok(
				Cache {
					cache: &config.cache
				}
			)
		}
	}
}

fn cache_inter(config: &Config, name: &str, exact: bool) -> Result<CacheResult> {
	let cache = Cache::get_cache(config)
		.context("Failed to read the cache file")?;
	
	for entry in fs::read_dir(cache.cache)? {
		let entry = entry.unwrap();
		let path = entry.path();
		let path_str = path.clone().into_os_string().into_string().unwrap();

		if path.is_dir() || !path_str.contains('_') {
			continue
		}

		let control = match fs::read_to_string(&path) {
			Ok(v) => v,
			Err(e) => {
				eprintln!("Unexpected error :: {}", e);
				break;
			}
		};

		let mut control = control
		.split("\n\n")
		.map(ControlFile::from)
		.filter_map(|pkg| pkg.ok());
	
		let entry = entry.path()
		.into_os_string()
		.into_string()
		.unwrap();

		let url =  &entry
		.split('/')
		.last()
		.unwrap()
		.replace("_", "/")
		.split('/')
		.collect::<Vec<_>>()[..2]
		.join("/");

		if exact {
			let control = control.find(|pkg| pkg.package == name);

			if let Some(mut pkg) = control {
				let url = format!("{}/{}", url, &pkg.filename);
				pkg.set_filename(&url);
				
				return Ok(
					CacheResult {
						pkg: Some(
							DebPackage {
								control: pkg,
								kind: PkgKind::Binary
							}
						),
						pkgs: None
					}
				);
			} else {
				anyhow::bail!(CacheError { msg: format!("{} was not found", cache.cache) });
			}
		} else {
			let mut pkgs = vec![];
			
			pkgs.append(
				&mut control
				.filter(|pkg| pkg.package.contains(name))
				.map(|mut pkg| {
					let url = format!("{}/{}", url, &pkg.filename);
					pkg.set_filename(&url);
					DebPackage {
						control: pkg,
						kind: PkgKind::Binary
					}
				})
				.collect::<Vec<_>>()
			);

			return Ok(
				CacheResult {
					pkg: None,
					pkgs: Some(pkgs)
				}
			);
		}
	}

	anyhow::bail!(CacheError { msg: format!("{} was not found", cache.cache) });
}

#[inline]
pub fn cache_search(config: &Config, name: &str) -> Result<Option<Vec<DebPackage>>> {
	Ok (
		cache_inter(config, name, false)?.pkgs
	)
}

#[inline]
pub fn cache_lookup(config: &Config, name: &str) -> Result<Option<DebPackage>> {
	Ok (
		cache_inter(config, name, true)?.pkg
	)
}

pub fn db_dump(config: &Config) -> Vec<DebPackage> {
	let db = if config.use_pre_existing_db {
		super::database::DEBIAN_DATABASE
	} else {
		&config.db
	};

	let control = fs::read_to_string(db).unwrap();

	let control = control
		.split("\n\n")
		.map(ControlFile::from)
		.filter_map(|ctrl| ctrl.ok())
		.map(|ctrl| DebPackage { control: ctrl, kind: PkgKind::Binary } )
		.collect::<Vec<_>>();
	
	control
}

#[inline]
pub fn check_installed(config: &Config, name: &str) -> Option<DebPackage> {
	db_dump(config).into_iter().find(|pkg| pkg.control.package == name)
}

pub fn add_package(config: &Config, pkg: DebPackage) -> Result<()> {
	let pkg = pkg.control;
	let db = if config.use_pre_existing_db {
		super::database::DEBIAN_DATABASE
	} else {
		&config.db
	};

	let mut data = format!("Package: {}
Version: {}
Priority: {}
Architecture: {}
Maintainer: {}
Description: {}", pkg.package, pkg.version, pkg.priority, pkg.architecture, pkg.maintainer, pkg.description);

	let mut depends = "".to_string();
	let mut breaks = "".to_string();
	let mut conflicts = "".to_string();

	if let Some(d) = pkg.depends {
		depends = d.join(", ");
		data.push_str(&format!("\nDepends: {}", depends));
	}

	if let Some(d) = pkg.breaks {
		breaks = d.join(", ");
		data.push_str(&format!("\nBreaks: {}", breaks));
	}
	
	if let Some(d) = pkg.conflicts {
		conflicts = d.join(", ");
		data.push_str(&format!("\nConflicts: {}", conflicts));
	}

	data.push('\n');

	let mut file = fs::OpenOptions::new()
		.write(true)
		.append(true)
		.open(db)?;

	if let Err(e) = writeln!(file, "{}", data) {
		eprintln!("Couldn't write to db: {}", e);
	}

	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn get_cache_test() {
		let config = Config::new("deb").unwrap();
		Cache::get_cache(&config).unwrap();
	}

	#[test]
	fn cache_dump_test() {
		let config = Config::new("deb").unwrap();
		cache_dump(&config).unwrap();
	}

	#[test]
	fn db_dump_test() {
		let config = Config::new("deb").unwrap();
		// THIS MAY NOT BE GOOD, IF YOU HAVE AN EMPTY DATABASED IT'LL FAIL
		assert!(db_dump(&config).len() > 0);
	}

	#[test]
	fn cache_lookup_test() {
		let config = Config::new("deb").unwrap();
		let pkg = cache_lookup(&config, "invalidPackage0101").unwrap();
		assert!(pkg.is_none());
	}
}