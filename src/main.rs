/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod action;
mod catchers;
mod condition;
mod config;
mod errors;
mod proxy;
mod routers;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let config = config::Config::create();
    server::run(&config).await
}
