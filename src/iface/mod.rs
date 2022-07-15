use crate::bindings::*;

use std::fs::OpenOptions;
use std::io::{Read, Result, Write};
use std::os::unix::prelude::{AsRawFd, RawFd};

mod ioctl {
    use crate::bindings::ifreq;

    nix::ioctl_write_int!(tunsetiff, 'T', 202);
    nix::ioctl_write_ptr_bad!(siocsifflags, nix::libc::SIOCSIFFLAGS, ifreq);
    nix::ioctl_read_bad!(siocgifflag, nix::libc::SIOCGIFFLAGS, ifreq);
}

pub enum Mode {
    Tun,
    Tap,
}

impl Mode {
    fn into_flags(self) -> i16 {
        match self {
            Mode::Tun => nix::libc::IFF_TUN as i16,
            Mode::Tap => nix::libc::IFF_TAP as i16,
        }
    }
}

pub enum Op {
    Add,
    Del,
}

impl Op {
    fn operate(&self, original_flags: &mut i16, mod_flags: i16) {
        match self {
            Op::Add => *original_flags |= mod_flags,
            Op::Del => *original_flags &= !(mod_flags),
        }
    }
}

pub struct Interface {
    name: [i8; 16],
    file: std::fs::File,
    socket: RawFd,
}

impl Interface {
    pub fn new(name: &str, mode: Mode, no_packet_info: bool) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        // Compute the flags.
        let mut flags = mode.into_flags();
        if no_packet_info {
            flags |= nix::libc::IFF_NO_PI as i16;
        }

        // Compute the device name.
        let mut ifr_name = [0i8; 16];
        for (i, c) in name.as_bytes().iter().enumerate().take(15) {
            ifr_name[i] = *c as i8;
        }

        // Construct the request with the computed flags and name.
        let mut ifr: ifreq = unsafe { std::mem::zeroed() };
        ifr.ifr_ifru.ifru_flags = flags;
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

    pub fn up(&self) -> Result<()> {
        self.mod_flag(Op::Add, (nix::libc::IFF_UP | nix::libc::IFF_RUNNING) as i16)
    }

    pub fn down(&self) -> Result<()> {
        self.mod_flag(Op::Del, (nix::libc::IFF_UP | nix::libc::IFF_RUNNING) as i16)
    }

    fn mod_flag(&self, op: Op, flags: i16) -> Result<()> {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };

        ifreq.ifr_ifrn.ifrn_name = self.name;
        unsafe { ioctl::siocgifflag(self.socket, &mut ifreq as *mut ifreq)? };
        unsafe { op.operate(&mut ifreq.ifr_ifru.ifru_flags, flags) };
        unsafe { ioctl::siocsifflags(self.socket, &ifreq as *const ifreq)? };

        Ok(())
    }
}

impl Read for Interface {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf)
    }
}

impl Write for Interface {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        nix::unistd::close(self.socket).expect("Failed to close the socket");
    }
}
