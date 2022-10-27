use thiserror::Error;
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LinkErrors {
    
    
    #[error("can not connect to database")]
    DatabaseConnectionError,

    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("Toml error")]
    TomlError(#[from] toml::de::Error),
    
    #[error("unknown data store error")]
    Unknown,
}