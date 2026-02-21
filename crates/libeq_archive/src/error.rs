use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Corrupt archive: {0}")]
    CorruptArchive(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::IO(io_err) => io_err,
            other => std::io::Error::new(std::io::ErrorKind::Other, other),
        }
    }
}
