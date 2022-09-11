use thiserror::Error;
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Errors {
    
    
    #[error("can not connect to database")]
    DatabaseConnectionError,
    
    #[error("unknown data store error")]
    Unknown,
}