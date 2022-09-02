#[cfg(feature = "mq")]
mod mq;
#[cfg(feature = "mq")]
pub use mq::*;

#[cfg(feature = "tokio")]
mod async_tap;
#[cfg(feature = "tokio")]
pub use async_tap::*;

use crate::{dev, error, iface};

/// A blocking TAP interface.
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
    /// Creates a blocking TAP interface without the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TAP device if successful.
    /// * `Err`: Otherwise.
    pub fn without_packet_info(name: &str) -> error::Result<Tap> {
        Ok(Tap(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: true,
        })?))
    }

    /// Creates a blocking TAP interface with the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TAP device if successful.
    /// * `Err`: Otherwise.
    pub fn with_packet_info(name: &str) -> error::Result<Tap> {
        Ok(Tap(dev::Dev::from_params(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: 1,
            non_blocking: false,
            no_packet_info: false,
        })?))
    }
}
