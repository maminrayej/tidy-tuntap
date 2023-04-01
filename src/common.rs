use std::fs;
use std::os::unix::prelude::*;
use std::sync::Arc;

use nix::sys::socket;

use crate::error::Result;
use crate::{bindings, ioctl};

/// Represents the mode of device.
pub enum Mode {
    Tun,
    Tap,
}

pub fn create_device(
    name: impl AsRef<str>,
    mode: Mode,
    device_count: usize,
    packet_info: bool,
    non_blocking: bool,
) -> Result<(Arc<[i8; 16]>, Vec<fs::File>, Arc<OwnedFd>, Arc<OwnedFd>)> {
    let mut flags = match mode {
        Mode::Tun => nix::libc::IFF_TUN,
        Mode::Tap => nix::libc::IFF_TAP,
    };

    if !packet_info {
        flags |= nix::libc::IFF_NO_PI;
    }

    if device_count > 1 {
        flags |= nix::libc::IFF_MULTI_QUEUE;
    }

    let non_blocking_flag = if non_blocking {
        nix::libc::O_NONBLOCK
    } else {
        0
    };

    // Convert the device name into a struct that the kernel expects.
    //
    // Kernel uses a constant called IFNAMSIZ with the value of 16 to
    // indicate the maximum number of characters the device name can have.
    // I don't know this string must be null terminated or not. So to be safe,
    // I truncate the first 15 characters of the `name` provided` by the user,
    // and copy it to the name array (which is null terminated because
    // it is initialized by zeros).
    //
    // Source: The IFNAMSIZ is defined in the `linux/if.h`.
    let mut ifr_name = [0i8; 16];
    for (i, c) in name.as_ref().as_bytes().iter().enumerate().take(15) {
        ifr_name[i] = *c as i8;
    }

    // Construct the request with the computed flags and name.
    let mut ifr: bindings::ifreq = unsafe { std::mem::zeroed() };
    ifr.ifr_ifru.ifru_flags = flags as i16;
    ifr.ifr_ifrn.ifrn_name = ifr_name;

    let mut files = Vec::with_capacity(device_count);
    for _ in 0..device_count {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(non_blocking_flag)
            .open("/dev/net/tun")?;

        // Call the ioctl to set the flags and name of the device.
        unsafe { ioctl::tunsetiff(file.as_raw_fd(), &ifr as *const bindings::ifreq as u64)? };

        files.push(file);
    }

    // Get the name chosen by the kernel.
    let name = unsafe { ifr.ifr_ifrn.ifrn_name };

    // Create the weird UDP socket. For explanation go to the documentation
    // of the socket field of the Interface struct.
    let inet4_socket = unsafe {
        OwnedFd::from_raw_fd(socket::socket(
            socket::AddressFamily::Inet,
            socket::SockType::Datagram,
            socket::SockFlag::empty(),
            None,
        )?)
    };

    let inet6_socket = unsafe {
        OwnedFd::from_raw_fd(socket::socket(
            socket::AddressFamily::Inet6,
            socket::SockType::Datagram,
            socket::SockFlag::empty(),
            None,
        )?)
    };

    Ok((
        Arc::new(name),
        files,
        Arc::new(inet4_socket),
        Arc::new(inet6_socket),
    ))
}
