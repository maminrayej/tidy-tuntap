#![cfg_attr(docsrs, feature(doc_cfg))]

/*!
 * There are three kinds of TUN devices (these are also true for TAP devices):
 * * [Blocking TUN](crate::Tun)
 * * [Multiqueue TUN](crate::MQTun)
 * * [Async TUN](crate::AsyncTun)
 *
 * Note that there are also three types of `Device`s but you can't instantiate them. They're
 * there to hold the shared code between TUN/TAP devices.
 */

mod bindings;

mod ioctl;
mod sockaddr;

pub mod error;
pub mod flags;

#[cfg(feature = "tokio")]
mod asyncd;
#[cfg(feature = "tokio")]
pub use asyncd::*;

mod device;
pub use device::*;

mod multiq;
pub use multiq::*;
