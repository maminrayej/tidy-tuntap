use std::net::Ipv4Addr;

use tidy_tuntap::*;

#[tokio::main]
async fn main() {
    let tun = tun::AsyncTun::without_packet_info("tun10").unwrap();
    tun.bring_up().unwrap();
    tun.set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    tun.set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    tun.set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();

    let mut buf = [0; 1500];

    tun.try_recv(&mut buf).unwrap();
}
