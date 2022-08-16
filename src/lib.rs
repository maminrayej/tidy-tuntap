//! tidy-tuntap is a Rust wrapper for working with TUN/TAP devices in Linux.
//!
//! Creating, modifying, reading from, and writing to a TUN/TAP device can be done using the provided
//! [Interface](`crate::iface::Interface`) struct. The device will be removed alongside
//! with all the added routings from the system when the created `Interface` is dropped.
//!
//! For more info: [tuntap.txt](https://www.kernel.org/doc/Documentation/networking/tuntap.txt)

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

pub mod error;
pub mod flags;
pub mod iface;
pub mod tap;
pub mod tun;
