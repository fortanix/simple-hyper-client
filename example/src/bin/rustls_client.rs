/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
**/

use example::create_rustls_connector;
use simple_hyper_client::blocking::Client;
use std::error::Error;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let connector = create_rustls_connector()?;

    let client = Client::with_connector(connector);
    let uri = "https://api.restful-api.dev/objects";
    println!("GET {uri}");
    let req = client.get(uri)?.send()?;
    println!("{:#?}", req);
    let mut body_str = String::new();
    req.into_body().read_to_string(&mut body_str)?;
    println!("HTTP body:");
    println!("{}", body_str);
    Ok(())
}
