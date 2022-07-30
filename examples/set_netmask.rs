use std::net::Ipv4Addr;

use tidy_tuntap::*;

fn main() {
    let iface = Interface::without_packet_info("tun10", Mode::Tun).unwrap();

    iface.set_addr(Ipv4Addr::new(10, 10, 10, 10)).unwrap();
    iface.set_netmask(Ipv4Addr::new(255, 255, 0, 0)).unwrap();

    assert_eq!(iface.get_netmask().unwrap(), Ipv4Addr::new(255, 255, 0, 0));
}
