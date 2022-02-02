use clap::{Arg, App, SubCommand, AppSettings};
use std::process;

fn main() {
	let mut config = opm::setup().unwrap_or_else(|err| {
		eprintln!("Could not setup the package manager due {}", err);
		opm::roll_back();
		process::exit(1);
	});


    let matches = App::new("Oxidized Package Manager")
				.setting(AppSettings::ArgRequiredElseHelp)
				.version("v1.5.1-beta")
				.author("FallAngel <fallangel@protonmail.com>")
				.about("A package manager fully written in Rust")
				.arg(Arg::with_name("list")
					.short("l")
					.long("list")
					.takes_value(false)
					.help("List all installed packages")
				)
				.subcommands( vec![
					SubCommand::with_name("install")
						.about("Install a package")
						.arg(Arg::with_name("package")
							.takes_value(true)
							.index(1)
							.required(true))
						.arg(Arg::with_name("force")
							.short("d")
							.long("dest")
							.takes_value(true)
							.index(2)
							.help("Install the package on <dest>/ folder"))
						.arg(Arg::with_name("force")
							.short("f")
							.long("force")
							.takes_value(false)
							.help("Force the installation of pacakages that may can break others")),
					SubCommand::with_name("update")
						.about("Update opm's packages cache"),
					SubCommand::with_name("remove")
						.about("Remove a package")
						.arg(Arg::with_name("package")
						.takes_value(true)
						.index(1)
						.required(true))
						.arg(Arg::with_name("purge")
							.required(false)
							.takes_value(false)
							.long("purge")
							.short("p")
							.help("Remove every file related to the package")),
					SubCommand::with_name("search")
						.about("Search for a package in the cache")
						.arg(Arg::with_name("package")
							.takes_value(true)
							.index(1)
							.required(true)),
					SubCommand::with_name("clear")
						.about("Clear OPM's cache")
				])
				.get_matches();

	match matches.occurrences_of("list") {
		0 => (),
		1 => opm::list_installed(&config),
		_ => println!("Invalid argument")
	};

    if let Some(package) = matches.subcommand_matches("install") {
		let force = !matches!(matches.occurrences_of("force"), 0);

        opm::install(&mut config, package.value_of("package").unwrap(), force, package.value_of("dest").map(|a| a.to_string())).unwrap_or_else(|err| {
            eprintln!("InstallError :: {}", err);
            process::exit(1);
        });
    }

    if matches.subcommand_matches("update").is_some() {
        opm::update(&mut config).unwrap_or_else(|err| {
			eprintln!("UpdateError :: {}", err);
			process::exit(1);
		})
    }

    if let Some(rm) = matches.subcommand_matches("remove") {
        let pkg = rm.value_of("package").unwrap();
		if rm.is_present("purge") {
			println!("Purgin 'em all HAHAHA");
			opm::remove(&config, pkg, true).unwrap_or_else(|err| {
				eprintln!("Could not purge {} :: {}", pkg, err);
				process::exit(1);
			});
		} else {
			opm::remove(&config, pkg, false).unwrap_or_else(|err| {
				eprintln!("Could not remove {} :: {}", pkg, err);
				process::exit(1);
			});
		}
    };

    if let Some(package) = matches.subcommand_matches("search") {
		let pkg =  package.value_of("package").unwrap();
		println!("Searching for {} ...", package.value_of("package").unwrap());
		opm::search(&mut config, pkg).unwrap_or_else(|err| {
			eprintln!("Failed to search for {} due {}", pkg, err);
			process::exit(1);
		});
    };

    if matches.subcommand_matches("clear").is_some() {
		opm::clear(&config).unwrap_or_else(|err| {
			eprintln!("Failed to clear cache due {}", err);
			process::exit(1);
		});
    };
}
