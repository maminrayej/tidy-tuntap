// The Rust standard library uses the `std::net::Ipv4Addr` as a representation of ipv4 addresses.
// But, the kernel uses `sockaddr_in` as a representation of ip addresses.
// So in order to pass an ip address representation between Rust code and the kernel, we need conversion between
// these two types. These conversions are provided by the two functions below.

pub fn to_sockaddr_in(addr: std::net::Ipv4Addr) -> nix::libc::sockaddr_in {
    let mut sockaddr_in: nix::libc::sockaddr_in = unsafe { std::mem::zeroed() };

    sockaddr_in.sin_family = nix::libc::AF_INET as u16;
    sockaddr_in.sin_addr = nix::libc::in_addr {
        s_addr: u32::from_le_bytes(addr.octets()),
    };
    sockaddr_in.sin_port = 0;

    sockaddr_in
}

pub fn to_ipv4(sockaddr_in: nix::libc::sockaddr_in) -> std::net::Ipv4Addr {
    sockaddr_in.sin_addr.s_addr.to_le_bytes().into()
}
