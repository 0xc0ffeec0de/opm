use anyhow::{Context, Result};
use ar::Archive;
use tar::Archive as tarar;
use xz2::read::XzDecoder;
use flate2::read::GzDecoder;

use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::str;

use crate::repos::{config::Config, errors::InstallError};
use super::package::{DebPackage, PkgKind, Info};

pub struct Package(pub DebPackage, pub Info);

fn unpack(filename: &str, dst: &str) -> Result<()> {
    let file = File::open(&filename)?;

    if filename.ends_with(".tar.gz") {
        let tar = GzDecoder::new(file);
        let mut archive = tarar::new(tar);
        archive.unpack(dst)
            .with_context(|| format!("Could not unpack {} archive", filename))?;
    } else if filename.ends_with(".tar.xz") {
        let tar = XzDecoder::new(file);
        let mut archive = tarar::new(tar);
        archive.unpack(dst)
            .with_context(|| format!("Could not unpack {} archive", filename))?;
    }

    Ok(())
}

pub fn extract(config: &Config, path: &str) -> Result<Package> {
    let mut archive = Archive::new(File::open(path).expect("msg"));
    let mut bytes: Vec<u8> = Vec::new();

    let mut file = File::open(path)
        .with_context(|| format!("Could not open the file {}", path))?;
    
    file.read_to_end(&mut bytes)
        .with_context(|| format!("Could not read the file {}", path))?;

    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result?;
        
        let filename = str::from_utf8(entry.header().identifier()).unwrap().to_string();
        let mut file = File::create(&filename)
            .with_context(|| "Could not create path file")?;

        io::copy(&mut entry, &mut file)
            .with_context(|| "Could not copy the contents of the file")?;

        match filename.as_ref() {
            "data.tar.xz"|"data.tar.gz" => unpack(&filename, &config.tmp)?,
            "control.tar.xz"|"control.tar.gz" => unpack(&filename, &config.info)?,
            _ => ()
        }

        fs::remove_file(&filename)
            .with_context(|| format!("Could not remove {}", filename))?;
    }

    println!("Done");
    let info = super::package::Info::load(std::path::Path::new(&config.info))?;
    let pkg = DebPackage::new(config, &info, PkgKind::Binary)?;

    if pkg.control.breaks.is_some() {
        anyhow::bail!(InstallError::Breaks(pkg.control.package))
    }
    
    Ok(
        Package(pkg, info)
    )
}
