/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::body::{Body, Buf, Frame, SizeHint};

use std::cmp;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// This is an implementation of `hyper::body::Body` for use with HTTP
/// `Request`s.
///
/// This can be constructed from `Arc<Vec<u8>>`, allowing the data to be
/// shared. It also implements `Clone` and `AsRef<[u8]>`, and it provides a
/// method to get its length and implements.
#[derive(Clone, Default)]
pub struct SharedBody(Option<SharedBytes>);

impl SharedBody {
    /// Create an empty request body.
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the length of the body in bytes.
    pub fn len(&self) -> usize {
        self.0.as_ref().map(SharedBytes::len).unwrap_or_default()
    }
}

impl From<Arc<Vec<u8>>> for SharedBody {
    fn from(arc: Arc<Vec<u8>>) -> Self {
        Self(Some(SharedBytes::Arc(arc)))
    }
}

impl From<Vec<u8>> for SharedBody {
    fn from(vec: Vec<u8>) -> Self {
        Self(Some(SharedBytes::Arc(Arc::new(vec))))
    }
}

impl From<String> for SharedBody {
    fn from(s: String) -> Self {
        Self(Some(SharedBytes::Arc(Arc::new(s.into_bytes()))))
    }
}

impl From<&'static [u8]> for SharedBody {
    fn from(slice: &'static [u8]) -> Self {
        Self(Some(SharedBytes::Static(slice)))
    }
}

impl From<&'static str> for SharedBody {
    fn from(s: &'static str) -> Self {
        Self(Some(SharedBytes::Static(s.as_bytes())))
    }
}

impl AsRef<[u8]> for SharedBody {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().map(SharedBytes::as_ref).unwrap_or(&[])
    }
}

impl Body for SharedBody {
    type Data = SharedBuf;
    type Error = io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let opt = self
            .get_mut()
            .0
            .take()
            .map(|bytes| SharedBuf { bytes, pos: 0 })
            .map(Frame::data)
            .map(Ok);
        Poll::Ready(opt)
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        let len = self.0.as_ref().map(SharedBytes::len).unwrap_or_default();
        SizeHint::with_exact(len as u64)
    }
}

pub struct SharedBuf {
    bytes: SharedBytes,
    pos: usize,
}

impl SharedBuf {
    pub fn len(&self) -> usize {
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

#[derive(Clone)]
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

impl AsRef<[u8]> for SharedBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Arc(vec) => vec,
            Self::Static(slice) => slice,
        }
    }
}
