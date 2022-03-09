/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::client::KeepClientAlive;

use hyper::body::{Buf, Bytes};
use hyper::Body as HyperBody;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use std::future::Future;
use std::{fmt, io};

/// A body type for HTTP responses that implement `std::io::Read`
pub struct Body {
    pub(super) keep_client_alive: KeepClientAlive,
    bytes: Bytes,
    rx: mpsc::Receiver<io::Result<Bytes>>,
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl Body {
    pub(super) fn new(
        mut hyper_body: HyperBody,
    ) -> (impl Future<Output = ()> + Send + 'static, Self) {
        let (tx, rx) = mpsc::channel(1);
        let fut = async move {
            loop {
                tokio::select! {
                    _ = tx.closed() => {
                        break; // body has been dropped.
                    }
                    res = hyper_body.next() => {
                        let res = match res {
                            None => break, // EOF
                            Some(Ok(chunk)) if chunk.is_empty() => continue,
                            Some(Ok(chunk)) => Ok(chunk),
                            Some(Err(e)) => Err(io::Error::new(io::ErrorKind::Other, e)),
                        };
                        if let Err(_) = tx.send(res).await {
                            break; // body has been dropped.
                        }
                    }
                }
            }
        };
        let body = Body {
            keep_client_alive: KeepClientAlive::empty(),
            bytes: Bytes::new(),
            rx,
        };
        (fut, body)
    }
}

impl io::Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.bytes.is_empty() {
            match self.rx.blocking_recv() {
                Some(Ok(bytes)) => {
                    self.bytes = bytes;
                }
                Some(Err(e)) => return Err(e),
                None => return Ok(0),
            }
        }
        (&mut self.bytes).reader().read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Body as HyperBody;
    use std::future::Future;
    use std::io::{self, Read};
    use std::thread;
    use tokio::time::{self, Duration};

    fn run_future<F: Future<Output = ()> + Send + 'static>(fut: F) {
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(fut);
        });
    }

    #[test]
    fn single_chunk() {
        let body = HyperBody::from("hello, world!");
        let (fut, mut reader) = Body::new(body);
        run_future(fut);

        let mut bytes = Vec::<u8>::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"hello, world!");
    }

    #[test]
    fn multiple_chunks() {
        let (mut sender, body) = HyperBody::channel();
        let (fut, mut reader) = Body::new(body);

        run_future(async move {
            let h = tokio::spawn(fut);

            sender.send_data("hello".into()).await.unwrap();
            time::sleep(Duration::from_millis(10)).await;
            sender.send_data(", ".into()).await.unwrap();
            sender.send_data("world!".into()).await.unwrap();

            drop(sender);
            h.await.unwrap();
        });

        let mut bytes = Vec::<u8>::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"hello, world!");
    }

    #[test]
    fn with_empty_chunk() {
        let (mut sender, body) = HyperBody::channel();
        let (fut, mut reader) = Body::new(body);

        run_future(async move {
            let h = tokio::spawn(fut);

            sender.send_data("hello".into()).await.unwrap();
            time::sleep(Duration::from_millis(10)).await;
            sender.send_data("".into()).await.unwrap();
            sender.send_data(", world!".into()).await.unwrap();

            drop(sender);
            h.await.unwrap();
        });

        let mut bytes = Vec::<u8>::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"hello, world!");
    }

    #[test]
    fn hyper_error() {
        let chunks: Vec<Result<_, io::Error>> = vec![
            Ok("hello"),
            Ok(" "),
            Ok("world"),
            Err(io::ErrorKind::BrokenPipe.into()),
        ];
        let stream = futures_util::stream::iter(chunks);
        let body = HyperBody::wrap_stream(stream);
        let (fut, mut reader) = Body::new(body);

        run_future(fut);

        let mut bytes = Vec::<u8>::new();
        let err = reader.read_to_end(&mut bytes).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(bytes, b"hello world");

        let mut buf = [0u8; 8];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }
}
