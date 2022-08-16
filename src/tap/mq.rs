use std::sync;

use crate::error::Result;
use crate::iface;
use crate::tap::Tap;

pub struct MQTap(Vec<Tap>);

impl MQTap {
    fn new(iface_params: iface::InterfaceParams) -> Result<MQTap> {
        let iface = sync::Arc::new(iface::Interface::new(iface_params)?);

        let tuns: Result<Vec<Tap>> = (0..iface.files.len())
            .map(|fd_index| Tap::new(iface.clone(), fd_index))
            .collect();

        Ok(MQTap(tuns?))
    }

    pub fn without_packet_info(name: &str, len: usize) -> Result<MQTap> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: len,
            non_blocking: false,
            no_packet_info: true,
        })
    }

    pub fn with_packet_info(name: &str, len: usize) -> Result<MQTap> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tap,
            fd_count: len,
            non_blocking: false,
            no_packet_info: false,
        })
    }
}
