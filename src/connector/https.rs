/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::connector::http::{get_host, ConnectError, HttpConnection, HttpConnector};
use crate::connector::{NetworkConnection, NetworkConnector};

use hyper::client::connect::{Connected, Connection};
use hyper::Uri;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, TlsStream};

use std::error::Error as StdError;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An HTTPS connector using native-tls.
///
/// TLS use is enforced by default. To allow plain `http` URIs call
/// [`fn allow_http_scheme()`].
pub struct HttpsConnector {
    force_tls: bool,
    tls: TlsConnector,
}

impl HttpsConnector {
    pub fn new(tls: TlsConnector) -> Self {
        HttpsConnector {
            tls,
            force_tls: true,
        }
    }

    /// If called, the connector will allow URIs with the `http` scheme.
    /// Otherwise only URIs with the `https` scheme are allowed.
    pub fn allow_http_scheme(mut self) -> Self {
        self.force_tls = false;
        self
    }

    async fn connect(
        uri: Uri,
        tls: TlsConnector,
        force_tls: bool,
    ) -> Result<HttpOrHttpsConnection, ConnectError> {
        let is_https = uri.scheme_str() == Some("https");
        if !is_https && force_tls {
            return Err(ConnectError::new("invalid URI: expected `https` scheme"));
        }
        let host = get_host(&uri)?.to_owned();
        let http = HttpConnector::connect(uri, true).await?;
        if is_https {
            let tls = tls
                .connect(&host, http.stream)
                .await
                .map_err(|e| ConnectError::new("TLS error").cause(e))?;

            Ok(HttpOrHttpsConnection::Https(tls))
        } else {
            Ok(HttpOrHttpsConnection::Http(http))
        }
    }
}

impl NetworkConnector for HttpsConnector {
    fn connect(
        &self,
        uri: Uri,
    ) -> Pin<
        Box<dyn Future<Output = Result<NetworkConnection, Box<dyn StdError + Send + Sync>>> + Send>,
    > {
        let tls = self.tls.clone();
        let force_tls = self.force_tls;
        Box::pin(async move {
            match HttpsConnector::connect(uri, tls, force_tls).await {
                Ok(conn) => Ok(NetworkConnection::new(conn)),
                Err(e) => Err(Box::new(e) as _),
            }
        })
    }
}

/// An HTTP or HTTPS connection
pub enum HttpOrHttpsConnection {
    Http(HttpConnection),
    Https(TlsStream<TcpStream>),
}

impl Connection for HttpOrHttpsConnection {
    fn connected(&self) -> Connected {
        // TODO: provide remote address
        // TODO: provide information about http protocol version (if negotiated through
        // ALPN)
        Connected::new()
    }
}

impl AsyncRead for HttpOrHttpsConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match Pin::get_mut(self) {
            HttpOrHttpsConnection::Http(s) => Pin::new(s).poll_read(cx, buf),
            HttpOrHttpsConnection::Https(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for HttpOrHttpsConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match Pin::get_mut(self) {
            HttpOrHttpsConnection::Http(s) => Pin::new(s).poll_write(cx, buf),
            HttpOrHttpsConnection::Https(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match Pin::get_mut(self) {
            HttpOrHttpsConnection::Http(s) => Pin::new(s).poll_flush(cx),
            HttpOrHttpsConnection::Https(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match Pin::get_mut(self) {
            HttpOrHttpsConnection::Http(s) => Pin::new(s).poll_shutdown(cx),
            HttpOrHttpsConnection::Https(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}
