#![cfg_attr(docsrs, feature(doc_cfg))]

/*!
 * TUN/TAP provides packet reception and transmission for user space programs.
 * It can be seen as a simple Point-to-Point or Ethernet device, which,
 * instead of receiving packets from physical media, receives them from
 * user space program and instead of sending packets via physical media
 * writes them to the user space program.
 *
 * This crate provides three different kinds of TUN/TAP devices:
 * * Blocking: [`Tun`](crate::Tun)/[`Tap`](crate::Tap)
 * * Multiqueue: [`MQTun`](crate::MQTun)/[`MQTap`](crate::MQTap)
 * * Non-blocking: [`AsyncTun`](crate::AsyncTun)/[`AsyncTap`](crate::AsyncTap)
 *
 * **NOTE**: There is a device type corrospoding to each TUN/TAP type. You can't construct these
 * devices since they're only there to contain the shared code between TUN/TAP devices.
 */

mod bindings;

mod ioctl;
mod sockaddr;

mod common;
pub use common::Mode;

pub mod error;
pub mod flags;

mod type_state;
pub use type_state::*;

mod device;
pub use device::*;

mod multiq;
pub use multiq::*;

mod asyncd;
pub use asyncd::*;
