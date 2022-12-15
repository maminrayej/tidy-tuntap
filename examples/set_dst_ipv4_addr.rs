use std::net::Ipv4Addr;

use tidy_tuntap::*;

fn main() {
    let tun = Tun::new("tun10", false).unwrap();

    tun.set_addr(Ipv4Addr::new(10, 10, 10, 10)).unwrap();
    tun.set_dst_addr(Ipv4Addr::new(10, 10, 10, 11)).unwrap();

    assert_eq!(tun.get_dst_addr().unwrap(), Ipv4Addr::new(10, 10, 10, 11));
}
