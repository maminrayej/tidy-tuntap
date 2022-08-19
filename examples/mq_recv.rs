use std::net::{Ipv4Addr, UdpSocket};
use std::os::unix::prelude::AsRawFd;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use etherparse::{IpHeader, PacketHeaders, TransportHeader};
use tidy_tuntap::*;

fn main() {
    let mq = Arc::new(tun::MQTun::without_packet_info("tun10", 3).unwrap());

    mq[0].bring_up().unwrap();
    mq[0].set_addr(Ipv4Addr::new(10, 10, 10, 1)).unwrap();
    mq[0].set_brd_addr(Ipv4Addr::new(10, 10, 10, 255)).unwrap();
    mq[0].set_netmask(Ipv4Addr::new(255, 255, 255, 0)).unwrap();

    let src_ports = Arc::new(Mutex::new(Vec::new()));
    let dst_ports = Arc::new(Mutex::new(Vec::new()));
    let sum = Arc::new(AtomicU8::new(0));

    let dsts = ["10.10.10.2:3131", "10.10.10.2:3132", "10.10.10.2:3133"];
    let data = [1u8.to_be_bytes(), 2u8.to_be_bytes(), 3u8.to_be_bytes()];
    let scks = [
        UdpSocket::bind("10.10.10.1:4141").unwrap(),
        UdpSocket::bind("10.10.10.1:4142").unwrap(),
        UdpSocket::bind("10.10.10.1:4143").unwrap(),
    ];

    // Spawn receivers
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let mq = mq.clone();
            let src_ports = src_ports.clone();
            let dst_ports = dst_ports.clone();
            let sum = sum.clone();

            std::thread::spawn(move || {
                let mut buf = [0; 1500];
                loop {
                    if sum.load(Ordering::SeqCst) == 6 {
                        break;
                    }

                    let mut fd_set = nix::sys::select::FdSet::new();
                    fd_set.insert(mq[i].as_raw_fd());
                    let ready_count = nix::sys::select::select(
                        None,
                        Some(&mut fd_set),
                        None,
                        None,
                        &mut nix::sys::time::TimeVal::new(0, 100_000),
                    )
                    .unwrap();

                    if ready_count < 1 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }

                    let bytes_read = mq[i].recv(&mut buf).unwrap();

                    if let Ok(packet) = PacketHeaders::from_ip_slice(&buf[..bytes_read]) {
                        let ip_h = packet.ip.unwrap();
                        let transport_h = packet.transport.unwrap();
                        if let (IpHeader::Version4(ipv4_h, _), TransportHeader::Udp(udp_h)) =
                            (ip_h, transport_h)
                        {
                            assert_eq!(ipv4_h.source, [10, 10, 10, 1]);
                            assert_eq!(ipv4_h.destination, [10, 10, 10, 2]);

                            let mut src_ports = src_ports.lock().unwrap();
                            src_ports.push(udp_h.source_port);

                            let mut dst_ports = dst_ports.lock().unwrap();
                            dst_ports.push(udp_h.destination_port);

                            packet.payload.iter().for_each(|data| {
                                sum.fetch_add(*data, Ordering::SeqCst);
                            });
                        }
                    }
                }
            })
        })
        .collect();

    // Spawn senders
    std::thread::scope(|s| {
        (0..3).for_each(|i| {
            let socket = &scks[i];
            let data = &data[i];
            let dst = &dsts[i];

            s.spawn(move || {
                socket.send_to(data, dst).unwrap();
            });
        });
    });

    for handle in handles {
        handle.join().unwrap();
    }

    let src_ports = Arc::try_unwrap(src_ports).unwrap().into_inner().unwrap();
    let dst_ports = Arc::try_unwrap(dst_ports).unwrap().into_inner().unwrap();

    assert_eq!(src_ports.len(), 3);
    assert!(src_ports.contains(&4141));
    assert!(src_ports.contains(&4142));
    assert!(src_ports.contains(&4143));

    assert_eq!(dst_ports.len(), 3);
    assert!(dst_ports.contains(&3131));
    assert!(dst_ports.contains(&3132));
    assert!(dst_ports.contains(&3133));
}
