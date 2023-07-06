/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::errors::Error;
use hyper::{client::conn, Body};
use tokio::net::TcpStream;

use salvo::prelude::{Request, Response};

pub struct Proxy {
    pub address: String,
}

impl Proxy {
    pub fn create(url: &str) -> Result<Proxy, Error> {
        match url.parse::<hyper::Uri>() {
            Ok(url) if url.host().is_none() => {
                Err(Error::InvalidURLForProxy("Empty host".to_string()))
            }
            Ok(url) => Ok(Proxy {
                address: format!(
                    "{}:{}",
                    url.host().unwrap().to_string(),
                    url.port_u16().unwrap_or(80)
                ),
            }),
            Err(e) => Err(Error::InvalidURLForProxy(e.to_string())),
        }
    }

    pub async fn handle(&self, req: &mut Request, res: &mut Response) -> Result<(), Error> {
        let stream = TcpStream::connect(&self.address).await?;

        let (mut sender, connection) = conn::handshake(stream).await?;

        tokio::spawn(async move { connection.await });

        let mut proxied_request = http::Request::builder().uri(req.uri());
        for (key, value) in req.headers().iter() {
            proxied_request = proxied_request.header(key, value);
        }
        let proxied_request = proxied_request.method(req.method());
        let proxied_request =
            proxied_request.body(req.take_body().or(Some(Body::from(""))).unwrap())?;
        let response = sender.send_request(proxied_request).await?;

        let (
            salvo_core::http::response::Parts {
                status,
                // version,
                headers,
                // extensions,
                ..
            },
            body,
        ) = response.into_parts();

        res.set_status_code(status);
        res.set_headers(headers);
        res.set_body(body.into());

        Ok(())
    }
}
