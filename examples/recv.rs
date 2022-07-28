use std::io::Read;
use std::net::{Ipv4Addr, UdpSocket};

use etherparse::{IpHeader, PacketHeaders, TransportHeader};
use tidy_tuntap::iface;

fn main() {
    let mut iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();

    iface.bring_up().unwrap();
    iface.set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    iface.set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    iface.set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();
    iface.set_owner(1000).unwrap();

    let data = [1; 10];
    let udp_socket = UdpSocket::bind("10.10.10.1:33333").unwrap();
    udp_socket.send_to(&data, "10.10.10.2:44444").unwrap();

    let mut buf = [0; 1500];
    loop {
        let bytes_read = iface.read(&mut buf).unwrap();

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
