use std::os::unix::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use std::{fs, io, net, ops};

use crate::common::create_device;
use crate::error::Result;
use crate::flags::Flags;
use crate::{bindings, ioctl, sockaddr, Mode};

// Determines what operation should be done one the active flags of the device.
enum Op {
    Add,
    Del,
}

/// Represents a blocking TUN/TAP device.
/// 
/// Contains the shared code between [`Tun`](crate::Tun) and [`Tap`](crate::Tap).
pub struct Device {
    pub(crate) name: Arc<[i8; 16]>,
    pub(crate) file: fs::File,

    pub(crate) inet4_socket: Arc<OwnedFd>,
    pub(crate) inet6_socket: Arc<OwnedFd>,
}

impl Device {
    fn new(name: impl AsRef<str>, mode: Mode, packet_info: bool) -> Result<Self> {
        let (name, mut files, inet4_socket, inet6_socket) =
            create_device(name, mode, 1, packet_info, false)?;

        Ok(Self {
            name,
            file: files.pop().unwrap(),
            inet4_socket,
            inet6_socket,
        })
    }

    /// # Returns
    /// The name of the device chosen by the kernel.
    #[rustfmt::skip]
    pub fn name(&self) -> String {
        String::from_iter(self.name.iter().map_while(|c| {
            let c = *c as u8 as char;

            if c != '\0' { Some(c) } else { None }
        }))
    }

    /// # Returns
    /// * `Ok`: Containing the active flags of the interface.
    /// * `Err`: If the ioctl failed.
    pub fn flags(&self) -> Result<Flags> {
        self.read_flags()?.try_into()
    }

    /// Brings the device up meaning makes it ready to send and receive packets.
    ///
    /// # Returns
    /// * `Ok`: If the device was successfully brought up.
    /// * `Err`: If the ioctl failed.
    pub fn bring_up(&self) -> Result<()> {
        self.mod_flags(Op::Add, nix::libc::IFF_UP | nix::libc::IFF_RUNNING)
    }

    /// Brings the device down meaning makes it unable to send and receive packets.
    ///
    /// # Returns
    /// * `Ok`: If the device was successfully brought down.
    /// * `Err`: If the ioctl failed.
    pub fn bring_down(&self) -> Result<()> {
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
    pub fn set_mtu(&self, mtu: i32) -> Result<()> {
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
    pub fn get_mtu(&self) -> Result<i32> {
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
    pub fn set_netmask(&self, netmask: net::Ipv4Addr) -> Result<()> {
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
    pub fn get_netmask(&self) -> Result<net::Ipv4Addr> {
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
    pub fn get_index(&self) -> Result<i32> {
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
    pub fn set_ipv6_addr(&self, addr: net::Ipv6Addr) -> Result<()> {
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
    pub fn get_ipv6_addrs(&self) -> Result<Vec<net::Ipv6Addr>> {
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
    pub fn del_ipv6_addr(&self, addr: net::Ipv6Addr) -> Result<()> {
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
    pub fn set_addr(&self, addr: net::Ipv4Addr) -> Result<()> {
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
    pub fn get_addr(&self) -> Result<net::Ipv4Addr> {
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
    pub fn del_addr(&self) -> Result<()> {
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
    pub fn set_brd_addr(&self, brd_addr: net::Ipv4Addr) -> Result<()> {
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
    pub fn get_brd_addr(&self) -> Result<net::Ipv4Addr> {
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
    pub fn set_dst_addr(&self, dst_addr: net::Ipv4Addr) -> Result<()> {
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
    pub fn get_dst_addr(&self) -> Result<net::Ipv4Addr> {
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
    //    pub fn set_owner(&self, owner: i32) -> Result<()> {
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
    //    pub fn set_group(&self, group: i32) -> Result<()> {
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
    //    pub fn persist(&self, persist: bool) -> Result<()> {
    //        unsafe { ioctl::tunsetpersist(self.file.as_raw_fd(), if persist { 1 } else { 0 })? };
    //
    //        Ok(())
    //    }

    // Returns an empty ifreq with the same name of this device.
    fn new_ifreq(&self) -> bindings::ifreq {
        let mut ifreq: bindings::ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifrn.ifrn_name = *self.name;

        ifreq
    }

    // Returns the active flags of the device.
    fn read_flags(&self) -> Result<i32> {
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
    fn mod_flags(&self, op: Op, new_flags: i32) -> Result<()> {
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

    /// Blocks and writes the data in `buf` into the device.
    ///
    /// # Arguments
    /// * `buf`: Data to be written into the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes written.
    /// * `Err`: If any error occurs.
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(nix::unistd::write(self.file.as_raw_fd(), buf)?)
    }

    /// Blocks and read the data from device into `buf`.
    ///
    /// # Arguments
    /// * `buf`: Buffer to be filled with data read from the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes read.
    /// * `Err`: If any error occurs.
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        Ok(nix::unistd::read(self.file.as_raw_fd(), buf)?)
    }
}

impl io::Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl io::Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

/// Represents a blocking TUN device.
pub struct Tun(Device);
impl Tun {
    pub fn new(name: impl AsRef<str>, packet_info: bool) -> Result<Self> {
        let device = Device::new(name, Mode::Tun, packet_info)?;

        Ok(Tun(device))
    }
}
impl ops::Deref for Tun {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for Tun {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents a blocking TAP device.
pub struct Tap(Device);
impl Tap {
    pub fn new(name: impl AsRef<str>, packet_info: bool) -> Result<Self> {
        let device = Device::new(name, Mode::Tap, packet_info)?;

        Ok(Tap(device))
    }
}
impl ops::Deref for Tap {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for Tap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
