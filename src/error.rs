use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Multiqueue len cannot be zero")]
    ZeroLenMultiQueue,

    #[error("{0}")]
    IOError(#[from] io::Error),

    #[error("{0}")]
    NixError(#[from] nix::Error),

    #[error("Failed to create Flags from the data returned by the kernel: {0:b}")]
    ConversionError(i32),
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::IOError(io_err) => io_err,
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}
