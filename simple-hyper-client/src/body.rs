/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Body, Buf, Frame, SizeHint};

use std::cmp;
use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// This is an implementation of `hyper::body::Body` for use with HTTP
/// `Request`s.
///
/// This can be constructed from `Arc<Vec<u8>>`, allowing the data to be
/// shared. The type can also wrap arbitrary other `hyper::body::Body`
/// instances.
pub struct RequestBody(InnerBody);

enum InnerBody {
    Shared(Option<SharedBytes>),
    Wrapped(BoxBody<Box<dyn Buf + Send + Sync>, Box<dyn Error + Send + Sync>>),
}

impl RequestBody {
    /// Create an empty request body.
    pub fn empty() -> Self {
        Self(InnerBody::Shared(None))
    }

    /// Create a `RequestBody` from an arbitrary other `hyper::body::Body`.
    pub fn wrap<B, D, E>(body: B) -> Self
    where
        B: Body<Data = D, Error = E> + Send + Sync + 'static,
        D: Buf + Send + Sync + 'static,
        E: Error + Send + Sync + 'static,
    {
        Self(InnerBody::Wrapped(
            body.map_frame(|f| f.map_data(|d| Box::new(d) as Box<dyn Buf + Send + Sync>))
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
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

impl From<Arc<Vec<u8>>> for RequestBody {
    fn from(arc: Arc<Vec<u8>>) -> Self {
        Self(InnerBody::Shared(Some(SharedBytes::Arc(arc))))
    }
}

impl From<Vec<u8>> for RequestBody {
    fn from(vec: Vec<u8>) -> Self {
        Self(InnerBody::Shared(Some(SharedBytes::Arc(Arc::new(vec)))))
    }
}

impl From<String> for RequestBody {
    fn from(s: String) -> Self {
        Self(InnerBody::Shared(Some(SharedBytes::Arc(Arc::new(
            s.into_bytes(),
        )))))
    }
}

impl From<&'static [u8]> for RequestBody {
    fn from(slice: &'static [u8]) -> Self {
        Self(InnerBody::Shared(Some(SharedBytes::Static(slice))))
    }
}

impl From<&'static str> for RequestBody {
    fn from(s: &'static str) -> Self {
        Self(InnerBody::Shared(Some(SharedBytes::Static(s.as_bytes()))))
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
            InnerBody::Shared(maybe_bytes) => {
                let opt = maybe_bytes
                    .take()
                    .map(|bytes| Buffer::Shared(SharedBuf { bytes, pos: 0 }))
                    .map(Frame::data)
                    .map(Ok);
                Poll::Ready(opt)
            }

            InnerBody::Wrapped(box_body) => {
                BoxBody::poll_frame(Pin::new(box_body), cx).map(|opt| {
                    opt.map(|res| res.map(|frame| frame.map_data(|bytes| Buffer::Wrapped(bytes))))
                })
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match &self.0 {
            InnerBody::Shared(maybe_bytes) => maybe_bytes.is_none(),
            InnerBody::Wrapped(box_body) => box_body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.0 {
            InnerBody::Shared(maybe_bytes) => {
                let len = maybe_bytes
                    .as_ref()
                    .map(SharedBytes::len)
                    .unwrap_or_default();
                SizeHint::with_exact(len as u64)
            }
            InnerBody::Wrapped(box_body) => box_body.size_hint(),
        }
    }
}

/// The `hyper::body::Body::Data` type for a [`RequestBody`].
pub enum Buffer {
    Shared(SharedBuf),
    Wrapped(Box<dyn Buf + Send + Sync>),
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

pub struct SharedBuf {
    bytes: SharedBytes,
    pos: usize,
}

impl SharedBuf {
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl Buf for SharedBuf {
    fn remaining(&self) -> usize {
        self.len() - self.pos
    }

    fn chunk(&self) -> &[u8] {
        match self.bytes {
            SharedBytes::Arc(ref bytes) => &bytes[self.pos..],
            SharedBytes::Static(ref bytes) => &bytes[self.pos..],
        }
    }

    fn advance(&mut self, cnt: usize) {
        self.pos = cmp::min(self.len(), self.pos + cnt);
    }
}

enum SharedBytes {
    Arc(Arc<Vec<u8>>),
    Static(&'static [u8]),
}

impl SharedBytes {
    fn len(&self) -> usize {
        match self {
            Self::Arc(ref bytes) => bytes.len(),
            Self::Static(ref bytes) => bytes.len(),
        }
    }
}
