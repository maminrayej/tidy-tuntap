use tidy_tuntap::*;

fn main() {
    let iface = Interface::without_packet_info("tun10", Mode::Tun).unwrap();

    let flags = iface.flags().unwrap();
    let name = iface.name();

    assert!(!flags.intersects(Flags::IFF_UP | Flags::IFF_RUNNING));
    assert_eq!(name, "tun10");
}
