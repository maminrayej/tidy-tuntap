use crate::{dev, error, iface};

/// A non-blocking TUN interface.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub struct AsyncTun(dev::AsyncDev);

impl std::ops::Deref for AsyncTun {
    type Target = dev::AsyncDev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AsyncTun {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsyncTun {
    /// Creates a non-blocking TUN interface without the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TUN interface if successful.
    /// * `Err`: Otherwise.
    pub fn without_packet_info(name: &str) -> error::Result<AsyncTun> {
        Ok(AsyncTun(dev::AsyncDev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tun,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: true,
            },
        )?))
    }

    /// Creates a non-blocking TUN interface with the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TUN interface if successful.
    /// * `Err`: Otherwise.
    pub fn with_packet_info(name: &str) -> error::Result<AsyncTun> {
        Ok(AsyncTun(dev::AsyncDev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tun,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: false,
            },
        )?))
    }
}
