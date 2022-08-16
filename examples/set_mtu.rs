use tidy_tuntap::*;

fn main() {
    let tun = tun::Tun::without_packet_info("tun10").unwrap();

    tun.set_mtu(1024).unwrap();

    assert_eq!(tun.get_mtu().unwrap(), 1024);
}
