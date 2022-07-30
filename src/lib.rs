//! tidy-tuntap is a Rust wrapper for working with TUN/TAP devices in Linux.
//!
//! Creating, modifying, reading from, and writing to a TUN/TAP device can be done using the provided
//! [Interface](`crate::Interface`) struct. The device will be removed alongside
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

// Provides ergonomics to work with flags needed to interact with the kernel.
mod flags;

// Contains the struct representing the TUN/TAP device.
mod iface;

// Contains different ioctls needed to interact with the TUN/TAP device.
mod ioctl;

// Contains helper functions to translate between what the kernel uses to represent addresses and
// what the Rust standard library uses.
mod sockaddr;

pub use flags::Flags;
pub use iface::{Interface, Mode};
