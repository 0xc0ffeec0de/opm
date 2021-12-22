// use std::env;

const PKG_FMT: &str = "PKG_FMT"; // The package format; It could be .deb, .rpm, etc

#[derive(Debug, Clone, Copy)]
pub enum PackageFormat {
    Deb,
    Rpm,
    Other,
}

impl PackageFormat {
    pub fn get_format() -> Option<Self> {
        if let Ok(pkg_fmt) = std::env::var(PKG_FMT) {
            let pkg_fmt = match pkg_fmt.trim().to_lowercase().as_ref() {
                "deb" => Self::Deb,
                "rpm" => Self::Rpm,
                _ => Self::Other
            };
            Some(pkg_fmt)
        } else {
            None
        }
    }

    pub fn from(fmt: &str) -> Self {
        match fmt {
            "deb" => PackageFormat::Deb,
            "rpm" => PackageFormat::Rpm,
            "oth" => PackageFormat::Other,
            _ => panic!("Invalid format")
        }
    }
}

