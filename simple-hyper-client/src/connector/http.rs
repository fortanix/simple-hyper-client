/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::connector::{NetworkConnection, NetworkConnector};
use crate::connector_impl::connect;
use hyper::client::connect::{Connected, Connection};
use hyper::Uri;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{fmt, io};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

pub(crate) const DEFAULT_HTTP_PORT: u16 = 80;
pub(crate) const DEFAULT_HTTPS_PORT: u16 = 443;

/// A simple HTTP connector
///
/// NOTE: this provides less functionality than [hyper's `HttpConnector`].
///
/// [hyper's `HttpConnector`]: https://docs.rs/hyper/0.14/hyper/client/struct.HttpConnector.html
#[derive(Clone)]
pub struct HttpConnector {
    connect_timeout: Option<Duration>,
}

impl HttpConnector {
    pub fn new() -> Self {
        HttpConnector {
            connect_timeout: None,
        }
    }

    /// Set the connect timeout. Default is None.
    pub fn connect_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connect_timeout = timeout;
        self
    }
}

impl NetworkConnector for HttpConnector {
    fn connect(
        &self,
        uri: Uri,
    ) -> Pin<
        Box<dyn Future<Output = Result<NetworkConnection, Box<dyn StdError + Send + Sync>>> + Send>,
    > {
        let connect_timeout = self.connect_timeout;
        Box::pin(async move {
            match connect(uri, false, connect_timeout).await {
                Ok(conn) => Ok(NetworkConnection::new(conn)),
                Err(e) => Err(Box::new(e) as _),
            }
        })
    }
}

/// A wrapper around [`tokio::net::TcpStream`]
///
/// [`tokio::net::TcpStream`]: https://docs.rs/tokio/1.0/tokio/net/struct.TcpStream.html
pub struct HttpConnection {
    pub(crate) stream: TcpStream,
}

impl Connection for HttpConnection {
    fn connected(&self) -> Connected {
        // TODO(#13): provide remote address
        Connected::new()
    }
}

impl HttpConnection {
    pub fn into_tcp_stream(self) -> TcpStream {
        self.stream
    }
}

impl AsyncRead for HttpConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for HttpConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

pub struct ConnectError {
    msg: &'static str,
    cause: Option<Box<dyn StdError + Send + Sync>>,
}

impl ConnectError {
    pub fn new(msg: &'static str) -> Self {
        ConnectError { msg, cause: None }
    }

    pub fn cause<E: Into<Box<dyn StdError + Send + Sync>>>(mut self, cause: E) -> Self {
        self.cause = Some(cause.into());
        self
    }
}

impl fmt::Debug for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref cause) = self.cause {
            f.debug_tuple("ConnectError")
                .field(&self.msg)
                .field(cause)
                .finish()
        } else {
            self.msg.fmt(f)
        }
    }
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg)?;
        if let Some(ref cause) = self.cause {
            write!(f, ": {}", cause)?;
        }
        Ok(())
    }
}

impl StdError for ConnectError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause.as_ref().map(|e| &**e as _)
    }
}
