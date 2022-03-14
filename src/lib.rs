/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod async_client;
pub mod blocking;
mod connector;
mod error;
mod shared_body;

pub use self::async_client::*;
pub use self::connector::{
    ConnectError, HttpConnection, HttpConnector, HyperConnectorAdapter, NetworkConnection,
    NetworkConnector,
};
#[cfg(feature = "tokio-native-tls")]
pub use self::connector::{HttpOrHttpsConnection, HttpsConnector};
pub use self::error::Error;
pub use self::shared_body::SharedBody;

pub use hyper::body::{aggregate, to_bytes, Buf, Bytes, HttpBody};
pub use hyper::{self, Method, StatusCode, Uri, Version};

pub type Request = hyper::Request<SharedBody>;
pub type Response = hyper::Response<hyper::Body>;
