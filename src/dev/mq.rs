use std::os::unix::prelude::AsRawFd;
use std::{ops, sync};

use crate::{bindings, dev, error, iface, ioctl};

/// Representing a blocking multiqueue TUN/TAP device.
#[cfg_attr(docsrs, doc(cfg(feature = "mq")))]
pub struct MQDev(dev::Dev);

impl ops::Deref for MQDev {
    type Target = dev::Dev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for MQDev {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MQDev {
    pub(crate) fn new(iface_params: iface::InterfaceParams) -> error::Result<Vec<MQDev>> {
        if iface_params.fd_count == 0 {
            return Err(error::Error::ZeroLenMultiQueue);
        }

        let iface = sync::Arc::new(iface::Interface::new(iface_params)?);

        let devs: error::Result<Vec<MQDev>> = (0..iface.files.len())
            .map(|fd_index| dev::Dev::new(iface.clone(), fd_index).map(MQDev))
            .collect();

        devs
    }

    /// Attaches the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: If attaching was succesful.
    /// * `Err`: Otherwise.
    pub fn attach(&self) -> error::Result<()> {
        let mut ifreq: bindings::ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifru.ifru_flags = nix::libc::IFF_ATTACH_QUEUE as i16;

        unsafe { ioctl::tunsetqueue(self.as_raw_fd(), &ifreq as *const bindings::ifreq as u64)? };

        Ok(())
    }

    /// Detaches the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: If detaching was succesful.
    /// * `Err`: Otherwise.
    pub fn detach(&self) -> error::Result<()> {
        let mut ifreq: bindings::ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifru.ifru_flags = nix::libc::IFF_DETACH_QUEUE as i16;

        unsafe { ioctl::tunsetqueue(self.as_raw_fd(), &ifreq as *const bindings::ifreq as u64)? };

        Ok(())
    }
}
