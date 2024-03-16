use tidy_tuntap::*;

fn main() {
    let tap = Tap::new("tap10", false).unwrap();

    let hwaddr = [0x00, 0x80, 0x41, 0x13, 0x37, 0x42];

    tap.set_hwaddr(hwaddr).unwrap();

    assert_eq!(tap.get_hwaddr().unwrap(), hwaddr);
}
