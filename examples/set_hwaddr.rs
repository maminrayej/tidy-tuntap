use tidy_tuntap::*;

fn main() {
    let tun = Tap::new("tun10", false).unwrap();

    let hwaddr = [0x00, 0x80, 0x41, 0x13, 0x37, 0x42];

    tun.set_hwaddr(hwaddr).unwrap();

    assert_eq!(tun.get_hwaddr().unwrap(), hwaddr);
}
