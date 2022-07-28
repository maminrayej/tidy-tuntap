use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use etherparse::PacketBuilder;
use tidy_tuntap::iface;

fn main() {
    let mut iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();
    iface.bring_up().unwrap();
    iface.set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    iface.set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    iface.set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();
    iface.set_owner(1000).unwrap();

    let socket = UdpSocket::bind("10.10.10.1:2424").unwrap();

    let data = [1; 10];
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
    builder.write(&mut packet, &data).unwrap();

    let _ = iface.write(&packet).unwrap();

    let mut buf = [0; 50];
    let (bytes_read, source) = socket.recv_from(&mut buf).unwrap();

    assert_eq!(bytes_read, 10);
    assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
    assert_eq!(source.port(), 4242);
    assert_eq!(data, &buf[..bytes_read]);
}
