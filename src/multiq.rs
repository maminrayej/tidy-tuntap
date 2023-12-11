use std::marker::PhantomData;
use std::ops;
use std::os::unix::prelude::AsRawFd;

use crate::common::create_device;
use crate::device::Device;
use crate::error::{Error, Result};
use crate::type_state::InterfaceType;
use crate::{bindings, ioctl};

/// Represents a multiqueue TUN/TAP device.
///
/// Contains a multiqueue device.
#[derive(Debug)]
pub struct MQDevice<IfType: InterfaceType>(Device<IfType>);
impl<IfType: InterfaceType> MQDevice<IfType> {
    pub(crate) fn new(
        name: impl AsRef<str>,
        device_count: usize,
        packet_info: bool,
    ) -> Result<Vec<Self>> {
        if device_count == 0 {
            return Err(Error::ZeroDevices);
        }

        let (name, files, inet4_socket, inet6_socket) =
            create_device(name, IfType::MODE, device_count, packet_info, false)?;

        Ok(files
            .into_iter()
            .map(move |file| Device::<IfType> {
                name: name.clone(),
                file,
                inet4_socket: inet4_socket.clone(),
                inet6_socket: inet6_socket.clone(),
                _phantom: PhantomData,
            })
            .map(MQDevice)
            .collect())
    }

    /// Attaches the multiqueue.
    ///
    /// # Returns
    /// * `Ok`: If attaching was succesful.
    /// * `Err`: Otherwise.
    pub fn attach(&self) -> Result<()> {
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
    pub fn detach(&self) -> Result<()> {
        let mut ifreq: bindings::ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifru.ifru_flags = nix::libc::IFF_DETACH_QUEUE as i16;

        unsafe { ioctl::tunsetqueue(self.as_raw_fd(), &ifreq as *const bindings::ifreq as u64)? };

        Ok(())
    }
}
impl<IfType: InterfaceType> ops::Deref for MQDevice<IfType> {
    type Target = Device<IfType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<IfType: InterfaceType> ops::DerefMut for MQDevice<IfType> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
