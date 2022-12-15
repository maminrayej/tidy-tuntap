use tidy_tuntap::*;

fn main() {
    let tun = Tun::new("tun10", false).unwrap();

    let flags = tun.flags().unwrap();
    let name = tun.name();

    assert!(!flags.intersects(flags::Flags::IFF_UP | flags::Flags::IFF_RUNNING));
    assert_eq!(name, "tun10");
}
