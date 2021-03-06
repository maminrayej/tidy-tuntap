use crate::bindings::*;
use crate::flags::Flags;
use crate::ioctl;
use crate::sockaddr::to_ipv4;
use crate::sockaddr::to_sockaddr_in;

use std::io;
use std::net;
use std::os::unix::prelude::{AsRawFd, RawFd};

/// Indicates whether to create a TUN device or a TAP device.
pub enum Mode {
    Tun,
    Tap,
}

impl Mode {
    fn into_flags(self) -> i32 {
        match self {
            Mode::Tun => nix::libc::IFF_TUN,
            Mode::Tap => nix::libc::IFF_TAP,
        }
    }
}

// Indicate what operation should be done on the active flags of the device.
enum Op {
    Add,
    Del,
}

// NOTE: To the dear reader and also the future me who is going to comeback to this code saying what
// the hell is this ifreq?!. Familiarize yourself with this struct by looking at the `linux/if.h` header file.
// We are going to work with the `ifreq` struct quite often in the impl blocks of this struct.
/// A struct representing a TUN/TAP device on a Linux system.
pub struct Interface {
    // Name returned by the kernel.
    name_raw: [i8; 16],

    // This field is for convenience to prevent allocation everytime the user
    // wants to get the name of the device as a string.
    name: String,

    // Wrapper around the raw file descriptor returned by the kernel when the device is created.
    // This field automatically closes the device when this struct gets dropped.
    file: std::fs::File,

    // Raw file descriptor representing a UDP socket.
    //
    // The need for this field is a little bit weird. So the issue is that we cannot
    // request some of the ioctls on the TUN/TAP device itself (which is represented by the file
    // descriptor inside the `file` field). So we have to create an ifreq with the name of the
    // device but, call the ioctl on a UDP socket. The discussion I've found around it suggests
    // it's a legacy thing: https://vtun-devel.narkive.com/igeeWwFF/bringing-up-a-tun-device
    //
    // NOTE: This socket must be manually closed when this struct gets dropped.
    socket: RawFd,
}

impl Interface {
    /// Creates a TUN/TAP device depending on the specified `mode` that will receive packet info with
    /// every packet it gets. The packet info contains data about the `flags` and the `proto` of
    /// the received packet. So the received packet will have the following format: \
    /// * `Flags` [2 bytes]
    /// * `Proto` [2 bytes]
    /// * `Raw protocol` (IP, IPv6, etc) frame.
    ///
    /// # Arguments
    /// * `name`: Name of the TUN/TAP device. Note that the actual name is chosen by the kernel and
    /// may be different from what you've passed to this function. To check if the kernel respected
    /// the name chosen by  you, call the [name](`Interface::name`) function. Also, the name must
    /// at most have 15 characters.
    /// * `mode`: Indicates what type of device should be created.
    ///
    /// # Returns
    /// * `Ok`: Containing the `Interface`.
    /// * `Err`: Can have various reasons like not being able to access the `/dev/net/tun` file, or
    /// failing to calling the ioctls.
    pub fn with_packet_info(name: &str, mode: Mode) -> io::Result<Self> {
        Self::new(name, mode, false)
    }

    /// Creates a TUN/TAP device depending on the specified `mode` that will not receive packet info with
    /// every packet it gets. For more info about packet info look at
    /// [with_packet_info](`Interface::with_packet_info`) function.
    ///
    /// # Arguments
    /// * `name`: Name of the TUN/TAP device. Note that the actual name is chosen by the kernel and
    /// may be different from what you've passed to this function. To check if the kernel respected
    /// the name chosen by  you, call the [name](`Interface::name`) function. Also, the name must
    /// at most have 15 characters.
    /// * `mode`: Indicates what type of device should be created.
    ///
    /// # Returns
    /// * `Ok`: Containing the `Interface`.
    /// * `Err`: Can have various reasons like not being able to access the `/dev/net/tun` file, or
    /// failing to calling the ioctls.
    pub fn without_packet_info(name: &str, mode: Mode) -> io::Result<Self> {
        Self::new(name, mode, true)
    }

    // Internal function that actually tries to create the TUN/TAP device.
    fn new(name: &str, mode: Mode, no_packet_info: bool) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        // Compute the flags.
        let mut flags = mode.into_flags();
        if no_packet_info {
            flags |= nix::libc::IFF_NO_PI;
        }

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
        for (i, c) in name.as_bytes().iter().enumerate().take(15) {
            ifr_name[i] = *c as i8;
        }

        // Construct the request with the computed flags and name.
        let mut ifr: ifreq = unsafe { std::mem::zeroed() };
        ifr.ifr_ifru.ifru_flags = flags as i16;
        ifr.ifr_ifrn.ifrn_name = ifr_name;

        // Call the ioctl to set the flags and name of the device.
        unsafe { ioctl::tunsetiff(file.as_raw_fd(), &ifr as *const ifreq as u64)? };

        // Get the name chosen by the kernel.
        let name_raw = unsafe { ifr.ifr_ifrn.ifrn_name };
        let name = String::from_iter(name_raw.iter().map_while(|c| {
            let c = *c as u8 as char;

            if c != '\0' {
                Some(c)
            } else {
                None
            }
        }));

        // Create the weird UDP socket. For explanation go to the documentation
        // of the socket field of the Interface struct.
        let socket = nix::sys::socket::socket(
            nix::sys::socket::AddressFamily::Inet,
            nix::sys::socket::SockType::Datagram,
            nix::sys::socket::SockFlag::empty(),
            None,
        )?;

        Ok(Interface {
            name_raw,
            name,
            file,
            socket,
        })
    }

    /// # Returns
    /// The name of the device chosen by the kernel.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// # Returns
    /// * `Ok`: Containing the active flags of the device.
    /// * `Err`: If the ioctl failed.
    pub fn flags(&self) -> io::Result<Flags> {
        self.read_flags()?.try_into()
    }

    /// Brings the device up meaning makes it ready to send and receive packets.
    ///
    /// # Returns
    /// * `Ok`: If the device was successfully brought up.
    /// * `Err`: If the ioctl failed.
    pub fn bring_up(&self) -> io::Result<()> {
        self.mod_flags(Op::Add, nix::libc::IFF_UP | nix::libc::IFF_RUNNING)
    }

    /// Brings the device down meaning makes it unable to send and receive packets.
    ///
    /// # Returns
    /// * `Ok`: If the device was successfully brought down.
    /// * `Err`: If the ioctl failed.
    pub fn bring_down(&self) -> io::Result<()> {
        self.mod_flags(Op::Del, nix::libc::IFF_UP | nix::libc::IFF_RUNNING)
    }

    /// Sets the MTU of the device.
    ///
    /// # Arguments
    /// * `mtu`: New MTU of the device.
    ///
    /// # Returns
    /// * `Ok`: If the MTU of the device has been successfully changed to `mtu`.
    /// * `Err`: If the ioctl failed.
    pub fn set_mtu(&self, mtu: i32) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_mtu = mtu;

        unsafe { ioctl::siocsifmtu(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    /// Gets the MTU of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the MTU of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_mtu(&self) -> io::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifmtu(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(unsafe { ifreq.ifr_ifru.ifru_mtu })
    }

    /// Sets the netmask of the device.
    ///
    /// # Arguments
    /// * `netmask`: New netmask of the device.
    ///
    /// # Returns
    /// * `Ok`: If the netmask of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_netmask(&self, netmask: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_netmask = unsafe { std::mem::transmute(to_sockaddr_in(netmask)) };

        unsafe { ioctl::siocsifnetmask(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    /// Gets the netmask of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the netmask of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_netmask(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifnetmask(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_netmask)
        }))
    }

    /// Sets the IPv4 address of the device.
    ///
    /// # Arguments
    /// * `addr`: New IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the IPv4 address of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_addr(&self, addr: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_addr = unsafe { std::mem::transmute(to_sockaddr_in(addr)) };

        unsafe { ioctl::siocsifaddr(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    /// Gets the IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the IPv4 address of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_addr(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifaddr(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_addr)
        }))
    }

    /// Sets the broadcast IPv4 address of the device.
    ///
    /// # Arguments
    /// * `brd_addr`: New broadcast address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the broadcast IPv4 address of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_brd_addr(&self, brd_addr: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_dstaddr = unsafe { std::mem::transmute(to_sockaddr_in(brd_addr)) };

        unsafe { ioctl::siocsifbrdaddr(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    /// Gets the broadcast IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the broadcast IPv4 address of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_brd_addr(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifbrdaddr(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_dstaddr)
        }))
    }

    /// Sets the destination IPv4 address of the device.
    ///
    /// # Arguments
    /// * `dst_addr`: New destination IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the destination IPv4 address of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_dst_addr(&self, dst_addr: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_dstaddr = unsafe { std::mem::transmute(to_sockaddr_in(dst_addr)) };

        unsafe { ioctl::siocsifdstaddr(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    /// Gets the destination IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the destination IPv4 address of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_dst_addr(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifdstaddr(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_dstaddr)
        }))
    }

    /// Sets the owner of the device.
    ///
    /// # Arguments
    /// * `owner`: User id of the new owner of this device.
    ///
    /// # Returns
    /// * `Ok`: If changing the ownership was successful.
    /// * `Err`: If the ioctl failed.
    pub fn set_owner(&self, owner: i32) -> io::Result<()> {
        unsafe { ioctl::tunsetowner(self.file.as_raw_fd(), owner as u64)? };

        Ok(())
    }

    /// Sets the group that the device belongs to.
    ///
    /// # Arguments
    /// * `group`: User id of the new owner of this device.
    ///
    /// # Returns
    /// * `Ok`: If changing the group was successful.
    /// * `Err`: If the ioctl failed.
    pub fn set_group(&self, group: i32) -> io::Result<()> {
        unsafe { ioctl::tunsetgroup(self.file.as_raw_fd(), group as u64)? };

        Ok(())
    }

    /// Can be used to make the TUN/TAP interface persistent. In this mode,
    /// the interface won't be destroyed when the last process closes the associated `/dev/net/tun` file descriptor.
    ///
    /// # Returns
    /// * `Ok`: If the device changed to be persistent.
    /// * `Err`: If the ioctl failed.
    pub fn persist(&self, persist: bool) -> io::Result<()> {
        unsafe { ioctl::tunsetpersist(self.file.as_raw_fd(), if persist { 1 } else { 0 })? };

        Ok(())
    }

    // Returns an empty ifreq with the same name of this device.
    fn new_ifreq(&self) -> ifreq {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifrn.ifrn_name = self.name_raw;

        ifreq
    }

    // Returns the active flags of the device.
    fn read_flags(&self) -> io::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifflags(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(unsafe { ifreq.ifr_ifru.ifru_flags.into() })
    }

    // Modifies the active flags of the device based the requested operation specified by `op`.
    fn mod_flags(&self, op: Op, new_flags: i32) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        // We first read the active flags.
        ifreq.ifr_ifru.ifru_flags = self.read_flags()? as i16;

        // Apply the requested modifications.
        unsafe {
            match op {
                // Consider having the bits: 1110 meaning second, third and forth flags are active.
                // If we want to active the first flag, we could OR 0001 with 1110 and get 1111.
                Op::Add => ifreq.ifr_ifru.ifru_flags |= new_flags as i16,

                // Consider having the bits: 1110 meaning second, third and forth flags are active.
                // If we want to deactive the second flag, we could first NOT the 0010 to get 1101.
                // Now if we AND 1101 with 1110, we get 1100.
                Op::Del => ifreq.ifr_ifru.ifru_flags &= !(new_flags) as i16,
            }

            // Then finally set the updated flags.
            ioctl::siocsifflags(self.socket, &ifreq as *const ifreq)?;
        }

        Ok(())
    }
}

impl io::Read for Interface {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl io::Write for Interface {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        nix::unistd::close(self.socket).expect("Failed to close the socket");
    }
}
