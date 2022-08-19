use std::net::Ipv4Addr;

use tidy_tuntap::*;

fn main() {
    let tun = tun::Tun::without_packet_info("tun10").unwrap();

    tun.set_addr(Ipv4Addr::new(10, 10, 10, 10)).unwrap();
    tun.set_netmask(Ipv4Addr::new(255, 255, 0, 0)).unwrap();

    assert_eq!(tun.get_netmask().unwrap(), Ipv4Addr::new(255, 255, 0, 0));
}
