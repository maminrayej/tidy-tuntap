use crate::bindings::ifreq;

nix::ioctl_write_int!(tunsetiff, 'T', 202);
nix::ioctl_write_int!(tunsetpersist, 'T', 203);
nix::ioctl_write_int!(tunsetowner, 'T', 204);
nix::ioctl_write_int!(tunsetgroup, 'T', 206);

nix::ioctl_write_ptr_bad!(siocsifflags, nix::libc::SIOCSIFFLAGS, ifreq);
nix::ioctl_write_ptr_bad!(siocsifaddr, nix::libc::SIOCSIFADDR, ifreq);
nix::ioctl_write_ptr_bad!(siocsifmtu, nix::libc::SIOCSIFMTU, ifreq);
nix::ioctl_write_ptr_bad!(siocsifnetmask, nix::libc::SIOCSIFNETMASK, ifreq);
nix::ioctl_write_ptr_bad!(siocsifdstaddr, nix::libc::SIOCSIFDSTADDR, ifreq);
nix::ioctl_write_ptr_bad!(siocsifbrdaddr, nix::libc::SIOCSIFBRDADDR, ifreq);
nix::ioctl_write_ptr_bad!(siocsifmetric, nix::libc::SIOCSIFMETRIC, ifreq);

nix::ioctl_read_bad!(siocgifflags, nix::libc::SIOCGIFFLAGS, ifreq);
nix::ioctl_read_bad!(siocgifmtu, nix::libc::SIOCGIFMTU, ifreq);
nix::ioctl_read_bad!(siocgifaddr, nix::libc::SIOCGIFADDR, ifreq);
nix::ioctl_read_bad!(siocgifdstaddr, nix::libc::SIOCGIFDSTADDR, ifreq);
nix::ioctl_read_bad!(siocgifbrdaddr, nix::libc::SIOCGIFBRDADDR, ifreq);
nix::ioctl_read_bad!(siocgifnetmask, nix::libc::SIOCGIFNETMASK, ifreq);
nix::ioctl_read_bad!(siocgifmetric, nix::libc::SIOCGIFMETRIC, ifreq);
