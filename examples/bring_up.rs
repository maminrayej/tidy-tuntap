use tidy_tuntap::flags::Flags;
use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();
    iface.bring_up().unwrap();

    let flags = iface.flags().unwrap();

    assert!(flags.contains(Flags::IFF_UP | Flags::IFF_RUNNING));
}
