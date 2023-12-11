use std::marker::PhantomData;
use std::os::unix::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use std::{fs, io, net};

use crate::common::create_device;
use crate::error::Result;
use crate::flags::Flags;
use crate::type_state::{InterfaceType, Tap};
use crate::{bindings, ioctl, sockaddr::sockaddr};

/// Represents a blocking TUN/TAP device.
///
/// Contains a blocking device.
#[derive(Debug)]
pub struct Device<IfType: InterfaceType> {
    pub(crate) name: Arc<[i8; 16]>,
    pub(crate) file: fs::File,

    pub(crate) inet4_socket: Arc<OwnedFd>,
    pub(crate) inet6_socket: Arc<OwnedFd>,
    pub(crate) _phantom: PhantomData<IfType>,
}

impl<IfType: InterfaceType> Device<IfType> {
    pub(crate) fn new(name: impl AsRef<str>, packet_info: bool) -> Result<Self> {
        let (name, mut files, inet4_socket, inet6_socket) =
            create_device(name, IfType::MODE, 1, packet_info, false)?;

        Ok(Self {
            name,
            file: files.pop().unwrap(),
            inet4_socket,
            inet6_socket,
            _phantom: PhantomData,
        })
    }

    /// Returns The name of the device chosen by the kernel.
    #[rustfmt::skip]
    pub fn name(&self) -> String {
        String::from_iter(self.name.iter().map_while(|c| {
            let c = *c as u8 as char;

            if c != '\0' { Some(c) } else { None }
        }))
    }

    /// Returns the active flags of the interface.
    pub fn flags(&self) -> Result<Flags> {
        self.read_flags()?.try_into()
    }

    /// Brings the device up which makes it ready to send and receive packets.
    pub fn bring_up(&self) -> Result<()> {
        self.add_flags(nix::libc::IFF_UP | nix::libc::IFF_RUNNING)
    }

    /// Brings the device down which makes it unable to send and receive packets.
    pub fn bring_down(&self) -> Result<()> {
        self.del_flags(nix::libc::IFF_UP | nix::libc::IFF_RUNNING)
    }

    /// Sets the MTU of the device.
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

    /// Returns the MTU of the device.
    pub fn get_mtu(&self) -> Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifmtu(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued an ioctl for getting the MTU, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_mtu` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_mtu })
    }

    /// Sets the netmask of the device.
    pub fn set_netmask(&self, netmask: net::Ipv4Addr) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_netmask = sockaddr::from(netmask);

        unsafe {
            ioctl::siocsifnetmask(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Sets the netmask of the device.
    pub fn get_netmask(&self) -> Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifnetmask(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued an ioctl for getting the netmask, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_netmask` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_netmask }.into())
    }

    /// Returns the index of the interface.
    pub fn get_index(&self) -> Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifindex(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued an ioctl for getting the index, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_ivalue` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_ivalue })
    }

    /// Adds the specified `addr` to the list of IPv6 addresses of the interface.
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

    /// Returns the list of IPv6 addresses of the interface.
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
                //                     address to Isockaddr::into(IPv6      ip from IPv6
            })
            .collect())
    }

    /// Deletes the specified IPv6 address from the interface.
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
    pub fn set_addr(&self, addr: net::Ipv4Addr) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_addr = sockaddr::from(addr);

        unsafe {
            ioctl::siocsifaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Returns the IPv4 address of the device.
    pub fn get_addr(&self) -> Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued a ioctl for getting the netmask, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_netmask` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_addr }.into())
    }

    /// Deletes the IPv4 address of the interface.
    pub fn del_addr(&self) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_addr = sockaddr::from(net::Ipv4Addr::from_str("0.0.0.0").unwrap());

        unsafe {
            ioctl::siocsifaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Sets the broadcast IPv4 address of the device.
    pub fn set_brd_addr(&self, addr: net::Ipv4Addr) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_broadaddr = sockaddr::from(addr);

        unsafe {
            ioctl::siocsifbrdaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Returns the broadcast IPv4 address of the device.
    pub fn get_brd_addr(&self) -> Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifbrdaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued a ioctl for getting the broadcast address, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_broadaddr` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_broadaddr }.into())
    }

    /// Sets the destination IPv4 address of the device.
    pub fn set_dst_addr(&self, addr: net::Ipv4Addr) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_dstaddr = sockaddr::from(addr);

        unsafe {
            ioctl::siocsifdstaddr(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Returns the destination IPv4 address of the device.
    pub fn get_dst_addr(&self) -> Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifdstaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued a ioctl for getting the destination address, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_dstaddr` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_dstaddr }.into())
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
    pub(crate) fn new_ifreq(&self) -> bindings::ifreq {
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

    fn add_flags(&self, flags: i32) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_flags = self.read_flags()? as i16;

        unsafe {
            ifreq.ifr_ifru.ifru_flags |= flags as i16;

            ioctl::siocsifflags(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?;
        }

        Ok(())
    }

    fn del_flags(&self, flags: i32) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_flags = self.read_flags()? as i16;

        unsafe {
            ifreq.ifr_ifru.ifru_flags &= !(flags) as i16;

            ioctl::siocsifflags(
                self.inet4_socket.as_raw_fd(),
                &ifreq as *const bindings::ifreq,
            )?;
        }

        Ok(())
    }

    /// Writes the data in `buf` into the device.
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(nix::unistd::write(self.file.as_raw_fd(), buf)?)
    }

    /// Reads the data from device into `buf`.
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        Ok(nix::unistd::read(self.file.as_raw_fd(), buf)?)
    }
}
// tap specific
impl Device<Tap> {
    /// Set the hwaddr of the device.
    pub fn set_hwaddr(&self, hwaddr: [u8; 6]) -> Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_hwaddr = sockaddr::from(hwaddr);

        unsafe {
            ioctl::siocsifhwaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        Ok(())
    }

    /// Get the hwaddr of the device.
    pub fn get_hwaddr(&self) -> Result<[u8; 6]> {
        let mut ifreq = self.new_ifreq();

        unsafe {
            ioctl::siocgifhwaddr(
                self.inet4_socket.as_raw_fd(),
                &mut ifreq as *mut bindings::ifreq,
            )?
        };

        // Safety:
        //
        // Since we issued a ioctl for getting the hardware address, it's safe to assume
        // that if the ioctl was successfull, kernel had set the `ifru_hwaddr` variant.
        Ok(unsafe { ifreq.ifr_ifru.ifru_hwaddr }.into())
    }
}
impl<IfType: InterfaceType> io::Read for Device<IfType> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl<IfType: InterfaceType> io::Write for Device<IfType> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl<IfType: InterfaceType> AsRawFd for Device<IfType> {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
