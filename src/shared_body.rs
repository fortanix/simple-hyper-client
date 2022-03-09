/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use headers::HeaderMap;
use hyper::body::{Buf, HttpBody};

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{cmp, io};

/// This is an alternative to `hyper::Body` for use with HTTP `Request`s
///
/// This can be constructed from `Arc<Vec<u8>>` while `hyper::Body` cannot.
/// Additionally this type provides a method to get its length.
pub struct SharedBody(Option<InnerBuf>);

enum InnerBuf {
    Arc(Arc<Vec<u8>>),
    Static(&'static [u8]),
}

impl SharedBody {
    pub fn len(&self) -> usize {
        match self.0.as_ref() {
            Some(InnerBuf::Arc(vec)) => vec.len(),
            Some(InnerBuf::Static(slice)) => slice.len(),
            None => 0,
        }
    }

    pub fn empty() -> Self {
        SharedBody(None)
    }
}

impl Default for SharedBody {
    /// Returns `SharedBody::empty()`.
    #[inline]
    fn default() -> Self {
        SharedBody::empty()
    }
}

impl From<Arc<Vec<u8>>> for SharedBody {
    fn from(arc: Arc<Vec<u8>>) -> Self {
        SharedBody(Some(InnerBuf::Arc(arc)))
    }
}

impl From<Vec<u8>> for SharedBody {
    fn from(vec: Vec<u8>) -> Self {
        SharedBody(Some(InnerBuf::Arc(Arc::new(vec))))
    }
}

impl From<String> for SharedBody {
    fn from(s: String) -> Self {
        SharedBody(Some(InnerBuf::Arc(Arc::new(s.into_bytes()))))
    }
}

impl From<&'static [u8]> for SharedBody {
    fn from(slice: &'static [u8]) -> Self {
        SharedBody(Some(InnerBuf::Static(slice)))
    }
}

impl From<&'static str> for SharedBody {
    fn from(s: &'static str) -> Self {
        SharedBody(Some(InnerBuf::Static(s.as_bytes())))
    }
}

impl HttpBody for SharedBody {
    type Data = SharedBuf;
    type Error = io::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let opt = self
            .get_mut()
            .0
            .take()
            .map(|bytes| SharedBuf { bytes, pos: 0 })
            .map(Ok);
        Poll::Ready(opt)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

pub struct SharedBuf {
    bytes: InnerBuf,
    pos: usize,
}

impl SharedBuf {
    fn len(&self) -> usize {
        match self.bytes {
            InnerBuf::Arc(ref bytes) => bytes.len(),
            InnerBuf::Static(ref bytes) => bytes.len(),
        }
    }
}

impl Buf for SharedBuf {
    fn remaining(&self) -> usize {
        self.len() - self.pos
    }

    fn chunk(&self) -> &[u8] {
        match self.bytes {
            InnerBuf::Arc(ref bytes) => &bytes[self.pos..],
            InnerBuf::Static(ref bytes) => &bytes[self.pos..],
        }
    }

    fn advance(&mut self, cnt: usize) {
        self.pos = cmp::min(self.len(), self.pos + cnt);
    }
}
