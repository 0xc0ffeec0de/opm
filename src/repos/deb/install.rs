///
/// Debian package install
/// 

use crate::repos::{errors::InstallError, deb::dependencies};
use crate::repos::config::Config;
use super::cache;
use super::{extract, download};
use super::scripts;

// TODO: Check for newer versions of the package if installed
pub fn install(config: &mut Config, name: &str) -> Result<(), InstallError> {
    if name.ends_with(".deb") {
        let pkg = extract::extract(name, &config.tmp)?;
        if let Some(pkg) = cache::check_installed(&pkg.control.package) {
            println!("{} is already installed\nFound:", name);
            println!("{} - {}", pkg.control.package, pkg.control.version);
            return Err(InstallError::AlreadyInstalled);
        }
        println!("Extracting ...");
        println!("Done");
        scripts::execute(&config.tmp)?;
    } else {
        if let Some(pkg) = cache::check_installed(name) {
            println!("{} is already installed\nFound:", name);
            println!("{} - {}", pkg.control.package, pkg.control.version);
            return Err(InstallError::AlreadyInstalled);
        }

        // Downloand and call install on the downloaded packages
        println!("Downloading {} for debian ...", name);

        if let Some(pkg) = cache::cache_lookup(config, name) {
            println!("Found {:?}", pkg.control.package);
            if let Some(dep) = dependencies::get_dependencies(config, &pkg) {
                let deps = dep.0;
                let sugg = dep.1;

                println!("Installing {} NEW packages", deps.len());
                deps.iter().for_each(|pkg| print!("{} ", pkg.control.package));
                println!();
                println!("Suggested packages:");
                sugg.iter().for_each(|pkg| print!("{} ", pkg));
                
                deps.into_iter().for_each(|pkg| {
                    if let Ok(path) = download::download(config, &pkg) {
                        let path = path
                            .into_os_string()
                            .into_string().unwrap();
                        
                        extract::extract(&path, std::path::Path::new("/"))
                            .unwrap_or_else(|e| panic!("Failed dependencie extraction due {}", e));
                    }
                })
            }
            let path = download::download(config, &pkg).unwrap();
            let path = path
                .into_os_string()
                .into_string().unwrap();
            
            extract::extract(&path, std::path::Path::new("/"))
                .unwrap_or_else(|e| panic!("Failed package extraction due {}", e));
            scripts::execute(&config.tmp)?;
        } else {
            println!("Package {} was not found!", name);
        }

    }
    

    Ok(())
}