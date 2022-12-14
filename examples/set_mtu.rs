use tidy_tuntap::*;

fn main() {
    let tun = Tun::new("tun10", false).unwrap();

    tun.set_mtu(1024).unwrap();

    assert_eq!(tun.get_mtu().unwrap(), 1024);
}
