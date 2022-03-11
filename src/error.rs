/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::Method;

use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    Http(http::Error),
    Hyper(hyper::Error),
    BodyNotAllowed(Method),
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Error::Http(e)
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Http(ref e) => write!(f, "{}", e),
            Error::Hyper(ref e) => write!(f, "{}", e),
            Error::BodyNotAllowed(ref m) => {
                write!(f, "{} requests are not allowed to have a body", m)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Http(ref e) => Some(e),
            Error::Hyper(ref e) => Some(e),
            Error::BodyNotAllowed(_) => None,
        }
    }
}
