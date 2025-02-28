/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::client::connect::{Connected, Connection};
use hyper::service::Service;
use hyper::Uri;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use std::error::Error as StdError;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub mod http;
pub mod hyper_adapter;

pub use self::http::{ConnectError, HttpConnection, HttpConnector};
pub use self::hyper_adapter::HyperConnectorAdapter;

trait NetworkStream: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static {}

impl<T> NetworkStream for T where T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static {}

/// A boxed network connection
pub struct NetworkConnection(Box<dyn NetworkStream>);

impl NetworkConnection {
    pub fn new<S>(stream: S) -> Self
    where
        S: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    {
        NetworkConnection(Box::new(stream))
    }
}

impl Connection for NetworkConnection {
    fn connected(&self) -> Connected {
        self.0.connected()
    }
}

impl AsyncRead for NetworkConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for NetworkConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}

/// Network connector trait with type erasure
pub trait NetworkConnector: Send + Sync + 'static {
    fn connect(
        &self,
        uri: Uri,
    ) -> Pin<
        Box<dyn Future<Output = Result<NetworkConnection, Box<dyn StdError + Send + Sync>>> + Send>,
    >;
}

#[derive(Clone)]
pub(crate) struct ConnectorAdapter(Arc<dyn NetworkConnector>);

impl ConnectorAdapter {
    pub fn new<T: NetworkConnector>(connector: T) -> Self {
        Self(Arc::new(connector))
    }
}

impl Service<Uri> for ConnectorAdapter {
    type Response = NetworkConnection;
    type Error = Box<dyn StdError + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        self.0.connect(uri)
    }
}
