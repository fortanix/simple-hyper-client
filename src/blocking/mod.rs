/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module provides a blocking interface on top of `hyper`'s HTTP client.
//!
//! The [`Client`] type in this module spawns a separate thread for running
//! async tasks. Additionally, since the client holds a connection pool
//! internally, it is advised that instances be reused as much as possible.

use crate::shared_body::SharedBody;

mod body;
mod client;

pub use self::body::Body;
pub use self::client::{Client, ClientBuilder, RequestBuilder};

pub type Request = hyper::Request<SharedBody>;
pub type Response = hyper::Response<Body>;
