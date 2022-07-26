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
