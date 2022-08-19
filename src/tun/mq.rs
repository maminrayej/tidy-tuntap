use crate::bindings::*;

use std::os::unix::prelude::AsRawFd;
use std::{ops, sync};

use crate::error::Result;
use crate::iface;
use crate::ioctl;
use crate::tun::Tun;

pub struct MQTun(Tun);

impl ops::Deref for MQTun {
    type Target = Tun;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MQTun {
    fn new(iface_params: iface::InterfaceParams) -> Result<Vec<MQTun>> {
        let iface = sync::Arc::new(iface::Interface::new(iface_params)?);

        let tuns: Result<Vec<MQTun>> = (0..iface.files.len())
            .map(|fd_index| Tun::new(iface.clone(), fd_index).map(MQTun))
            .collect();

        tuns
    }

    pub fn without_packet_info(name: &str, len: usize) -> Result<Vec<MQTun>> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: len,
            non_blocking: false,
            no_packet_info: true,
        })
    }

    pub fn with_packet_info(name: &str, len: usize) -> Result<Vec<MQTun>> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: len,
            non_blocking: false,
            no_packet_info: false,
        })
    }

    pub fn attach(&self) -> Result<()> {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifru.ifru_flags = nix::libc::IFF_ATTACH_QUEUE as i16;

        unsafe { ioctl::tunsetqueue(self.as_raw_fd(), &ifreq as *const ifreq as u64)? };

        Ok(())
    }

    pub fn detach(&self) -> Result<()> {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifru.ifru_flags = nix::libc::IFF_DETACH_QUEUE as i16;

        unsafe { ioctl::tunsetqueue(self.as_raw_fd(), &ifreq as *const ifreq as u64)? };

        Ok(())
    }
}
