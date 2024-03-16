use std::io::{Write, Read};
use std::marker::PhantomData;
use std::{ops, io};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::ready;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::unix::AsyncFd;

use crate::common::create_device;
use crate::device::Device;
use crate::error::Result;
use crate::InterfaceType;

/// Represents a non-blocking TUN/TAP device.
///
/// Contains an async device.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[derive(Debug)]
pub struct AsyncDevice<IfType: InterfaceType>(AsyncFd<Device<IfType>>);
impl<IfType: InterfaceType> AsyncDevice<IfType> {
    pub(crate) fn new(name: impl AsRef<str>, packet_info: bool) -> Result<Self> {
        let (name, mut files, inet4_socket, inet6_socket) =
            create_device(name, IfType::MODE, 1, packet_info, true)?;

        Ok(AsyncDevice(
            AsyncFd::new(Device::<IfType> {
                name,
                file: files.pop().unwrap(),
                inet4_socket,
                inet6_socket,
                _phantom: PhantomData,
            })
            .unwrap(),
        ))
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
    pub fn try_recv(&self, buf: &mut [u8]) -> Result<usize> {
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
    pub fn try_send(&self, buf: &[u8]) -> Result<usize> {
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
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
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
    pub async fn send(&self, buf: &[u8]) -> Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;

            match guard.try_io(|tun| Ok(tun.get_ref().send(buf)?)) {
                Ok(result) => return Ok(result?),
                Err(_would_block) => continue,
            }
        }
    }
}

impl<IfType: InterfaceType + Unpin> AsyncRead for AsyncDevice<IfType> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let self_mut = self.get_mut();

        loop {
            let mut guard = ready!(self_mut.0.poll_read_ready_mut(cx))?;

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

impl<IfType: InterfaceType + Unpin> AsyncWrite for AsyncDevice<IfType> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let self_mut = self.get_mut();

        loop {
            let mut guard = ready!(self_mut.0.poll_write_ready_mut(cx))?;

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
impl<IfType: InterfaceType> ops::Deref for AsyncDevice<IfType> {
    type Target = Device<IfType>;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}
impl<IfType: InterfaceType> ops::DerefMut for AsyncDevice<IfType> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut()
    }
}
