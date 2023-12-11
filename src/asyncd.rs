use std::future::poll_fn;
use std::marker::PhantomData;
use std::ops;
use std::pin::Pin;

use futures::io::AllowStdIo;
use futures::{AsyncRead, AsyncWrite};

use crate::common::create_device;
use crate::device::Device;
use crate::error::{Error, Result};
use crate::InterfaceType;

/// Represents a non-blocking TUN/TAP device.
///
/// Contains an async device.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[derive(Debug)]
pub struct AsyncDevice<IfType: InterfaceType>(AllowStdIo<Device<IfType>>);
impl<IfType: InterfaceType> AsyncDevice<IfType> {
    pub(crate) fn new(name: impl AsRef<str>, packet_info: bool) -> Result<Self> {
        let (name, mut files, inet4_socket, inet6_socket) =
            create_device(name, IfType::MODE, 1, packet_info, true)?;

        Ok(AsyncDevice(AllowStdIo::new(Device::<IfType> {
            name,
            file: files.pop().unwrap(),
            inet4_socket,
            inet6_socket,
            _phantom: PhantomData,
        })))
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
    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        poll_fn(|cx| {
            <AllowStdIo<Device<IfType>> as AsyncRead>::poll_read(Pin::new(&mut self.0), cx, buf)
        })
        .await
        .map_err(Error::IOError)
    }

    /// Asyncronously writes data from `buf` to the device.
    ///
    /// # Arguments
    /// * `buf`: Buffer to be written to the device.
    ///
    /// # Returns
    /// * `Ok`: Containing the number of bytes written from the device.
    /// * `Err`: If writting data was unsuccessful.
    pub async fn send(&mut self, buf: &[u8]) -> Result<usize> {
        poll_fn(|cx| {
            <AllowStdIo<Device<IfType>> as AsyncWrite>::poll_write(Pin::new(&mut self.0), cx, buf)
        })
        .await
        .map_err(Error::IOError)
    }
}
// Currently necessary due to missing Deref and DerefMut implementation: https://github.com/rust-lang/futures-rs/issues/2806.
impl<IfType: InterfaceType> AsyncRead for AsyncDevice<IfType> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        <AllowStdIo<Device<IfType>> as AsyncRead>::poll_read(
            Pin::new(&mut self.get_mut().0),
            cx,
            buf,
        )
    }
}
impl<IfType: InterfaceType> AsyncWrite for AsyncDevice<IfType> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        <AllowStdIo<Device<IfType>> as AsyncWrite>::poll_write(
            Pin::new(&mut self.get_mut().0),
            cx,
            buf,
        )
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        <AllowStdIo<Device<IfType>> as AsyncWrite>::poll_close(Pin::new(&mut self.get_mut().0), cx)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        <AllowStdIo<Device<IfType>> as AsyncWrite>::poll_flush(Pin::new(&mut self.get_mut().0), cx)
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
