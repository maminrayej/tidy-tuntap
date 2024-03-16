use crate::{error::Result, AsyncDevice, Device, MQDevice, Mode};

pub trait InterfaceType: Sized {
    const MODE: Mode;

    fn new(name: impl AsRef<str>, packet_info: bool) -> Result<Device<Self>> {
        Device::<Self>::new(name, packet_info)
    }
    fn new_mq(
        name: impl AsRef<str>,
        device_count: usize,
        packet_info: bool,
    ) -> Result<Vec<MQDevice<Self>>> {
        MQDevice::<Self>::new(name, device_count, packet_info)
    }
    fn new_async(name: impl AsRef<str>, packet_info: bool) -> Result<AsyncDevice<Self>> {
        AsyncDevice::<Self>::new(name, packet_info)
    }
}

pub struct Tun;
impl InterfaceType for Tun {
    const MODE: Mode = Mode::Tun;
}
pub struct Tap;
impl InterfaceType for Tap {
    const MODE: Mode = Mode::Tap;
}
