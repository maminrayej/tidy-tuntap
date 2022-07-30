use std::net::Ipv4Addr;

use tidy_tuntap::*;

fn main() {
    let iface = Interface::without_packet_info("tun10", Mode::Tun).unwrap();

    iface.set_addr(Ipv4Addr::new(10, 10, 10, 10)).unwrap();
    iface.set_dst_addr(Ipv4Addr::new(10, 10, 10, 11)).unwrap();

    assert_eq!(iface.get_dst_addr().unwrap(), Ipv4Addr::new(10, 10, 10, 11));
}
