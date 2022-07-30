bitflags::bitflags! {
    /// Bitflags used by the kernel to indicate the active flag word of the device.
    ///
    /// For more info: `man netdevice`
    pub struct Flags: i32 {
        /// Interface is running.
        const IFF_UP = nix::libc::IFF_UP ;

        /// Valid broadcast address set.
        const IFF_BROADCAST = nix::libc::IFF_BROADCAST ;

        /// Internal debugging flag.
        const IFF_DEBUG = nix::libc::IFF_DEBUG ;

        /// Interface is a loopback interface.
        const IFF_LOOPBACK = nix::libc::IFF_LOOPBACK ;

        /// Interface is a point-to-point link.
        const IFF_POINTOPOINT = nix::libc::IFF_POINTOPOINT;

        /// Resources allocated.
        const IFF_RUNNING = nix::libc::IFF_RUNNING ;

        /// No arp protocol, L2 destination address not set.
        const IFF_NOARP = nix::libc::IFF_NOARP ;

        /// Interface is in promiscuous mode.
        const IFF_PROMISC = nix::libc::IFF_PROMISC ;

        /// Avoid use of trailers.
        const IFF_NOTRAILERS = nix::libc::IFF_NOTRAILERS ;

        /// Receive all multicast packets.
        const IFF_ALLMULTI = nix::libc::IFF_ALLMULTI ;

        /// Master of a load balancing bundle.
        const IFF_MASTER = nix::libc::IFF_MASTER ;

        /// Slave of a load balancing bundle.
        const IFF_SLAVE = nix::libc::IFF_SLAVE ;

        /// Supports multicast.
        const IFF_MULTICAST = nix::libc::IFF_MULTICAST ;

        /// Is able to select media type via ifmap.
        const IFF_PORTSEL = nix::libc::IFF_PORTSEL ;

        /// Auto media selection active.
        const IFF_AUTOMEDIA = nix::libc::IFF_AUTOMEDIA ;

        /// The addresses are lost when the interface goes down.
        const IFF_DYNAMIC = nix::libc::IFF_DYNAMIC ;

        /// Driver signals L1 up (since Linux 2.6.17).
        const IFF_LOWER_UP = nix::libc::IFF_LOWER_UP ;

        /// Driver signals dormant (since Linux 2.6.17).
        const IFF_DORMANT = nix::libc::IFF_DORMANT ;

        /// Echo sent packets (since Linux 2.6.25).
        const IFF_ECHO = nix::libc::IFF_ECHO ;
    }
}

// The kernel returns these flags as an `i32`. This impl tries to convert that
// to a more ergonomic struct provided above.
impl TryFrom<i32> for Flags {
    type Error = std::io::Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        Flags::from_bits(value).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to create Flags from the data returned by the kernel",
            )
        })
    }
}
