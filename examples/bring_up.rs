use tidy_tuntap::*;

fn main() {
    let tun = tun::Tun::without_packet_info("tun10").unwrap();

    tun.bring_up().unwrap();

    let flags = tun.flags().unwrap();

    assert!(flags.contains(flags::Flags::IFF_UP | flags::Flags::IFF_RUNNING));
}
