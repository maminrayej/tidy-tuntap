use crate::bindings;

// Can be used to set the flags and name of the TUN/TAP device.
// The flag can indicate whether the device is a TUN device or a TAP device.
// We can also determine whether we want the packet info or not.
nix::ioctl_write_int!(tunsetiff, 'T', 202);

// Can be used to make the TUN/TAP device persistent. In this mode,
// the device won't be destroyed when the last process closes the associated file descriptor.
nix::ioctl_write_int!(tunsetpersist, 'T', 203);

// Can be used to assign a persistent device to a given user or a whole group
// in order to give a non-root user access to a TUN/TAP device.
nix::ioctl_write_int!(tunsetowner, 'T', 204);
nix::ioctl_write_int!(tunsetgroup, 'T', 206);

// Can be used to attach or detach a mutliqueue.
nix::ioctl_write_int!(tunsetqueue, 'T', 217);

// Can be used to set and get the active flags of the device.
nix::ioctl_write_ptr_bad!(siocsifflags, nix::libc::SIOCSIFFLAGS, bindings::ifreq);
nix::ioctl_read_bad!(siocgifflags, nix::libc::SIOCGIFFLAGS, bindings::ifreq);

// Can be used to set and get the ip address of the device.
nix::ioctl_write_ptr_bad!(siocsifaddr, nix::libc::SIOCSIFADDR, bindings::ifreq);
nix::ioctl_write_ptr_bad!(siocsifaddr6, nix::libc::SIOCSIFADDR, bindings::in6_ifreq);
nix::ioctl_read_bad!(siocgifaddr, nix::libc::SIOCGIFADDR, bindings::ifreq);

// Can be used to set and get the MTU of the device.
nix::ioctl_write_ptr_bad!(siocsifmtu, nix::libc::SIOCSIFMTU, bindings::ifreq);
nix::ioctl_read_bad!(siocgifmtu, nix::libc::SIOCGIFMTU, bindings::ifreq);

// Can be used to set and get the netmask of the device.
nix::ioctl_write_ptr_bad!(siocsifnetmask, nix::libc::SIOCSIFNETMASK, bindings::ifreq);
nix::ioctl_read_bad!(siocgifnetmask, nix::libc::SIOCGIFNETMASK, bindings::ifreq);

// Can be used to set and get the destination address of a point to point device.
nix::ioctl_write_ptr_bad!(siocsifdstaddr, nix::libc::SIOCSIFDSTADDR, bindings::ifreq);
nix::ioctl_read_bad!(siocgifdstaddr, nix::libc::SIOCGIFDSTADDR, bindings::ifreq);

// Can be used to set and get the broadcast address of the device.
nix::ioctl_write_ptr_bad!(siocsifbrdaddr, nix::libc::SIOCSIFBRDADDR, bindings::ifreq);
nix::ioctl_read_bad!(siocgifbrdaddr, nix::libc::SIOCGIFBRDADDR, bindings::ifreq);

// Can be used to set and get the metric of the device.
nix::ioctl_write_ptr_bad!(siocsifmetric, nix::libc::SIOCSIFMETRIC, bindings::ifreq);
nix::ioctl_read_bad!(siocgifmetric, nix::libc::SIOCGIFMETRIC, bindings::ifreq);

// Can be used to get the device index.
nix::ioctl_read_bad!(siocgifindex, bindings::SIOCGIFINDEX, bindings::ifreq);

// Can be used to delete an IPv6 address of the device.
nix::ioctl_write_ptr_bad!(siocdifaddr6, bindings::SIOCDIFADDR, bindings::in6_ifreq);
