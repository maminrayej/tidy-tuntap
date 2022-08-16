use std::sync;

use crate::error::Result;
use crate::iface;
use crate::tun::Tun;

pub struct MQTun(Vec<Tun>);

impl MQTun {
    fn new(iface_params: iface::InterfaceParams) -> Result<MQTun> {
        let iface = sync::Arc::new(iface::Interface::new(iface_params)?);

        let tuns: Result<Vec<Tun>> = (0..iface.files.len())
            .map(|fd_index| Tun::new(iface.clone(), fd_index))
            .collect();

        Ok(MQTun(tuns?))
    }

    pub fn without_packet_info(name: &str, len: usize) -> Result<MQTun> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: len,
            non_blocking: false,
            no_packet_info: true,
        })
    }

    pub fn with_packet_info(name: &str, len: usize) -> Result<MQTun> {
        Self::new(iface::InterfaceParams {
            name,
            mode: iface::Mode::Tun,
            fd_count: len,
            non_blocking: false,
            no_packet_info: false,
        })
    }
}
