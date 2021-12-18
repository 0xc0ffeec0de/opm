// use std::error::Error;
use std::fmt::{self, Display};
use std::io::Error as ioError;
use reqwest::Error as reqwestError;

// TODO: Make better erros and create more
#[derive(Debug)]
pub enum InstallError {
    InvalidPackage(String),
    IoError(String),
    NetworkingError(String),
    DataBaseError(String),
    AlreadyInstalled
}
#[derive(Debug)]pub enum SetupError {
    Error(String)
}
#[derive(Debug)]
pub enum ConfigError {
    Error(String)
}

impl Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallError::InvalidPackage(msg) => write!(f, "Invalid Package => {}", msg),
            InstallError::IoError(msg) => write!(f, "I/O Error => {}", msg),
            InstallError::NetworkingError(msg) => write!(f, "Networking Error => {}", msg),
            InstallError::DataBaseError(msg) => write!(f, "DataBase Error => {}", msg),
            InstallError::AlreadyInstalled => write!(f, "Package is already installed")
        }
    }
}

impl From<ioError> for InstallError {
    fn from(err: ioError) -> Self {
        InstallError::IoError(err.to_string())
    }
}

impl From<reqwestError> for InstallError {
    fn from(err: reqwestError) -> Self {
        InstallError::NetworkingError(err.to_string())
    }
}

impl From<ConfigError> for InstallError {
    fn from(err: ConfigError) -> Self {
        InstallError::InvalidPackage(err.to_string())
    }
}

impl Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetupError::Error(msg) => write!(f, "{}", msg)
        }
    }
}

impl<E: std::error::Error + 'static> From<E> for SetupError {
    fn from(error: E) -> Self {
        SetupError::Error(error.to_string())
    }
}

impl<E: std::error::Error + 'static> From<E> for ConfigError {
    fn from(error: E) -> Self {
        ConfigError::Error(error.to_string())
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Error(msg) => write!(f, "{}", msg)
        }
    }
}

impl From<InstallError> for ConfigError {
    fn from(err: InstallError) -> Self {
        ConfigError::Error(err.to_string())
    }
}