use std::net::{Ipv4Addr, UdpSocket};

use etherparse::{IpHeader, PacketHeaders, TransportHeader};
use tidy_tuntap::*;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    let mut tun = tun::AsyncTun::without_packet_info("tun10").unwrap();
    tun.bring_up().unwrap();
    tun.set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    tun.set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    tun.set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();

    let data = [1; 10];
    let udp_socket = UdpSocket::bind("10.10.10.1:33333").unwrap();
    udp_socket.send_to(&data, "10.10.10.2:44444").unwrap();

    let mut buf = [0; 1500];
    loop {
        let bytes_read = tun.read(&mut buf).await.unwrap();

        if let Ok(packet) = PacketHeaders::from_ip_slice(&buf[..bytes_read]) {
            let ip_h = packet.ip.unwrap();
            let transport_h = packet.transport.unwrap();
            if let (IpHeader::Version4(ipv4_h, _), TransportHeader::Udp(udp_h)) =
                (ip_h, transport_h)
            {
                assert_eq!(ipv4_h.source, [10, 10, 10, 1]);
                assert_eq!(ipv4_h.destination, [10, 10, 10, 2]);
                assert_eq!(udp_h.source_port, 33333);
                assert_eq!(udp_h.destination_port, 44444);
                assert_eq!(packet.payload, data);
                break;
            }
        }
    }
}
