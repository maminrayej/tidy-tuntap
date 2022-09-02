#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_tun;
#[cfg(feature = "tokio")]
pub use async_tun::*;

use crate::{dev, error, iface};

/// A blocking TUN interface.
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
    /// Creates a blocking TUN interface without the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TUN device if successful.
    /// * `Err`: Otherwise.
    pub fn without_packet_info(name: &str) -> error::Result<Tun> {
        Ok(Tun(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: true,
        })?))
    }

    /// Creates a blocking TUN interface with the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TUN device if successful.
    /// * `Err`: Otherwise.
    pub fn with_packet_info(name: &str) -> error::Result<Tun> {
        Ok(Tun(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: false,
        })?))
    }
}
