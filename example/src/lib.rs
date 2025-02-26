/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
**/

use simple_hyper_client::NetworkConnector;
use std::error::Error;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

pub fn create_native_tls_connector() -> Result<impl NetworkConnector, Box<dyn Error>> {
    let tls_connector = tokio_native_tls::native_tls::TlsConnector::new()?;
    Ok(simple_hyper_client_native_tls::HttpsConnector::new(
        tls_connector.into(),
    ))
}

pub fn create_rustls_connector() -> Result<impl NetworkConnector, Box<dyn Error>> {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let tls_connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(config));
    Ok(simple_hyper_client_rustls::HttpsConnector::new(
        tls_connector.into(),
    ))
}
