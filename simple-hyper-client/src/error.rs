/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use derive_more::{IsVariant, TryUnwrap, Unwrap};
use hyper::Method;
use std::error;

pub type HyperClientError = hyper_util::client::legacy::Error;

#[derive(Debug, thiserror::Error, IsVariant, TryUnwrap, Unwrap)]
#[try_unwrap(ref)]
#[unwrap(ref)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] http::Error),

    #[error("{0}{source}", source = render_source_error(.0))]
    Client(#[from] HyperClientError),

    #[error("{0} requests are not allowed to have a body")]
    BodyNotAllowed(Method),
}

fn render_source_error(err: &HyperClientError) -> String {
    if let Some(err) = error::Error::source(err) {
        format!(": {err}")
    } else {
        Default::default()
    }
}
