#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_tun;
#[cfg(feature = "tokio")]
pub use async_tun::*;

use std::sync;

use crate::error::Result;
use crate::{dev, iface};

pub struct Tun(dev::Dev);

impl std::ops::Deref for Tun {
    type Target = dev::Dev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Tun {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
