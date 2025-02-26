/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use simple_hyper_client::{ConnectError, HttpConnection, NetworkConnection, NetworkConnector};

use simple_hyper_client::connector_impl;
use simple_hyper_client::hyper::client::connect::{Connected, Connection};
use simple_hyper_client::Uri;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector};

use std::convert::TryFrom;
use std::error::Error as StdError;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// An HTTPS connector using tokio-rustls.
///
/// TLS use is enforced by default. To allow plain `http` URIs call
/// [`fn allow_http_scheme()`].
pub struct HttpsConnector {
    force_tls: bool,
    tls: TlsConnector,
    connect_timeout: Option<Duration>,
}

impl HttpsConnector {
    pub fn new(tls: TlsConnector) -> Self {
        HttpsConnector {
            tls,
            force_tls: true,
            connect_timeout: None,
        }
    }

    /// Set the connect timeout. Default is None.
    pub fn connect_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connect_timeout = timeout;
        self
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
        connect_timeout: Option<Duration>,
    ) -> Result<HttpOrHttpsConnection, ConnectError> {
        let is_https = uri.scheme_str() == Some("https");
        if !is_https && force_tls {
            return Err(ConnectError::new("invalid URI: expected `https` scheme"));
        }
        let host = connector_impl::get_host(&uri)?.to_owned();
        let http = connector_impl::connect(uri, true, connect_timeout).await?;
        if is_https {
            let server_name = rustls_pki_types::ServerName::try_from(host)
                .map_err(|err| ConnectError::new("invalid host name").cause(err))?;
            let tls = tls
                .connect(server_name, http.into_tcp_stream())
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
        let connect_timeout = self.connect_timeout;
        Box::pin(async move {
            match HttpsConnector::connect(uri, tls, force_tls, connect_timeout).await {
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
        // TODO(#13): provide remote address
        // TODO(#14): provide information about http protocol version (if
        // negotiated through ALPN)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rustls_pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
    use simple_hyper_client::{to_bytes, Client};
    use simple_hyper_client::{StatusCode, Uri};
    use std::{convert::TryFrom, net::SocketAddr, sync::Arc};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        sync::oneshot,
        task::JoinHandle,
    };
    use tokio_rustls::{
        rustls::{ClientConfig, RootCertStore, ServerConfig},
        TlsAcceptor, TlsConnector,
    };

    const RESPONSE_OK: &str = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!\r\n";

    fn get_tls_connector() -> TlsConnector {
        let test_ca_bytes = include_bytes!("../../test-ca/ca.cert");
        let test_ca_der = CertificateDer::from_pem_slice(test_ca_bytes).unwrap();
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.add(test_ca_der).unwrap();
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        TlsConnector::from(Arc::new(config))
    }

    fn get_tls_acceptor() -> TlsAcceptor {
        let test_end_cert_bytes = include_bytes!("../../test-ca/end.fullchain");
        let test_end_cert = CertificateDer::pem_slice_iter(test_end_cert_bytes)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let test_end_key_bytes = include_bytes!("../../test-ca/end.key");
        let test_end_key = PrivateKeyDer::from_pem_slice(test_end_key_bytes).unwrap();
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(test_end_cert, test_end_key)
            .unwrap();
        TlsAcceptor::from(Arc::new(config))
    }

    async fn start_tls_server(
        resp: &'static str,
        mut shutdown_rev: oneshot::Receiver<()>,
    ) -> (JoinHandle<()>, SocketAddr) {
        let listener = TcpListener::bind("localhost:0")
            .await
            .expect("Failed to bind to localhost");
        let local_addr = listener.local_addr().unwrap();
        let acceptor = get_tls_acceptor();

        let server_handler = tokio::spawn(async move {
            println!("Started TLS server at {}", local_addr);
            loop {
                tokio::select! {
                    Ok((stream, peer_addr)) = listener.accept() => {
                        let acceptor = acceptor.clone();
                        tokio::spawn(async move {
                            match acceptor.accept(stream).await {
                                Ok(mut tls_stream) => {
                                    println!("TLS connection established with {}", peer_addr);
                                    let mut input = Vec::with_capacity(1024);
                                    let _n = tls_stream.read(&mut input).await.expect("failed to read");
                                    println!("Received: {}", String::from_utf8_lossy(&input));
                                    tls_stream.write_all(resp.as_bytes()).await.expect("failed to write");
                                }
                                Err(err) => eprintln!("TLS handshake failed: {}", err),
                            }
                        });
                    }

                    // Stop the server when a shutdown signal is received
                    _ = &mut shutdown_rev => {
                        println!("Shutting down TLS server at {} ...", local_addr);
                        break;
                    }
                }
            }
        });
        (server_handler, local_addr)
    }

    #[tokio::test]
    async fn test_connect_invalid_scheme() {
        let tls_connector = get_tls_connector();
        let uri = Uri::try_from("http://example.com").unwrap();

        let result = HttpsConnector::connect(uri, tls_connector, true, None).await;
        match result {
            Err(err) => assert!(
                err.to_string()
                    .contains("invalid URI: expected `https` scheme"),
                "{}",
                err
            ),
            Ok(_) => panic!("Expecting error for invalid URI scheme"),
        }
    }

    #[tokio::test]
    async fn test_connect() {
        let _ = env_logger::try_init();
        let tls_connector = get_tls_connector();

        let (tx, rx) = oneshot::channel::<()>();
        let (server_handler, addr) = start_tls_server(RESPONSE_OK, rx).await;

        // Note: localhost is important, since the cert we use have localhost in SAN
        let uri = Uri::try_from(format!("https://localhost:{}", addr.port())).unwrap();
        let connector = HttpsConnector::new(tls_connector);
        let client = Client::with_connector(connector);
        let response = client
            .post(uri)
            .unwrap()
            .body(r#"plain text"#)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response).await.unwrap();
        assert_eq!(body, "Hello, world!".as_bytes());
        tx.send(()).unwrap();
        server_handler.await.unwrap();
    }
}
