mod iface_param;

use std::net;
use std::os::unix::{
    io::OwnedFd,
    prelude::{AsRawFd, FromRawFd, OpenOptionsExt},
};

use std::str::FromStr;

use crate::{bindings, error, flags, ioctl, sockaddr};

pub(crate) use iface_param::*;

/// Indicates the type of the interface.
pub enum Mode {
    Tun,
    Tap,
}

impl Mode {
    fn into_flag(self) -> i32 {
        match self {
            Mode::Tun => nix::libc::IFF_TUN,
            Mode::Tap => nix::libc::IFF_TAP,
        }
    }
}

// Indicate what operation should be done on the active flags of the interface.
enum Op {
    Add,
    Del,
}

/// A struct representing a TUN/TAP device on a Linux system.
pub struct Interface {
    // Name returned by the kernel.
    name_raw: [i8; 16],

    pub(crate) files: Vec<std::fs::File>,

    // Raw file descriptor representing a UDP socket.
    //
    // The need for this field is a little bit weird. So the issue is that we cannot
    // request some of the ioctls on the TUN/TAP device itself (which is represented by the file
    // descriptor inside the `file` field). So we have to create an ifreq with the name of the
    // device but, call the ioctl on a UDP socket. The discussion I've found around it suggests
    // it's a legacy thing: https://vtun-devel.narkive.com/igeeWwFF/bringing-up-a-tun-device
    inet4_socket: OwnedFd,

    // Same rationale as `inet4_socket`, but for ioctls related to IPv6 addressing.
    inet6_socket: OwnedFd,
}

impl Interface {
    pub(crate) fn new(params: iface_param::InterfaceParams) -> error::Result<Self> {
        let mut flags = params.mode.into_flag();
        if params.no_packet_info {
            flags |= nix::libc::IFF_NO_PI;
        }

        if params.fd_count > 1 {
            flags |= nix::libc::IFF_MULTI_QUEUE;
        }

        let non_blocking_flag = if params.non_blocking {
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
        for (i, c) in params.name.as_bytes().iter().enumerate().take(15) {
            ifr_name[i] = *c as i8;
        }

        // Construct the request with the computed flags and name.
        let mut ifr: bindings::ifreq = unsafe { std::mem::zeroed() };
        ifr.ifr_ifru.ifru_flags = flags as i16;
        ifr.ifr_ifrn.ifrn_name = ifr_name;

        let mut files = Vec::with_capacity(params.fd_count);
        for _ in 0..params.fd_count {
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
        let name_raw = unsafe { ifr.ifr_ifrn.ifrn_name };

        // Create the weird UDP socket. For explanation go to the documentation
        // of the socket field of the Interface struct.
        let inet4_socket = unsafe {
            OwnedFd::from_raw_fd(nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Inet,
                nix::sys::socket::SockType::Datagram,
                nix::sys::socket::SockFlag::empty(),
                None,
            )?)
        };

        let inet6_socket = unsafe {
            OwnedFd::from_raw_fd(nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Inet6,
                nix::sys::socket::SockType::Datagram,
                nix::sys::socket::SockFlag::empty(),
                None,
            )?)
        };

        Ok(Interface {
            name_raw,
            files,
            inet4_socket,
            inet6_socket,
        })
    }

    /// # Returns
    /// The name of the device chosen by the kernel.
    #[rustfmt::skip]
    pub fn name(&self) -> String {
        String::from_iter(self.name_raw.iter().map_while(|c| {
            let c = *c as u8 as char;

            if c != '\0' { Some(c) } else { None }
        }))
    }

    /// # Returns
    /// * `Ok`: Containing the active flags of the interface.
    /// * `Err`: If the ioctl failed.
    pub fn flags(&self) -> error::Result<flags::Flags> {
        self.read_flags()?.try_into()
    }

    /// Brings the device up meaning makes it ready to send and receive packets.
    ///
    /// # Returns
    /// * `Ok`: If the device was successfully brought up.
    /// * `Err`: If the ioctl failed.
    pub fn bring_up(&self) -> error::Result<()> {
        self.mod_flags(Op::Add, nix::libc::IFF_UP | nix::libc::IFF_RUNNING)
    }

    /// Brings the device down meaning makes it unable to send and receive packets.
    ///
    /// # Returns
    /// * `Ok`: If the device was successfully brought down.
    /// * `Err`: If the ioctl failed.
    pub fn bring_down(&self) -> error::Result<()> {
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
    pub fn set_mtu(&self, mtu: i32) -> error::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_mtu = mtu;

        unsafe {
            ioctl::siocsifmtu(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Gets the MTU of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the MTU of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_mtu(&self) -> error::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifmtu(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety: Since we issued an ioctl for getting the MTU, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_mtu` variant.
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
    pub fn set_netmask(&self, netmask: net::Ipv4Addr) -> error::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_netmask = sockaddr::to_sockaddr(netmask);

        unsafe {
            ioctl::siocsifnetmask(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Gets the netmask of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the netmask of the interface.
    /// * `Err`: If the ioctl failed.
    pub fn get_netmask(&self) -> error::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifnetmask(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety: Since we issued an ioctl for getting the netmask, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_netmask` variant.
        Ok(sockaddr::to_ipv4(unsafe { ifreq.ifr_ifru.ifru_netmask }))
    }

    /// Gets the index of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the index of the interface.
    /// * `Err`: If the ioctl failed.
    pub fn get_index(&self) -> error::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifindex(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety: Since we issued an ioctl for getting the index, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_ivalue` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_ivalue })
    }

    /// Adds the specified `addr` to the list of IPv6 addresses of the interface.
    ///
    /// # Arguments
    /// * `addr`: New IPv6 address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the specified IPv6 address has been successfully added.
    /// * `Err`: If the ioctl failed.
    pub fn set_ipv6_addr(&self, addr: net::Ipv6Addr) -> error::Result<()> {
        let ifindex = self.get_index()?;

        #[rustfmt::skip]
        let in6_ifreq = bindings::in6_ifreq {
            ifr6_addr: nix::libc::in6_addr { s6_addr: addr.octets() },
            ifr6_prefixlen: 64,
            ifr6_ifindex: ifindex,
        };

        unsafe {
            ioctl::siocsifaddr6(
                self.inet6_socket.as_raw_fd(),
                &in6_ifreq as *const bindings::in6_ifreq,
            )?;
        }

        Ok(())
    }

    /// Gets the list of IPv6 addresses of the interface.
    ///
    /// # Returns
    /// * `Ok`: Containing the IPv6 addresses of the interface.
    /// * `Err`: If the ioctl failed.
    pub fn get_ipv6_addrs(&self) -> error::Result<Vec<net::Ipv6Addr>> {
        // `getifaddrs` returns all addresses of all interfaces in the system.
        Ok(nix::ifaddrs::getifaddrs()?
            // We are only interested in the addresses related to our interface.
            .filter(|iface| iface.interface_name == self.name())
            // Among the addresses related to our interface, we want the IPv6 ones.
            .filter_map(|iface| {
                iface
                    .address
                    .and_then(|addr| addr.as_sockaddr_in6().map(|in6_addr| in6_addr.ip()))
                //                   ----------------------                -------------
                //                     try to convert the                   extract the
                //                     address to IPv6                      ip from IPv6
            })
            .collect())
    }

    /// Deletes the specified IPv6 address from the interface.
    ///
    /// # Arguments
    /// * `addr`: IPv6 address to be removed from the interface.
    ///
    /// # Returns
    /// * `Ok`: If the specified IPv6 address was removed.
    /// * `Err`: If the ioctl failed.
    pub fn del_ipv6_addr(&self, addr: net::Ipv6Addr) -> error::Result<()> {
        let ifindex = self.get_index()?;

        #[rustfmt::skip]
        let in6_ifreq = bindings::in6_ifreq {
            ifr6_addr: nix::libc::in6_addr { s6_addr: addr.octets() },
            ifr6_prefixlen: 64,
            ifr6_ifindex: ifindex,
        };

        unsafe {
            ioctl::siocdifaddr6(
                self.inet6_socket.as_raw_fd(),
                &in6_ifreq as *const bindings::in6_ifreq,
            )?;
        }

        Ok(())
    }

    /// Sets the IPv4 address of the device.
    ///
    /// # Arguments
    /// * `addr`: New IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the IPv4 address of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_addr(&self, addr: net::Ipv4Addr) -> error::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_addr = sockaddr::to_sockaddr(addr);

        unsafe {
            ioctl::siocsifaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Gets the IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the IPv4 address of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_addr(&self) -> error::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety: Since we issued a ioctl for getting the netmask, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_netmask` variant.
        Ok(sockaddr::to_ipv4(unsafe { ifreq.ifr_ifru.ifru_addr }))
    }

    /// Deletes the IPv4 address of the interface.
    ///
    /// # Returns
    /// * `Ok`: If the IPv4 address was removed.
    /// * `Err`: If the ioctl failed.
    pub fn del_addr(&self) -> error::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_addr =
            sockaddr::to_sockaddr(net::Ipv4Addr::from_str("0.0.0.0").unwrap());

        unsafe {
            ioctl::siocsifaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Sets the broadcast IPv4 address of the device.
    ///
    /// # Arguments
    /// * `brd_addr`: New broadcast address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the broadcast IPv4 address of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_brd_addr(&self, brd_addr: net::Ipv4Addr) -> error::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_broadaddr = sockaddr::to_sockaddr(brd_addr);

        unsafe {
            ioctl::siocsifbrdaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Gets the broadcast IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the broadcast IPv4 address of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_brd_addr(&self) -> error::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifbrdaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety: Since we issued a ioctl for getting the broadcast address, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_broadaddr` variant.
        Ok(sockaddr::to_ipv4(unsafe { ifreq.ifr_ifru.ifru_broadaddr }))
    }

    /// Sets the destination IPv4 address of the device.
    ///
    /// # Arguments
    /// * `dst_addr`: New destination IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: If the destination IPv4 address of the device has been successfully changed.
    /// * `Err`: If the ioctl failed.
    pub fn set_dst_addr(&self, dst_addr: net::Ipv4Addr) -> error::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_dstaddr = sockaddr::to_sockaddr(dst_addr);

        unsafe {
            ioctl::siocsifdstaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Gets the destination IPv4 address of the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the destination IPv4 address of the device.
    /// * `Err`: If the ioctl failed.
    pub fn get_dst_addr(&self) -> error::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifdstaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety: Since we issued a ioctl for getting the destination address, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_dstaddr` variant.
        Ok(sockaddr::to_ipv4(unsafe { ifreq.ifr_ifru.ifru_dstaddr }))
    }

    //    /// Sets the owner of the device.
    //    ///
    //    /// # Arguments
    //    /// * `owner`: User id of the new owner of this device.
    //    ///
    //    /// # Returns
    //    /// * `Ok`: If changing the ownership was successful.
    //    /// * `Err`: If the ioctl failed.
    //    pub fn set_owner(&self, owner: i32) -> error::Result<()> {
    //        unsafe { ioctl::tunsetowner(self.file.as_raw_fd(), owner as u64)? };
    //
    //        Ok(())
    //    }
    //
    //    /// Sets the group that the device belongs to.
    //    ///
    //    /// # Arguments
    //    /// * `group`: User id of the new owner of this device.
    //    ///
    //    /// # Returns
    //    /// * `Ok`: If changing the group was successful.
    //    /// * `Err`: If the ioctl failed.
    //    pub fn set_group(&self, group: i32) -> error::Result<()> {
    //        unsafe { ioctl::tunsetgroup(self.file.as_raw_fd(), group as u64)? };
    //
    //        Ok(())
    //    }
    //
    //    /// Can be used to make the TUN/TAP interface persistent. In this mode,
    //    /// the interface won't be destroyed when the last process closes the associated `/dev/net/tun` file descriptor.
    //    ///
    //    /// # Returns
    //    /// * `Ok`: If the device changed to be persistent.
    //    /// * `Err`: If the ioctl failed.
    //    pub fn persist(&self, persist: bool) -> error::Result<()> {
    //        unsafe { ioctl::tunsetpersist(self.file.as_raw_fd(), if persist { 1 } else { 0 })? };
    //
    //        Ok(())
    //    }

    // Returns an empty ifreq with the same name of this device.
    pub(crate) fn new_ifreq(&self) -> bindings::ifreq {
        let mut ifreq: bindings::ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifrn.ifrn_name = self.name_raw;

        ifreq
    }

    // Returns the active flags of the device.
    fn read_flags(&self) -> error::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifflags(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        Ok(unsafe { ifreq.ifr_ifru.ifru_flags.into() })
    }

    // Modifies the active flags of the device based on the requested operation specified by `op`.
    fn mod_flags(&self, op: Op, new_flags: i32) -> error::Result<()> {
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
            ioctl::siocsifflags(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?;
        }

        Ok(())
    }
}
