use crate::error::Result;
use crate::{dev, iface};

#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub struct AsyncTap(dev::AsyncDev);

impl std::ops::Deref for AsyncTap {
    type Target = dev::AsyncDev;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AsyncTap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsyncTap {
    pub fn without_packet_info(name: &str) -> Result<AsyncTap> {
        Ok(AsyncTap(dev::AsyncDev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tap,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: true,
            },
        )?))
    }

    pub fn with_packet_info(name: &str) -> Result<AsyncTap> {
        Ok(AsyncTap(dev::AsyncDev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tap,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: false,
            },
        )?))
    }
}
