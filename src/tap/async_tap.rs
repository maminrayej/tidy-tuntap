use crate::{dev, error, iface};

/// A non-blocking TAP interface.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub struct AsyncTap(dev::AsyncDev);

impl std::ops::Deref for AsyncTap {
    type Target = dev::AsyncDev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AsyncTap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsyncTap {
    /// Creates a non-blocking TAP interface without the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TAP interface if successful.
    /// * `Err`: Otherwise.
    pub fn without_packet_info(name: &str) -> error::Result<AsyncTap> {
        Ok(AsyncTap(dev::AsyncDev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tap,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: true,
            },
        )?))
    }

    /// Creates a non-blocking TAP interface with the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the TAP interface if successful.
    /// * `Err`: Otherwise.
    pub fn with_packet_info(name: &str) -> error::Result<AsyncTap> {
        Ok(AsyncTap(dev::AsyncDev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tap,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: false,
            },
        )?))
    }
}
