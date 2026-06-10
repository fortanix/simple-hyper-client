/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::client::KeepClientAlive;

use futures_util::StreamExt;
use http_body_util::BodyExt;
use hyper::body::Body as HyperBody;
use hyper::body::{Buf, Bytes};
use tokio::sync::mpsc;

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
    pub(super) fn new<T>(hyper_body: T) -> (impl Future<Output = ()> + Send + 'static, Self)
    where
        T: HyperBody + Send + 'static,
        T::Data: Send + Into<Bytes>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel(1);

        let fut = async move {
            let mut stream = std::pin::pin!(hyper_body.into_data_stream());

            loop {
                tokio::select! {
                    _ = tx.closed() => {
                        break; // body has been dropped.
                    }

                    res = stream.next() => {
                        let res = match res {
                            None => break, // EOF

                            Some(Ok(chunk)) => {
                                let chunk = chunk.into();

                                if chunk.is_empty() {
                                    continue;
                                } else {
                                    Ok(chunk)
                                }
                            }

                            Some(Err(e)) => Err(io::Error::new(io::ErrorKind::Other, e)),
                        };

                        if tx.send(res).await.is_err() {
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
    use http_body_util::{Channel, Full, StreamBody};
    use hyper::body::Frame;
    use std::io::Read;
    use std::{future::Future, thread};
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
        let body = Full::new(&b"hello, world!"[..]);
        let (fut, mut reader) = Body::new(body);
        run_future(fut);

        let mut bytes = Vec::<u8>::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"hello, world!");
    }

    #[test]
    fn multiple_chunks() {
        let (mut sender, body) = Channel::<&[u8]>::new(5);
        let (fut, mut reader) = Body::new(body);

        run_future(async move {
            let h = tokio::spawn(fut);

            sender.send_data(b"hello").await.unwrap();
            time::sleep(Duration::from_millis(10)).await;
            sender.send_data(b", ").await.unwrap();
            sender.send_data(b"world!").await.unwrap();

            drop(sender);
            h.await.unwrap();
        });

        let mut bytes = Vec::<u8>::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"hello, world!");
    }

    #[test]
    fn with_empty_chunk() {
        let (mut sender, body) = Channel::<&[u8]>::new(5);
        let (fut, mut reader) = Body::new(body);

        run_future(async move {
            let h = tokio::spawn(fut);

            sender.send_data(b"hello").await.unwrap();
            time::sleep(Duration::from_millis(10)).await;
            sender.send_data(b"").await.unwrap();
            sender.send_data(b", world!").await.unwrap();

            drop(sender);
            h.await.unwrap();
        });

        let mut bytes = Vec::<u8>::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"hello, world!");
    }

    #[test]
    fn hyper_error() {
        eprintln!("test");
        let chunks: Vec<Result<&[u8], io::Error>> = vec![
            Ok(b"hello"),
            Ok(b" "),
            Ok(b"world"),
            Err(io::ErrorKind::BrokenPipe.into()),
        ];
        let chunks = chunks.into_iter().map(|res| res.map(Frame::data));
        let stream = futures_util::stream::iter(chunks);
        let body = StreamBody::new(stream);
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
