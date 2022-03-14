/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::connector::{NetworkConnection, NetworkConnector};

use hyper::client::connect::Connection;
use hyper::service::Service;
use hyper::Uri;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;

use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// An adapter that given `T: hyper::client::connect::Connect`
/// implements [`NetworkConnector`]
///
/// NOTE: this is only meant as a last resort, if you can directly
/// implement [`NetworkConnector`] for your connector then avoid
/// using this adapter to reduce allocations.
pub struct HyperConnectorAdapter<T>(Arc<Mutex<T>>);

impl<T> HyperConnectorAdapter<T> {
    pub fn new(inner: T) -> Self {
        HyperConnectorAdapter(Arc::new(Mutex::new(inner)))
    }
}

impl<S, T> NetworkConnector for HyperConnectorAdapter<S>
where
    S: Service<Uri, Response = T> + Send + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
    S::Future: Unpin + Send,
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    fn connect(
        &self,
        uri: Uri,
    ) -> Pin<
        Box<dyn Future<Output = Result<NetworkConnection, Box<dyn StdError + Send + Sync>>> + Send>,
    > {
        let inner = self.0.clone();
        Box::pin(async move {
            match inner.lock().await.call(uri).await {
                Ok(conn) => Ok(NetworkConnection::new(conn)),
                Err(e) => Err(e.into()),
            }
        })
    }
}
