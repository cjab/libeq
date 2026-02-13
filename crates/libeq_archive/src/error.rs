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
