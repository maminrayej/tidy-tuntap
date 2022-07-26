use tidy_tuntap::flags::Flags;
use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();

    let flags = iface.flags().unwrap();
    let name = iface.name();

    assert!(!flags.intersects(Flags::IFF_UP | Flags::IFF_RUNNING));
    assert_eq!(name, "tun10");
}
