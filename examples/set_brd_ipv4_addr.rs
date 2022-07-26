use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();

    iface
        .set_addr(std::net::Ipv4Addr::new(128, 0, 0, 1))
        .unwrap();
    iface
        .set_brd_addr(std::net::Ipv4Addr::new(129, 0, 0, 1))
        .unwrap();

    assert_eq!(
        iface.get_brd_addr().unwrap(),
        std::net::Ipv4Addr::new(129, 0, 0, 1)
    );
}
