use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();

    iface
        .set_addr(std::net::Ipv4Addr::new(128, 0, 0, 1))
        .unwrap();
    iface
        .set_netmask(std::net::Ipv4Addr::new(255, 255, 0, 0))
        .unwrap();

    assert_eq!(
        iface.get_netmask().unwrap(),
        std::net::Ipv4Addr::new(255, 255, 0, 0)
    );
}
