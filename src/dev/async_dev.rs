use std::io::{self, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{dev, error, iface};

/// A non-blocking device representing a TUN/TAP device.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub struct AsyncDev(AsyncFd<dev::Dev>);

impl std::ops::Deref for AsyncDev {
    type Target = iface::Interface;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl AsyncDev {
    // Creates a new `AsyncDev` with the specified `iface_params`.
    pub(crate) fn from_params(iface_params: iface::InterfaceParams) -> error::Result<Self> {
        Ok(AsyncDev(AsyncFd::new(dev::Dev::from_params(
            iface_params,
        )?)?))
    }

    /// Tries to read data from the device and fill the buffer `buf`.
    ///
    /// # Arguments
    /// * `buf`: Buffer to be filled with the data read from the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes read from the device.
    /// * `Err`: If the device was not ready to be read(a `WOULDBLOCK` err), or some other error
    /// occurred.
    pub fn try_recv(&self, buf: &mut [u8]) -> error::Result<usize> {
        self.0.get_ref().recv(buf)
    }

    /// Tries to write data from the buf to the device.
    ///
    /// # Arguments
    /// * `buf`: Data to be written to the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes written to the device.
    /// * `Err`: If the device was not ready to be written to(a `WOULDBLOCK` err), or some other error
    /// occurred.
    pub fn try_send(&self, buf: &[u8]) -> error::Result<usize> {
        self.0.get_ref().send(buf)
    }

    /// Asyncronously reads data from the device and writes to the `buf`.
    ///
    /// # Arguments
    /// * `buf`: Buffer to be filled with the data read from the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes read from the device.
    /// * `Err`: If reading data was unsuccessful.
    pub async fn recv(&self, buf: &mut [u8]) -> error::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|tun| Ok(tun.get_ref().recv(buf)?)) {
                Ok(result) => return Ok(result?),
                Err(_would_block) => continue,
            }
        }
    }

    /// Asyncronously writes data from `buf` to the device.
    ///
    /// # Arguments
    /// * `buf`: Buffer to be written to the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes written from the device.
    /// * `Err`: If writting data was unsuccessful.
    pub async fn send(&self, buf: &[u8]) -> error::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            match guard.try_io(|tun| Ok(tun.get_ref().send(buf)?)) {
                Ok(result) => return Ok(result?),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncRead for AsyncDev {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let self_mut = self.get_mut();

        loop {
            let mut guard = futures::ready!(self_mut.0.poll_read_ready_mut(cx))?;

            match guard.try_io(|inner| {
                let read = inner.get_mut().read(buf.initialize_unfilled())?;
                buf.advance(read);

                Ok(read)
            }) {
                Ok(result) => return Poll::Ready(result.map(|_| ())),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncDev {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let self_mut = self.get_mut();

        loop {
            let mut guard = futures::ready!(self_mut.0.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
