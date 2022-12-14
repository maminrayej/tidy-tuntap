use std::net::Ipv6Addr;
use std::str::FromStr;

use tidy_tuntap::*;

fn main() {
    let tun = Tun::new("tun10", false).unwrap();

    tun.set_ipv6_addr(Ipv6Addr::from_str("fe80::be8f:5838:c7ca:b98").unwrap())
        .unwrap();

    tun.set_ipv6_addr(Ipv6Addr::from_str("fe80::be8f:5838:c7ca:b99").unwrap())
        .unwrap();
    tun.del_ipv6_addr(Ipv6Addr::from_str("fe80::be8f:5838:c7ca:b99").unwrap())
        .unwrap();

    let ipv6_addrs = tun.get_ipv6_addrs().unwrap();

    assert_eq!(ipv6_addrs.len(), 1);
    assert_eq!(
        ipv6_addrs[0],
        Ipv6Addr::from_str("fe80::be8f:5838:c7ca:b98").unwrap()
    );
}
