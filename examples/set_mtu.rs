use tidy_tuntap::*;

fn main() {
    let iface = Interface::without_packet_info("tun10", Mode::Tun).unwrap();

    iface.set_mtu(1024).unwrap();

    assert_eq!(iface.get_mtu().unwrap(), 1024);
}
