use crate::bindings::*;
use crate::flags::Flags;
use crate::ioctl;
use crate::sockaddr::to_ipv4;
use crate::sockaddr::to_sockaddr_in;

use std::io;
use std::net;
use std::os::unix::prelude::{AsRawFd, RawFd};

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

enum Op {
    Add,
    Del,
}

pub struct Interface {
    name: [i8; 16],
    file: std::fs::File,
    socket: RawFd,
}

impl Interface {
    pub fn new(name: &str, mode: Mode, no_packet_info: bool) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        // Compute the flags.
        let mut flags = mode.into_flags();
        if no_packet_info {
            flags |= nix::libc::IFF_NO_PI;
        }

        // Compute the device name.
        let mut ifr_name = [0i8; 16];
        for (i, c) in name.as_bytes().iter().enumerate().take(15) {
            ifr_name[i] = *c as i8;
        }

        // Construct the request with the computed flags and name.
        let mut ifr: ifreq = unsafe { std::mem::zeroed() };
        ifr.ifr_ifru.ifru_flags = flags as i16;
        ifr.ifr_ifrn.ifrn_name = ifr_name;

        // Issue the ioctl to submit the request against the TUN/TAP file.
        unsafe { ioctl::tunsetiff(file.as_raw_fd(), &ifr as *const ifreq as u64)? };

        // Get the name chosen by the kernel.
        let name = unsafe { ifr.ifr_ifrn.ifrn_name };

        let socket = nix::sys::socket::socket(
            nix::sys::socket::AddressFamily::Inet,
            nix::sys::socket::SockType::Datagram,
            nix::sys::socket::SockFlag::empty(),
            None,
        )?;

        Ok(Interface { name, file, socket })
    }

    pub fn name(&self) -> String {
        self.name.iter().map(|c| *c as u8 as char).collect()
    }

    pub fn flags(&self) -> io::Result<Flags> {
        self.read_flags()?.try_into()
    }

    pub fn bring_up(&self) -> io::Result<()> {
        self.mod_flags(Op::Add, (nix::libc::IFF_UP | nix::libc::IFF_RUNNING) as i16)
    }

    pub fn bring_down(&self) -> io::Result<()> {
        self.mod_flags(Op::Del, (nix::libc::IFF_UP | nix::libc::IFF_RUNNING) as i16)
    }

    pub fn set_metric(&self, metric: i32) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_ivalue = metric;

        unsafe { ioctl::siocsifmetric(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    pub fn get_metric(&self) -> io::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifmetric(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(unsafe { ifreq.ifr_ifru.ifru_ivalue })
    }

    pub fn set_mtu(&self, mtu: i32) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_mtu = mtu;

        unsafe { ioctl::siocsifmtu(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    pub fn get_mtu(&self) -> io::Result<i32> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifmtu(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(unsafe { ifreq.ifr_ifru.ifru_mtu })
    }

    pub fn set_netmask(&self, netmask: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_netmask = unsafe { std::mem::transmute(to_sockaddr_in(netmask)) };

        unsafe { ioctl::siocsifnetmask(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    pub fn get_netmask(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifnetmask(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_netmask)
        }))
    }

    pub fn set_addr(&self, addr: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_addr = unsafe { std::mem::transmute(to_sockaddr_in(addr)) };

        unsafe { ioctl::siocsifaddr(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    pub fn get_addr(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifaddr(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_addr)
        }))
    }

    pub fn set_dst_addr(&self, addr: net::Ipv4Addr) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_dstaddr = unsafe { std::mem::transmute(to_sockaddr_in(addr)) };

        unsafe { ioctl::siocsifdstaddr(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }

    pub fn get_dst_addr(&self) -> io::Result<net::Ipv4Addr> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifdstaddr(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(to_ipv4(unsafe {
            std::mem::transmute(ifreq.ifr_ifru.ifru_dstaddr)
        }))
    }

    pub fn set_owner(&self, owner: i32) -> io::Result<()> {
        unsafe { ioctl::tunsetowner(self.file.as_raw_fd(), owner as u64)? };

        Ok(())
    }

    pub fn set_group(&self, group: i32) -> io::Result<()> {
        unsafe { ioctl::tunsetgroup(self.file.as_raw_fd(), group as u64)? };

        Ok(())
    }

    pub fn persist(&self, persist: bool) -> io::Result<()> {
        unsafe { ioctl::tunsetpersist(self.file.as_raw_fd(), if persist { 1 } else { 0 })? };

        Ok(())
    }

    fn new_ifreq(&self) -> ifreq {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifrn.ifrn_name = self.name;

        ifreq
    }

    fn read_flags(&self) -> io::Result<i16> {
        let mut ifreq = self.new_ifreq();

        unsafe { ioctl::siocgifflags(self.socket, &mut ifreq as *mut ifreq)? };

        Ok(unsafe { ifreq.ifr_ifru.ifru_flags })
    }

    fn mod_flags(&self, op: Op, new_flags: i16) -> io::Result<()> {
        let mut ifreq = self.new_ifreq();

        ifreq.ifr_ifru.ifru_flags = self.read_flags()?;

        unsafe {
            match op {
                Op::Add => ifreq.ifr_ifru.ifru_flags |= new_flags,
                Op::Del => ifreq.ifr_ifru.ifru_flags &= !(new_flags),
            }

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
