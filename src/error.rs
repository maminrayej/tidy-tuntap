use std::io;

pub type Result<T> = std::result::Result<T, Error>;

/// Different errors that can occur.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Specifying len of 0 when creating a multiqueue interface.
    #[error("Multiqueue len cannot be zero")]
    ZeroLenMultiQueue,

    /// Errors happening during IO operations.
    #[error("{0}")]
    IOError(#[from] io::Error),

    /// Errors occurring when calling into the `nix` crate.
    #[error("{0}")]
    NixError(#[from] nix::Error),

    /// Failing to conver the flags returned by the kernel to [`Flags`](crate::flags::Flags).
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
