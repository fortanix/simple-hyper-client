/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use futures_util::task::{waker, ArcWake};

use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::{self, Thread};

pub(crate) fn wait<F, T>(fut: F) -> T
where
    F: Future<Output = T>,
{
    let thread = ThreadWaker(thread::current());
    let waker = waker(Arc::new(thread));
    let mut cx = Context::from_waker(&waker);

    futures_util::pin_mut!(fut);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => (), // fallthrough
        }
        thread::park();
    }
}

struct ThreadWaker(Thread);

impl ArcWake for ThreadWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.0.unpark();
    }
}
