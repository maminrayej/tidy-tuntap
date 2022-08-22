#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_tap;
#[cfg(feature = "tokio")]
pub use async_tap::*;

use std::{io, sync};

use crate::dev;
use crate::error::Result;
use crate::iface;

pub struct Tap(dev::Dev);

impl std::ops::Deref for Tap {
    type Target = dev::Dev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Tap {
    pub(crate) fn new(iface: sync::Arc<iface::Interface>, fd_index: usize) -> Result<Self> {
        Ok(Tap(dev::Dev::new(iface, fd_index)?))
    }

    pub fn without_packet_info(name: &str) -> Result<Tap> {
        Ok(Tap(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: true,
        })?))
    }

    pub fn with_packet_info(name: &str) -> Result<Tap> {
        Ok(Tap(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: false,
        })?))
    }
}

impl io::Read for Tap {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Write for Tap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
