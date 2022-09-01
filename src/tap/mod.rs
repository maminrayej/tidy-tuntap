#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_tap;
#[cfg(feature = "tokio")]
pub use async_tap::*;

use std::sync;

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

impl std::ops::DerefMut for Tap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
