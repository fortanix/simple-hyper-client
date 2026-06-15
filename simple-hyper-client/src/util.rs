/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http_body_util::BodyExt;
use hyper::body::{Body, Buf, Bytes};

/// Collects all of the data frames from this body into a [`Buf`].
///
/// As opposed to [`to_bytes()`], this function avoids copying the data
/// and is useful when you don't need a contiguous slice of data.
pub async fn aggregate<T>(body: T) -> Result<impl Buf, T::Error>
where
    T: Body,
{
    Ok(body.collect().await?.aggregate())
}

/// Collects all of the data frames from this body into [`Bytes`].
pub async fn to_bytes<T>(body: T) -> Result<Bytes, T::Error>
where
    T: Body,
{
    Ok(body.collect().await?.to_bytes())
}
