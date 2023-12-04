/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::connector::{ConnectorAdapter, NetworkConnector};
use crate::error::Error;
use crate::shared_body::SharedBody;
use crate::Response;

use headers::{ContentLength, Header, HeaderMap, HeaderMapExt};
use hyper::{Client as HyperClient, Method, Request, Uri};

use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// A wrapper for [hyper's `Client` type] providing a simpler interface
///
/// Example usage:
/// ```ignore
/// let connector = HttpConnector::new();
/// let client = Client::with_connector(connector);
/// let response = client.get("http://example.com/")?.send().await?;
/// ```
///
/// [hyper's `Client` type]: https://docs.rs/hyper/latest/hyper/client/struct.Client.html
#[derive(Clone)]
pub struct Client {
    inner: Arc<HyperClient<ConnectorAdapter, SharedBody>>,
}

macro_rules! define_method_fn {
    (@internal $name:ident, $method:ident, $method_str:expr) => {
        #[doc = "Initiate a "]
        #[doc = $method_str]
        #[doc = " request with the specified URI."]
        ///
        /// Returns an error if `uri` is invalid.
        pub fn $name<U>(&self, uri: U) -> Result<RequestBuilder<'_>, Error>
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

    /// Create a new `Client` using the specified connector.
    pub fn with_connector<C: NetworkConnector>(connector: C) -> Self {
        ClientBuilder::new().build(connector)
    }

    /// This method can be used instead of [Client::request]
    /// if the caller already has a [Request].
    pub async fn send(&self, request: Request<SharedBody>) -> Result<Response, Error> {
        Ok(self.inner.request(request).await?)
    }

    /// Initiate a request with the specified method and URI.
    ///
    /// Returns an error if `uri` is invalid.
    pub fn request<U>(&self, method: Method, uri: U) -> Result<RequestBuilder<'_>, Error>
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

// NOTE: the default values are taken from https://docs.rs/hyper/0.13.10/hyper/client/struct.Builder.html
// NOTE: not all configurable aspects of hyper Client are exposed here.
/// A builder for [`Client`]
///
/// [`Client`]: struct.Client.html
#[derive(Clone)]
pub struct ClientBuilder {
    max_idle_per_host: usize,
    idle_timeout: Option<Duration>,
}

impl ClientBuilder {
    pub(crate) fn new() -> Self {
        ClientBuilder {
            max_idle_per_host: usize::MAX,
            idle_timeout: Some(Duration::from_secs(90)),
        }
    }

    /// Sets the maximum idle connection per host allowed in the pool.
    ///
    /// Default is usize::MAX (no limit).
    pub fn pool_max_idle_per_host(&mut self, max_idle: usize) -> &mut Self {
        self.max_idle_per_host = max_idle;
        self
    }

    /// Set an optional timeout for idle sockets being kept-alive.
    ///
    /// Pass `None` to disable timeout.
    ///
    /// Default is 90 seconds.
    pub fn pool_idle_timeout(&mut self, val: Option<Duration>) -> &mut Self {
        self.idle_timeout = val;
        self
    }

    /// Combine the configuration of this builder with a connector to create a
    /// `Client`.
    pub fn build<C: NetworkConnector>(&self, connector: C) -> Client {
        Client {
            inner: Arc::new(
                HyperClient::builder()
                    .pool_max_idle_per_host(self.max_idle_per_host)
                    .pool_idle_timeout(self.idle_timeout)
                    .executor(TokioExecutor)
                    .build(ConnectorAdapter::new(connector)),
            ),
        }
    }
}

pub(crate) struct RequestDetails {
    pub(crate) method: Method,
    pub(crate) uri: Uri,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Option<SharedBody>,
}

impl fmt::Debug for RequestDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestDetails")
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("headers", &self.headers.len())
            .field("body", &self.body.as_ref().map_or("None", |_| "Some(...)"))
            .finish()
    }
}

impl RequestDetails {
    pub fn new(method: Method, uri: Uri) -> Self {
        RequestDetails {
            method,
            uri,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    pub async fn send(self, client: &Client) -> Result<Response, Error> {
        let req = self.into_request()?;
        Ok(client.inner.request(req).await?)
    }

    pub fn into_request(mut self) -> Result<Request<SharedBody>, Error> {
        let can_have_body = match self.method {
            // See RFC 7231 section 4.3
            Method::GET | Method::HEAD | Method::DELETE => false,
            _ => true,
        };
        let body = match can_have_body {
            true => {
                let body = self.body.unwrap_or_else(|| SharedBody::empty());
                // NOTE: body cannot be chunked in this implementation, so we
                // don't worry about chunked encoding here. But if this changes
                // then we should not set `ContentLength` automatically if the
                // request body is chunked, see RFC 7230 section 3.3.2.
                self.headers.typed_insert(ContentLength(body.len() as u64));
                body
            }
            false if self.body.is_some() => return Err(Error::BodyNotAllowed(self.method)),
            false => SharedBody::empty(),
        };
        let mut req = Request::builder().method(self.method).uri(self.uri);
        match req.headers_mut() {
            Some(headers) => {
                *headers = self.headers;
            }
            // There is an error in req, but the only way to extract the error is through `req.body()`
            None => match req.body(SharedBody::empty()) {
                Err(e) => return Err(e.into()),
                Ok(_) => {
                    panic!("request builder must have errors if `fn headers_mut()` returns None")
                }
            },
        }

        Ok(req.body(body)?)
    }
}

/// An HTTP request builder
///
/// This is created through [`Client::get()`], [`Client::post()`] etc.
/// You need to call [`send()`] to actually send the request over the network.
/// If you don't want to send it and just want the resultant [Request], you
/// can call [RequestBuilder::build].
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

    /// Get the resultant [Request].
    ///
    /// Prefer [RequestBuilder::send] unless you have a specific
    /// need to get the resultant [Request].
    pub fn build(self) -> Result<Request<SharedBody>, Error> {
        self.details.into_request()
    }

    /// Send the request over the network.
    ///
    /// Returns an error before sending the request if there is something wrong
    /// with the request parameters (method, uri, etc.).
    pub async fn send(self) -> Result<Response, Error> {
        self.details.send(&self.client).await
    }
}

#[derive(Copy, Clone)]
pub(crate) struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::spawn(fut);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::HttpConnector;
    use headers::ContentType;
    use hyper::body::to_bytes;
    use hyper::StatusCode;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const RESPONSE_OK: &str = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!\r\n";
    const RESPONSE_404: &str =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 23\r\n\r\nResource was not found.\r\n";

    async fn test_http_server(resp: &'static str) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut input = Vec::new();
            stream.read(&mut input).await.unwrap();
            stream.write_all(resp.as_bytes()).await.unwrap();
        });
        addr
    }

    #[tokio::test]
    async fn http_client() {
        let addr = test_http_server(RESPONSE_OK).await;
        let url = format!("http://{}/", addr);

        let connector = HttpConnector::new();
        let client = Client::with_connector(connector);
        let response = client
            .post(url)
            .unwrap()
            .header(ContentType::json())
            .body(r#"{"key":"value"}"#)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response).await.unwrap();
        assert_eq!(body, "Hello, world!".as_bytes());
    }

    #[tokio::test]
    async fn drop_client_before_response() {
        let addr = test_http_server(RESPONSE_404).await;
        let url = format!("http://{}/", addr);

        let connector = HttpConnector::new();
        let client = Client::with_connector(connector);
        let response = client.get(url).unwrap().send().await.unwrap();
        drop(client);

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().len(), 1);
        let body = to_bytes(response).await.unwrap();
        assert_eq!(body, "Resource was not found.");
    }

    #[tokio::test]
    async fn http_connector_connect_timeout() {
        // IP address chosen from 192.0.2.0/24 block defined in RFC 5737.
        let url = "http://192.0.2.1/";
        let connector = HttpConnector::new().connect_timeout(Some(Duration::from_millis(100)));
        let client = Client::with_connector(connector);
        let err = client.get(url).unwrap().send().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            "error trying to connect: I/O error: connection timed out"
        );
    }
}
