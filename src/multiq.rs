use std::ops;
use std::os::unix::prelude::AsRawFd;

use crate::common::create_device;
use crate::device::Device;
use crate::error::{Error, Result};
use crate::{bindings, ioctl, Mode};

/// Represents a multiqueue TUN/TAP device.
///
/// Contains the shared code between [`MQTun`](crate::MQTun) and [`MQTap`](crate::MQTap).
#[derive(Debug)]
pub struct MQDevice(Device);
impl MQDevice {
    fn new(
        name: impl AsRef<str>,
        mode: Mode,
        device_count: usize,
        packet_info: bool,
    ) -> Result<impl Iterator<Item = Self>> {
        if device_count == 0 {
            return Err(Error::ZeroDevices);
        }

        let (name, files, inet4_socket, inet6_socket) =
            create_device(name, mode, device_count, packet_info, false)?;

        Ok(files
            .into_iter()
            .map(move |file| Device {
                name: name.clone(),
                file,
                inet4_socket: inet4_socket.clone(),
                inet6_socket: inet6_socket.clone(),
            })
            .map(MQDevice))
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
impl ops::Deref for MQDevice {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for MQDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents a multiqueue TUN device.
#[derive(Debug)]
pub struct MQTun(MQDevice);
impl MQTun {
    pub fn new(name: impl AsRef<str>, device_count: usize, packet_info: bool) -> Result<Vec<Self>> {
        let devices = MQDevice::new(name, Mode::Tun, device_count, packet_info)?;

        Ok(devices.map(MQTun).collect())
    }
}
impl ops::Deref for MQTun {
    type Target = MQDevice;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for MQTun {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents a mutliqueue TAP device.
#[derive(Debug)]
pub struct MQTap(MQDevice);
impl MQTap {
    pub fn new(name: impl AsRef<str>, device_count: usize, packet_info: bool) -> Result<Vec<Self>> {
        let devices = MQDevice::new(name, Mode::Tap, device_count, packet_info)?;

        Ok(devices.map(MQTap).collect())
    }
}
impl ops::Deref for MQTap {
    type Target = MQDevice;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for MQTap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
