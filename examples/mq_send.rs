use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use etherparse::PacketBuilder;
use tidy_tuntap::*;

fn main() {
    let mq = Tun::new_mq("tun10", 3, false).unwrap();

    mq[0].bring_up().unwrap();
    mq[0].set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    mq[0].set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    mq[0].set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();

    let socket = UdpSocket::bind("10.10.10.1:2424").unwrap();

    let data = 2u8.to_be_bytes();
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
    builder.write(&mut packet, &data).unwrap();

    std::thread::scope(|s| {
        mq.iter().for_each(|tun| {
            s.spawn(|| {
                tun.send(&packet).unwrap();
            });
        });
    });

    let mut buf = [0u8; 3];

    let mut bytes_read = 0;
    while bytes_read != 3 {
        let (read, source) = socket.recv_from(&mut buf[bytes_read..]).unwrap();

        bytes_read += read;

        assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
        assert_eq!(source.port(), 4242);
    }

    (0..3).for_each(|i| {
        assert_eq!(buf[i], 2u8);
    });
}
