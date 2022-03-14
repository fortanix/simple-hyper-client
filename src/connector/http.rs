/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::connector::{NetworkConnection, NetworkConnector};

use hyper::client::connect::{Connected, Connection};
use hyper::Uri;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

use std::error::Error as StdError;
use std::future::Future;
use std::net::Ipv6Addr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::{fmt, io};

const DEFAULT_HTTP_PORT: u16 = 80;
const DEFAULT_HTTPS_PORT: u16 = 443;

/// A simple HTTP connector
///
/// NOTE: this provides less functionality than [hyper's `HttpConnector`].
///
/// [hyper's `HttpConnector`]: https://docs.rs/hyper/0.14/hyper/client/struct.HttpConnector.html
#[derive(Clone)]
pub struct HttpConnector {
    _private: (),
}

impl HttpConnector {
    pub fn new() -> Self {
        HttpConnector { _private: () }
    }

    pub(super) async fn connect(
        uri: Uri,
        allow_https: bool,
    ) -> Result<HttpConnection, ConnectError> {
        match uri.scheme_str() {
            Some("http") => {}
            Some("https") if allow_https => {}
            Some(_) => {
                return Err(ConnectError::new(if allow_https {
                    "invalid URI: expected `http` or `https` scheme"
                } else {
                    "invalid URI: expected `http` scheme"
                }))
            }
            None => return Err(ConnectError::new("invalid URI: missing scheme")),
        }
        let host = get_host(&uri)?;
        let port = uri.port_u16().unwrap_or_else(|| {
            if uri.scheme_str() == Some("http") {
                DEFAULT_HTTP_PORT
            } else {
                DEFAULT_HTTPS_PORT
            }
        });
        let stream = TcpStream::connect((host, port))
            .await
            .map_err(|e| ConnectError::new("I/O error").cause(e))?;

        Ok(HttpConnection { stream })
    }
}

impl NetworkConnector for HttpConnector {
    fn connect(
        &self,
        uri: Uri,
    ) -> Pin<
        Box<dyn Future<Output = Result<NetworkConnection, Box<dyn StdError + Send + Sync>>> + Send>,
    > {
        Box::pin(async move {
            match Self::connect(uri, false).await {
                Ok(conn) => Ok(NetworkConnection::new(conn)),
                Err(e) => Err(Box::new(e) as _),
            }
        })
    }
}

pub(super) fn get_host(uri: &Uri) -> Result<&str, ConnectError> {
    let host = uri
        .host()
        .ok_or(ConnectError::new("invalid URI: missing host"))?;

    if host.starts_with("[") && host.ends_with("]") {
        let maybe_ipv6 = host.strip_prefix('[').unwrap().strip_suffix(']').unwrap();
        if let Ok(_) = Ipv6Addr::from_str(maybe_ipv6) {
            return Ok(maybe_ipv6);
        }
    }
    Ok(host)
}

/// A wrapper around [`tokio::net::TcpStream`]
///
/// [`tokio::net::TcpStream`]: https://docs.rs/tokio/1.0/tokio/net/struct.TcpStream.html
pub struct HttpConnection {
    pub(super) stream: TcpStream,
}

impl Connection for HttpConnection {
    fn connected(&self) -> Connected {
        // TODO: provide remote address
        Connected::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_host_correctness() {
        assert_eq!(
            get_host(&Uri::from_static("http://example.com")).ok(),
            Some("example.com")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://1.2.3.4:80/test")).ok(),
            Some("1.2.3.4")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://[::1]")).ok(),
            Some("::1")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://[::1.2.3.4]:8080")).ok(),
            Some("::1.2.3.4")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://[test.com]")).ok(),
            Some("[test.com]")
        );
    }
}
