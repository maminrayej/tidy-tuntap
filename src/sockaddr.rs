// The Rust standard library uses the `std::net::Ipv4Addr` as a representation of
// ipv4 addresses. But, the kernel uses `sockaddr_in` as a representation of ip addresses.
// So in order to pass an ip address representation between Rust code and the kernel,
// we need conversion between these two types. These conversions are provided by the two functions below.
//
// NOTE: As far as I understand kernel exposes `sockaddr` struct to represent ipv4 addresses in an
// `ifreq`. But, it's best practice to use `sockaddr_in` to initialize the address and transmute it to
// `sockaddr` when wanting to send it to the kernel. As shown below, these two structs have the
// same size (and maybe alignment?) so transmuting between these two types suppose to be safe.
//
// struct sockaddr {
//      unsigned short int sa;              2  bytes
//      char sa_data[14];                   14 bytes
// }                                      ------------
//                                          16 bytes
//
// struct sockaddr_in {
//      unsigned short sin_family;          2 bytes
//      unsigned short sin_port;            2 bytes
//
//      // in_addr is basically a struct
//      // containing an unsigned int
//      in_addr        sin_addr;            4 bytes
//
//      char padding[8];                    8 bytes
// }                                      -----------
//                                         16 bytes

use std::net;

use crate::bindings;

pub use bindings::sockaddr as sockaddr;

impl From<net::Ipv4Addr> for bindings::sockaddr {
    fn from(addr: net::Ipv4Addr) -> Self {
        let mut sockaddr_in: nix::libc::sockaddr_in = unsafe { std::mem::zeroed() };
    
        sockaddr_in.sin_family = nix::libc::AF_INET as u16;
        sockaddr_in.sin_addr = nix::libc::in_addr {
            s_addr: u32::from_le_bytes(addr.octets()),
        };
        sockaddr_in.sin_port = 0;
    
        unsafe { std::mem::transmute(sockaddr_in) }
    }
}
impl From<[u8; 6]> for bindings::sockaddr {
    fn from(addr: [u8; 6]) -> Self {
        let mut padded_addr = [0u8; 14];
        padded_addr[..6].copy_from_slice(&addr);
        
        bindings::sockaddr { sa_family: nix::libc::ARPHRD_ETHER, sa_data: padded_addr.map(|x| x as i8) }
    }
}
impl Into<net::Ipv4Addr> for bindings::sockaddr {
    fn into(self) -> net::Ipv4Addr {
        let sockaddr_in: nix::libc::sockaddr_in = unsafe { std::mem::transmute(self) };

        sockaddr_in.sin_addr.s_addr.to_le_bytes().into()
    }
}
impl Into<[u8; 6]> for bindings::sockaddr {
    fn into(self) -> [u8; 6] {
        let mut addr = [0i8; 6];
        addr.copy_from_slice(&self.sa_data[..6]);
        addr.map(|x| x as u8)
    }
}
