/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::body::Body;
use super::Response;
use crate::async_client::{ClientBuilder as AsyncClientBuilder, RequestDetails};
use crate::connector::NetworkConnector;
use crate::error::Error;
use crate::shared_body::SharedBody;

use futures_executor::block_on;
use headers::{Header, HeaderMap, HeaderMapExt};
use hyper::{Method, Uri};
use tokio::runtime;
use tokio::sync::{mpsc, oneshot};

use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// A wrapper for [hyper's `Client` type] providing a blocking interface
///
/// Example usage:
/// ```ignore
/// let connector = HttpConnector::new();
/// let client = Client::with_connector(connector);
/// let response = client.get("http://example.com/")?.send()?;
/// ```
///
/// [hyper's `Client` type]: https://docs.rs/hyper/latest/hyper/client/struct.Client.html
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

type ResponseSender = oneshot::Sender<Result<Response, Error>>;

struct ClientInner {
    tx: Option<mpsc::UnboundedSender<(RequestDetails, ResponseSender)>>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        // signal shutdown to the thread
        self.tx.take();
        self.thread.take().map(|h| h.join());
    }
}

macro_rules! define_method_fn {
    (@internal $name:ident, $method:ident, $method_str:expr) => {
        #[doc = "Initiate a "]
        #[doc = $method_str]
        #[doc = " request with the specified URI."]
        ///
        /// Returns an error if `uri` is invalid.
        pub fn $name<U>(&self, uri: U) -> Result<RequestBuilder, Error>
        where
            Uri: TryFrom<U>,
            <Uri as TryFrom<U>>::Error: Into<http::Error>,
        {
            self.request(Method::$method, uri)
        }
    };

    ($name:ident, $method:ident) => {
        define_method_fn!(@internal $name, $method, stringify!($method));
    };
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn with_connector<C: NetworkConnector>(connector: C) -> Self {
        ClientBuilder::new().build(connector)
    }

    /// Initiate a request with the specified method and URI.
    ///
    /// Returns an error if `uri` is invalid.
    pub fn request<U>(&self, method: Method, uri: U) -> Result<RequestBuilder, Error>
    where
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        let uri = uri.try_into().map_err(Into::into).map_err(Error::Http)?;
        Ok(RequestBuilder {
            client: self,
            details: RequestDetails::new(method, uri),
        })
    }

    define_method_fn!(get, GET);
    define_method_fn!(head, HEAD);
    define_method_fn!(post, POST);
    define_method_fn!(patch, PATCH);
    define_method_fn!(put, PUT);
    define_method_fn!(delete, DELETE);
}

/// A builder for [`Client`].
///
/// [`Client`]: struct.Client.html
#[derive(Clone)]
pub struct ClientBuilder(AsyncClientBuilder);

impl ClientBuilder {
    fn new() -> Self {
        ClientBuilder(AsyncClientBuilder::new())
    }

    /// Sets the maximum idle connection per host allowed in the pool.
    ///
    /// Default is usize::MAX (no limit).
    pub fn pool_max_idle_per_host(&mut self, max_idle: usize) -> &mut Self {
        self.0.pool_max_idle_per_host(max_idle);
        self
    }

    /// Set an optional timeout for idle sockets being kept-alive.
    ///
    /// Pass `None` to disable timeout.
    ///
    /// Default is 90 seconds.
    pub fn pool_idle_timeout(&mut self, val: Option<Duration>) -> &mut Self {
        self.0.pool_idle_timeout(val);
        self
    }

    /// Combine the configuration of this builder with a connector to create a
    /// `Client`.
    pub fn build<C: NetworkConnector>(&self, connector: C) -> Client {
        let async_client = self.0.build(connector);
        let (tx, mut rx) = mpsc::unbounded_channel::<(RequestDetails, ResponseSender)>();

        let thread = thread::spawn(move || {
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(); // TODO: send back an error through a oneshot channel

            rt.block_on(async move {
                while let Some((req_details, resp_tx)) = rx.recv().await {
                    let async_client = async_client.clone();
                    tokio::spawn(async move {
                        match req_details.send(&async_client).await {
                            Ok(resp) => {
                                let (parts, hyper_body) = resp.into_parts();
                                let (fut, body) = Body::new(hyper_body);
                                let _ = resp_tx.send(Ok(Response::from_parts(parts, body)));
                                fut.await;
                            }
                            Err(e) => {
                                let _: Result<_, _> = resp_tx.send(Err(e));
                            }
                        }
                    });
                }
            })
        });

        Client {
            inner: Arc::new(ClientInner {
                tx: Some(tx),
                thread: Some(thread),
            }),
        }
    }
}

/// An HTTP request builder
///
/// This is created through [`Client::get()`], [`Client::post()`] etc.
/// You need to call [`send()`] to actually send the request over the network.
///
/// [`Client::get()`]: struct.Client.html#method.get
/// [`Client::post()`]: struct.Client.html#method.post
/// [`send()`]: struct.RequestBuilder.html#method.send
pub struct RequestBuilder<'a> {
    client: &'a Client,
    details: RequestDetails,
}

impl<'a> RequestBuilder<'a> {
    /// Set the request body.
    pub fn body<B: Into<SharedBody>>(mut self, body: B) -> Self {
        self.details.body = Some(body.into());
        self
    }

    /// Set the request headers.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.details.headers = headers;
        self
    }

    /// Set a single header using [`HeaderMapExt::typed_insert()`].
    ///
    /// [`HeaderMapExt::typed_insert()`]: https://docs.rs/headers/0.3.5/headers/trait.HeaderMapExt.html#tymethod.typed_insert
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.details.headers.typed_insert(header);
        self
    }

    /// Send the request over the network.
    ///
    /// Returns an error before sending the request if there is something wrong
    /// with the request parameters (method, uri, etc.).
    pub fn send(self) -> Result<Response, Error> {
        let RequestBuilder { client, details } = self;
        let (tx, rx) = oneshot::channel();
        client
            .inner
            .tx
            .as_ref()
            .expect("runtime thread exited early")
            .send((details, tx))
            .expect("runtime thread panicked");

        // TODO: replace `block_on` with `rx.blocking_recv()` once we move to tokio 1.16+
        block_on(async move {
            match rx.await {
                Ok(res) => res,
                Err(_) => panic!("event loop panicked"),
            }
        })
        .map(|mut resp| {
            resp.body_mut().keep_client_alive = KeepClientAlive(Some(client.inner.clone()));
            resp
        })
    }
}

pub(super) struct KeepClientAlive(#[allow(unused)] Option<Arc<ClientInner>>);

impl KeepClientAlive {
    pub fn empty() -> Self {
        KeepClientAlive(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::HttpConnector;
    use headers::ContentType;
    use hyper::StatusCode;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener};
    use std::thread;

    const RESPONSE_OK: &str = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!\r\n";
    const RESPONSE_404: &str =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 23\r\n\r\nResource was not found.\r\n";

    fn test_http_server(resp: &'static str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut input = Vec::new();
            stream.read(&mut input).unwrap();
            stream.write_all(resp.as_bytes()).unwrap();
        });
        addr
    }

    #[test]
    fn http_client_ok() {
        let addr = test_http_server(RESPONSE_OK);
        let url = format!("http://{}/", addr);

        let connector = HttpConnector::new();
        let client = Client::with_connector(connector);
        let mut response = client
            .request(Method::POST, url)
            .unwrap()
            .header(ContentType::json())
            .body(r#"{"key":"value"}"#)
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let mut body = String::new();
        response.body_mut().read_to_string(&mut body).unwrap();
        assert_eq!(body, "Hello, world!");
    }

    #[test]
    fn drop_client_before_response() {
        let addr = test_http_server(RESPONSE_404);
        let url = format!("http://{}/", addr);

        let connector = HttpConnector::new();
        let client = Client::with_connector(connector);
        let mut response = client.get(url).unwrap().send().unwrap();
        drop(client);

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().len(), 1);
        let mut body = String::new();
        response.body_mut().read_to_string(&mut body).unwrap();
        assert_eq!(body, "Resource was not found.");
    }
}
