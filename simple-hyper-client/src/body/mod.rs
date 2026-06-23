/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use self::shared::{SharedBody, SharedBuf};

use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Body, Buf, Bytes, Frame, SizeHint};

use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

pub(crate) mod shared;

/// This is an implementation of `hyper::body::Body` for use with HTTP
/// `Request`s.
///
/// This can be constructed from `Arc<Vec<u8>>`, allowing the data to be
/// shared. The type can also wrap arbitrary other `hyper::body::Body`
/// instances, as long as they have `Bytes` as their `Data` associated type.
pub struct RequestBody(InnerBody);

enum InnerBody {
    Shared(SharedBody),
    Wrapped(BoxBody<Bytes, Box<dyn Error + Send + Sync>>),
}

impl RequestBody {
    /// Create an empty request body.
    pub fn empty() -> Self {
        SharedBody::empty().into()
    }

    /// Create a `RequestBody` from an arbitrary other `hyper::body::Body`.
    pub fn wrap<B, E>(body: B) -> Self
    where
        B: Body<Data = Bytes, Error = E> + Send + Sync + 'static,
        E: Error + Send + Sync + 'static,
    {
        // Note: we do not support wrapping bodies with arbitrary `Buf`s as their
        // `Data` associated type, because that would require us to call
        // `BodyExt::map_frame()`, which returns a body type that does not provide
        // accurate size hints, which can affect the transfer-encoding and
        // content-length headers sent along with the request.
        Self(InnerBody::Wrapped(
            body.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
                .boxed(),
        ))
    }
}

impl Default for RequestBody {
    /// Returns `Self::empty()`.
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Into<SharedBody>> From<T> for RequestBody {
    fn from(shared_body: T) -> Self {
        Self(InnerBody::Shared(shared_body.into()))
    }
}

impl Body for RequestBody {
    type Data = Buffer;
    type Error = Box<dyn Error + Send + Sync>;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match &mut self.get_mut().0 {
            InnerBody::Shared(shared_body) => SharedBody::poll_frame(Pin::new(shared_body), cx)
                .map(|opt| {
                    opt.map(|res| {
                        res.map(|frame| frame.map_data(|bytes| Buffer::Shared(bytes)))
                            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
                    })
                }),

            InnerBody::Wrapped(box_body) => {
                BoxBody::poll_frame(Pin::new(box_body), cx).map(|opt| {
                    opt.map(|res| res.map(|frame| frame.map_data(|bytes| Buffer::Wrapped(bytes))))
                })
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match &self.0 {
            InnerBody::Shared(shared_body) => shared_body.is_end_stream(),
            InnerBody::Wrapped(box_body) => box_body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.0 {
            InnerBody::Shared(shared_body) => shared_body.size_hint(),
            InnerBody::Wrapped(box_body) => box_body.size_hint(),
        }
    }
}

/// The `hyper::body::Body::Data` type for a [`RequestBody`].
pub enum Buffer {
    Shared(SharedBuf),
    Wrapped(Bytes),
}

impl Buf for Buffer {
    fn remaining(&self) -> usize {
        match self {
            Self::Shared(shared_buf) => shared_buf.remaining(),
            Self::Wrapped(bytes) => bytes.remaining(),
        }
    }

    fn chunk(&self) -> &[u8] {
        match self {
            Self::Shared(shared_buf) => shared_buf.chunk(),
            Self::Wrapped(bytes) => bytes.chunk(),
        }
    }

    fn advance(&mut self, cnt: usize) {
        match self {
            Self::Shared(shared_buf) => shared_buf.advance(cnt),
            Self::Wrapped(bytes) => bytes.advance(cnt),
        }
    }
}
