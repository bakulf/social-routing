/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::catchers;
use crate::config;
use crate::routers;
use salvo::logging::Logger;
use salvo::prelude::{Server, Service, TcpListener};

pub async fn run(config: &config::Config) {
    let service = Service::new(routers::routers(&config.rules).hoop(Logger))
        .with_catchers(catchers::catchers());

    tracing::info!(target: "Service", binding=config.server.bind, "binding the server");

    Server::new(TcpListener::bind(&config.server.bind))
        .serve(service)
        .await;
}
