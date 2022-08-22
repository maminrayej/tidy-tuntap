use std::io::{self, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::error::Result;
use crate::{dev, iface};

pub struct AsyncTun(AsyncFd<dev::Dev>);

impl std::ops::Deref for AsyncTun {
    type Target = dev::Dev;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl AsyncTun {
    pub fn without_packet_info(name: &str) -> Result<AsyncTun> {
        Ok(AsyncTun(AsyncFd::new(dev::Dev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tun,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: true,
            },
        )?)?))
    }

    pub fn with_packet_info(name: &str) -> Result<AsyncTun> {
        Ok(AsyncTun(AsyncFd::new(dev::Dev::from_params(
            iface::InterfaceParams {
                name,
                mode: iface::Mode::Tun,
                fd_count: 1,
                non_blocking: true,
                no_packet_info: false,
            },
        )?)?))
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;

            match guard.try_io(|tun| Ok(tun.get_ref().recv(buf)?)) {
                Ok(result) => return Ok(result?),
                Err(_would_block) => continue,
            }
        }
    }

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

impl AsyncRead for AsyncTun {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let self_mut = self.get_mut();

        loop {
            let mut guard = futures::ready!(self_mut.0.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().read(buf.initialize_unfilled())) {
                Ok(_result) => return Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncTun {
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
