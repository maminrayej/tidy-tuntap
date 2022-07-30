use tidy_tuntap::*;

fn main() {
    let iface = Interface::without_packet_info("tun10", Mode::Tun).unwrap();
    iface.bring_up().unwrap();

    let flags = iface.flags().unwrap();

    assert!(flags.contains(Flags::IFF_UP | Flags::IFF_RUNNING));
}
