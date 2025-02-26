/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    connector::http::{DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT},
    ConnectError, HttpConnection,
};
use hyper::Uri;
use std::io;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

pub async fn connect(
    uri: Uri,
    allow_https: bool,
    connect_timeout: Option<Duration>,
) -> Result<HttpConnection, ConnectError> {
    match uri.scheme_str() {
        Some("http") => {}
        Some("https") if allow_https => {}
        Some(_) => {
            return Err(ConnectError::new(if allow_https {
                "invalid URI: expected `http` or `https` scheme"
            } else {
                "invalid URI: expected `http` scheme"
            }))
        }
        None => return Err(ConnectError::new("invalid URI: missing scheme")),
    }
    let host = get_host(&uri)?;
    let port = uri.port_u16().unwrap_or_else(|| {
        if uri.scheme_str() == Some("http") {
            DEFAULT_HTTP_PORT
        } else {
            DEFAULT_HTTPS_PORT
        }
    });
    let connect = TcpStream::connect((host, port));
    let stream = match connect_timeout {
        Some(duration) => match time::timeout(duration, connect).await {
            Ok(Ok(stream)) => Ok(stream),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "connection timed out",
            )),
        },
        None => connect.await,
    }
    .map_err(|e| ConnectError::new("I/O error").cause(e))?;

    Ok(HttpConnection { stream })
}

pub fn get_host(uri: &Uri) -> Result<&str, ConnectError> {
    let host = uri
        .host()
        .ok_or(ConnectError::new("invalid URI: missing host"))?;

    if host.starts_with("[") && host.ends_with("]") {
        let maybe_ipv6 = host.strip_prefix('[').unwrap().strip_suffix(']').unwrap();
        if let Ok(_) = Ipv6Addr::from_str(maybe_ipv6) {
            return Ok(maybe_ipv6);
        }
    }
    Ok(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_host_correctness() {
        assert_eq!(
            get_host(&Uri::from_static("http://example.com")).ok(),
            Some("example.com")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://1.2.3.4:80/test")).ok(),
            Some("1.2.3.4")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://[::1]")).ok(),
            Some("::1")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://[::1.2.3.4]:8080")).ok(),
            Some("::1.2.3.4")
        );
        assert_eq!(
            get_host(&Uri::from_static("http://[test.com]")).ok(),
            Some("[test.com]")
        );
    }
}
