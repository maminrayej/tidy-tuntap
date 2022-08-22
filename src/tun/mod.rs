#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_tun;
#[cfg(feature = "tokio")]
pub use async_tun::*;

use std::{io, sync};

use crate::error::Result;
use crate::{dev, iface};

pub struct Tun(dev::Dev);

impl std::ops::Deref for Tun {
    type Target = dev::Dev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Tun {
    pub(crate) fn new(iface: sync::Arc<iface::Interface>, fd_index: usize) -> Result<Self> {
        Ok(Tun(dev::Dev::new(iface, fd_index)?))
    }

    pub fn without_packet_info(name: &str) -> Result<Tun> {
        Ok(Tun(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: true,
        })?))
    }

    pub fn with_packet_info(name: &str) -> Result<Tun> {
        Ok(Tun(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: false,
        })?))
    }
}

impl io::Read for Tun {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Write for Tun {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
