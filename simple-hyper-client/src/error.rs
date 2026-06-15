/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::Method;

pub type HyperClientError = hyper_util::client::legacy::Error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    Client(#[from] HyperClientError),

    #[error("{0} requests are not allowed to have a body")]
    BodyNotAllowed(Method),
}
