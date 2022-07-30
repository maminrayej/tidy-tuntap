// NOTE: There are two types of ioctls in the kernel. The good ones and the bad ones.
// The good ioctls use a combination of an ioctl identifier with an ioctl sequence number to
// generate the ioctl numbers. For example the TUNSETIFF ioctl is a good ioctl that uses the
// `T` identifier and 202 sequence number. There are also bad ioctls that don't use the new system
// of generating ioctl numbers and use hardcoded values. These ioctls are postfixed as `*_bad` by
// the nix crate.
// There is a difference between calling these two types of ioctls. When calling a good ioctl, we
// pass the value of the pointer to our struct as an `int`. That's why the TUNSETIFF ioctl is an
// `ioctl_write_int` macro instead of an `ioctl_write_ptr`. (I don't why the latter does not work
// but I've tested it and it doesn't. Feel free to open an issue on Github and correct this piece
// of document if you've found it to be false)
// But when calling the bad ioctls, we use the `ioctl_write_ptr_bad` and pass the pointer to our
// struct.

use crate::bindings::ifreq;

// Can be used to set the flags and name of the TUN/TAP device.
// The flag can indicate whether the device is a TUN device or a TAP device.
// We can also determine whether we want the packet info or not.
// For more info about its usage look at the `Interface::new` function in the `iface` module.
nix::ioctl_write_int!(tunsetiff, 'T', 202);

// Can be used to make the TUN/TAP interface persistent. In this mode,
// the interface won't be destroyed when the last process closes the associated /dev/net/tun file descriptor.
nix::ioctl_write_int!(tunsetpersist, 'T', 203);

// Can be used to assign a persistent interface to a given user or a whole group
// in order to give a non-root user access to a TUN/TAP interface.
nix::ioctl_write_int!(tunsetowner, 'T', 204);
nix::ioctl_write_int!(tunsetgroup, 'T', 206);

// Can be used to set and get the active flags of the device.
nix::ioctl_write_ptr_bad!(siocsifflags, nix::libc::SIOCSIFFLAGS, ifreq);
nix::ioctl_read_bad!(siocgifflags, nix::libc::SIOCGIFFLAGS, ifreq);

// Can be used to set and get the ip address of the device.
nix::ioctl_write_ptr_bad!(siocsifaddr, nix::libc::SIOCSIFADDR, ifreq);
nix::ioctl_read_bad!(siocgifaddr, nix::libc::SIOCGIFADDR, ifreq);

// Can be used to set and get the MTU of the device.
nix::ioctl_write_ptr_bad!(siocsifmtu, nix::libc::SIOCSIFMTU, ifreq);
nix::ioctl_read_bad!(siocgifmtu, nix::libc::SIOCGIFMTU, ifreq);

// Can be used to set and get the netmask of the device.
nix::ioctl_write_ptr_bad!(siocsifnetmask, nix::libc::SIOCSIFNETMASK, ifreq);
nix::ioctl_read_bad!(siocgifnetmask, nix::libc::SIOCGIFNETMASK, ifreq);

// Can be used to set and get the destination address of a point to point device.
nix::ioctl_write_ptr_bad!(siocsifdstaddr, nix::libc::SIOCSIFDSTADDR, ifreq);
nix::ioctl_read_bad!(siocgifdstaddr, nix::libc::SIOCGIFDSTADDR, ifreq);

// Can be used to set and get the broadcast address of the device.
nix::ioctl_write_ptr_bad!(siocsifbrdaddr, nix::libc::SIOCSIFBRDADDR, ifreq);
nix::ioctl_read_bad!(siocgifbrdaddr, nix::libc::SIOCGIFBRDADDR, ifreq);

// Can be used to set and get the metric of the device.
nix::ioctl_write_ptr_bad!(siocsifmetric, nix::libc::SIOCSIFMETRIC, ifreq);
nix::ioctl_read_bad!(siocgifmetric, nix::libc::SIOCGIFMETRIC, ifreq);
