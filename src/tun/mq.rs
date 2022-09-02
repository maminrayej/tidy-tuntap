use crate::{dev, error, iface};

/// A blocking multiqueue TUN interface.
#[cfg_attr(docsrs, doc(cfg(feature = "mq")))]
pub struct MQTun(dev::MQDev);

impl std::ops::Deref for MQTun {
    type Target = dev::MQDev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MQTun {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MQTun {
    fn new(iface_params: iface::InterfaceParams) -> error::Result<Vec<MQTun>> {
        let devs = dev::MQDev::new(iface_params)?;

        let tuns = devs.into_iter().map(MQTun).collect();

        Ok(tuns)
    }

    /// Creates a blocking multiqueue TUN interface without the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    /// * `len`: Number of devices in the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: Containing the TUN device if successful.
    /// * `Err`: If `len` is 0 or the interface creation fails.
    pub fn without_packet_info(name: &str, len: usize) -> error::Result<Vec<MQTun>> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: len,
            non_blocking: false,
            no_packet_info: true,
        })
    }

    /// Creates a blocking multiqueue TUN interface with the packet info with the specified `name`.
    ///
    /// # Arguments
    /// * `name`: Suggested name of the interface.
    /// * `len`: Number of devices in the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: Containing the TUN device if successful.
    /// * `Err`: If `len` is 0 or the interface creation fails.
    pub fn with_packet_info(name: &str, len: usize) -> error::Result<Vec<MQTun>> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: len,
            non_blocking: false,
            no_packet_info: false,
        })
    }
}
