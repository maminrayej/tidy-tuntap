use std::net::Ipv4Addr;

use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();

    iface.set_addr(Ipv4Addr::new(10, 10, 10, 10)).unwrap();

    assert_eq!(iface.get_addr().unwrap(), Ipv4Addr::new(10, 10, 10, 10));
}
