#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_dev;
#[cfg(feature = "tokio")]
pub use async_dev::*;

use std::os::unix::prelude::{AsRawFd, RawFd};
use std::{fs, io, mem, sync};

use crate::{error, iface};

/// Representing a blocking TUN/TAP device.
pub struct Dev {
    iface: sync::Arc<iface::Interface>,

    // Owner of this file is `iface`.
    // Therefore we wrap it in manually drop to not double close the file.
    file: mem::ManuallyDrop<fs::File>,
}

impl std::ops::Deref for Dev {
    type Target = iface::Interface;

    fn deref(&self) -> &Self::Target {
        self.iface.as_ref()
    }
}

impl Dev {
    // Creates new `fd_index`th `Dev`, bound to the `iface`.
    // For example in a multiqueue scenario, calling this function with `fd_index` = 0 will create
    // the first `Dev` bound to the `iface`.
    pub(crate) fn new(iface: sync::Arc<iface::Interface>, fd_index: usize) -> error::Result<Self> {
        let file = iface.files[fd_index].try_clone()?;

        Ok(Dev {
            iface,
            file: mem::ManuallyDrop::new(file),
        })
    }

    // Creates a `Dev` using the parameters specified by `iface_params`.
    pub(crate) fn from_params(iface_params: iface::InterfaceParams) -> error::Result<Self> {
        let iface = iface::Interface::new(iface_params)?;

        Dev::new(sync::Arc::new(iface), 0)
    }

    /// Blocks and writes the data in `buf` into the device.
    ///
    /// # Arguments
    /// * `buf`: Data to be written into the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes written.
    /// * `Err`: If any error occurs.
    pub fn send(&self, buf: &[u8]) -> error::Result<usize> {
        Ok(nix::unistd::write(self.file.as_raw_fd(), buf)?)
    }

    /// Blocks and read the data from device into `buf`.
    ///
    /// # Arguments
    /// * `buf`: Buffer to be filled with data read from the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes read.
    /// * `Err`: If any error occurs.
    pub fn recv(&self, buf: &mut [u8]) -> error::Result<usize> {
        Ok(nix::unistd::read(self.file.as_raw_fd(), buf)?)
    }
}

impl io::Read for Dev {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl io::Write for Dev {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl AsRawFd for Dev {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
