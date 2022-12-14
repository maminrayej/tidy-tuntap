use tidy_tuntap::*;

fn main() {
    let tun = Tun::new("tun10", false).unwrap();

    tun.bring_up().unwrap();

    let flags = tun.flags().unwrap();

    assert!(flags.contains(flags::Flags::IFF_UP | flags::Flags::IFF_RUNNING));
}
