use anyhow::Result;
use std::{path::Path, io::{self, ErrorKind, Write}};
use super::{config::Config, utils::PackageFormat};

fn get_answer() -> Result<String> {
    let mut answer = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut answer)?;
    // Ok(answer.to_ascii_lowercase().trim().chars().next().unwrap())
    Ok(answer)
}

pub fn setup() -> Result<Config> {
    #[allow(deprecated)]
    let home = std::env::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
        
    let root = format!("{}/.opm/", home);
    let config_file = format!("{}/config.json", root);
    let config;

    if Path::new(&config_file).exists() {
        config = Config::from(&config_file);
    } else {
        println!("Entering setup mode ...");
        match PackageFormat::get_format()? {
            PackageFormat::Deb => {
                print!("Are you on a Debian-based distro? [y/n] ");
                if get_answer()?.to_ascii_lowercase().trim().starts_with('y') {
                    config = Config::new("deb")?
                } else {
                    print!("Insert the package format: ");
                    config = Config::new(get_answer()?.trim())?
                }
            },
            PackageFormat::Rpm => {
                print!("Are you on a RHEL-based distro? [y/n] ");
                if get_answer()?.to_ascii_lowercase().trim().starts_with('y') {
                    config = Config::new("rpm")?
                } else {
                    print!("Insert the package format: ");
                    config = Config::new(get_answer()?.trim())?
                }
            }
            PackageFormat::Other => panic!("Unrecognized package"),
        }
        
        config.setup()?;
        println!("Done");
        let config_file = format!("{}config.json", root);
        println!("Saving config file to {}", config_file);
        config.save(&config_file);
    }

    Ok(config)
}

#[allow(deprecated)]
pub fn roll_back() {
    println!("Rolling back ...");
    let home = std::env::home_dir().unwrap()
    .into_os_string().into_string().unwrap();
    let root = format!("{}/.opm/", home);

    match std::fs::remove_dir_all(root){
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => (),
            _ => panic!("Clould not rollback due {}", e)
        }
    }
}