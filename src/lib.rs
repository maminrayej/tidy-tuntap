#![cfg_attr(docsrs, feature(doc_cfg))]

//! tidy-tuntap is a Rust wrapper for working with TUN/TAP devices in Linux.

// For working with TUN/TAP devices in Linux, we need some structs that the kernel uses to
// pass data between kernel space and userspace. Bindings to these structs are provided using this
// binding module. Most notably, the `ifreq` struct.
//
// For more info take a look at:
//  * The `build.rs` file.
//  * The `wrapper.h` file.
//  * man netdevice
mod bindings;

mod ioctl;
mod sockaddr;

pub mod dev;
pub mod error;
pub mod flags;
pub mod iface;
pub mod tap;
pub mod tun;
