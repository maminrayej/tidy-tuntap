#[cfg(feature = "mq")]
mod mq;

#[cfg(feature = "mq")]
pub use mq::*;

use std::{io, sync};

use crate::error::Result;
use crate::iface;

pub struct Tap {
    iface: std::sync::Arc<iface::Interface>,

    // Dropping this file hence closing it is the responsibility of the `iface`.
    file: std::mem::ManuallyDrop<std::fs::File>,
}

impl std::ops::Deref for Tap {
    type Target = iface::Interface;

    fn deref(&self) -> &Self::Target {
        self.iface.as_ref()
    }
}

impl Tap {
    pub(crate) fn new(iface: sync::Arc<iface::Interface>, fd_index: usize) -> Result<Self> {
        let file = iface.files[fd_index].try_clone()?;

        Ok(Tap {
            iface,
            file: std::mem::ManuallyDrop::new(file),
        })
    }

    fn from_params(iface_params: iface::InterfaceParams) -> Result<Self> {
        let iface = iface::Interface::new(iface_params)?;

        Tap::new(sync::Arc::new(iface), 0)
    }

    pub fn without_packet_info(name: &str) -> Result<Tap> {
        Self::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: true,
        })
    }

    pub fn with_packet_info(name: &str) -> Result<Tap> {
        Self::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: false,
        })
    }

    pub fn iface(&self) -> &iface::Interface {
        self.iface.as_ref()
    }
}

impl io::Read for Tap {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl io::Write for Tap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
