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
mod util;

pub use self::async_client::*;
pub use self::connector::{
    ConnectError, HttpConnection, HttpConnector, HyperConnectorAdapter, NetworkConnection,
    NetworkConnector,
};
pub use self::error::{Error, HyperClientError};
pub use self::shared_body::SharedBody;
pub use self::util::{aggregate, to_bytes};

pub use hyper::body::{Body, Buf, Bytes, Incoming};
pub use hyper::{self, Method, StatusCode, Uri, Version};
pub use hyper_util::client::legacy::{Builder as HyperClientBuilder, Client as HyperClient};
pub use tower_service;

pub type Request = hyper::Request<SharedBody>;
pub type Response = hyper::Response<Incoming>;

pub mod compat {
    pub use hyper_util::client::legacy::connect::{Connected, Connection};
}

#[doc(hidden)]
pub mod connector_impl;
