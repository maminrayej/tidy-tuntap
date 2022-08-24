use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use etherparse::PacketBuilder;
use tidy_tuntap::*;

#[tokio::main]
async fn main() {
    let tun = tun::AsyncTun::without_packet_info("tun10").unwrap();
    tun.bring_up().unwrap();
    tun.set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    tun.set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    tun.set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();

    let socket = UdpSocket::bind("10.10.10.1:2424").unwrap();

    let data = [1; 10];
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
    builder.write(&mut packet, &data).unwrap();

    let _ = tun.send(&packet).await.unwrap();

    let mut buf = [0; 50];
    let (bytes_read, source) = socket.recv_from(&mut buf).unwrap();

    assert_eq!(bytes_read, 10);
    assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
    assert_eq!(source.port(), 4242);
    assert_eq!(data, &buf[..bytes_read]);
}
