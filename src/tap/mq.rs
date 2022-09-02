use crate::{dev, error, iface};

/// A blocking multiqueue TAP interface.
#[cfg_attr(docsrs, doc(cfg(feature = "mq")))]
pub struct MQTap(dev::MQDev);

impl std::ops::Deref for MQTap {
    type Target = dev::MQDev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MQTap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MQTap {
    fn new(iface_params: iface::InterfaceParams) -> error::Result<Vec<MQTap>> {
        let devs = dev::MQDev::new(iface_params)?;

        let tuns = devs.into_iter().map(MQTap).collect();

        Ok(tuns)
    }

    /// Creates a blocking multiqueue TAP interface without the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    /// * `len`: Number of devices in the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: Containing the TAP device if successful.
    /// * `Err`: If `len` is 0 or the interface creation fails.
    pub fn without_packet_info(name: &str, len: usize) -> error::Result<Vec<MQTap>> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: len,
            non_blocking: false,
            no_packet_info: true,
        })
    }

    /// Creates a blocking multiqueue TAP interface with the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    /// * `len`: Number of devices in the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: Containing the TAP device if successful.
    /// * `Err`: If `len` is 0 or the interface creation fails.
    pub fn with_packet_info(name: &str, len: usize) -> error::Result<Vec<MQTap>> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: len,
            non_blocking: false,
            no_packet_info: false,
        })
    }
}
