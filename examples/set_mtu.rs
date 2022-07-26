use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();

    iface.set_mtu(1024).unwrap();

    assert_eq!(iface.get_mtu().unwrap(), 1024);
}
