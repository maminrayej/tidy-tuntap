bitflags::bitflags! {
    // IFF_UP            Interface is running.
    // IFF_BROADCAST     Valid broadcast address set.
    // IFF_DEBUG         Internal debugging flag.
    // IFF_LOOPBACK      Interface is a loopback interface.
    // IFF_POINTOPOINT   Interface is a point-to-point link.
    //
    // IFF_RUNNING       Resources allocated.
    // IFF_NOARP         No arp protocol, L2 destination address not set.
    // IFF_PROMISC       Interface is in promiscuous mode.
    // IFF_NOTRAILERS    Avoid use of trailers.
    // IFF_ALLMULTI      Receive all multicast packets.
    // IFF_MASTER        Master of a load balancing bundle.
    // IFF_SLAVE         Slave of a load balancing bundle.
    // IFF_MULTICAST     Supports multicast
    // IFF_PORTSEL       Is able to select media type via ifmap.
    // IFF_AUTOMEDIA     Auto media selection active.
    // IFF_DYNAMIC       The addresses are lost when the interface goes down.
    // IFF_LOWER_UP      Driver signals L1 up (since Linux 2.6.17)
    // IFF_DORMANT       Driver signals dormant (since Linux 2.6.17)
    // IFF_ECHO          Echo sent packets (since Linux 2.6.25)
    pub struct Flags: i16 {
        const IFF_UP = nix::libc::IFF_UP as i16;
        const IFF_BROADCAST = nix::libc::IFF_BROADCAST as i16;
        const IFF_DEBUG = nix::libc::IFF_DEBUG as i16;
        const IFF_LOOPBACK = nix::libc::IFF_LOOPBACK as i16;

        const IFF_RUNNING = nix::libc::IFF_RUNNING as i16;
        const IFF_NOARP = nix::libc::IFF_NOARP as i16;
        const IFF_PROMISC = nix::libc::IFF_PROMISC as i16;
        const IFF_NOTRAILERS = nix::libc::IFF_NOTRAILERS as i16;
        const IFF_ALLMULTI = nix::libc::IFF_ALLMULTI as i16;
        const IFF_MASTER = nix::libc::IFF_MASTER as i16;
        const IFF_SLAVE = nix::libc::IFF_SLAVE as i16;
        const IFF_MULTICAST = nix::libc::IFF_MULTICAST as i16;
        const IFF_PORTSEL = nix::libc::IFF_PORTSEL as i16;
        const IFF_AUTOMEDIA = nix::libc::IFF_AUTOMEDIA as i16;
        const IFF_DYNAMIC = nix::libc::IFF_DYNAMIC as i16;
        const IFF_LOWER_UP = nix::libc::IFF_LOWER_UP as i16;
        const IFF_DORMANT = nix::libc::IFF_DORMANT as i16;
        const IFF_ECHO = nix::libc::IFF_ECHO as i16;
    }
}

impl TryFrom<i16> for Flags {
    type Error = std::io::Error;

    fn try_from(value: i16) -> std::result::Result<Self, Self::Error> {
        Flags::from_bits(value).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to create Flags from the data returned by the kernel",
            )
        })
    }
}
