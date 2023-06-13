/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use async_trait::async_trait;
use salvo::http::errors::StatusError;
use salvo::prelude::{Depot, Json, Request, Response, Writer};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid proxy URL: `{0}")]
    InvalidURLForProxy(String),

    #[error("IO error: `{0}`")]
    IOError(#[from] std::io::Error),

    #[error("Http error: `{0}`")]
    HttpError(#[from] http::Error),

    #[error("Hyper error: `{0}`")]
    HyperError(#[from] hyper::Error),
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
        }

        match self {
            Error::InvalidURLForProxy(_e) => panic!("We should not be here"),

            Error::IOError(e) => {
                res.set_status_error(StatusError::bad_request());
                res.render(Json(ErrorResponse {
                    error: e.to_string(),
                }));
            }

            Error::HttpError(e) => {
                res.set_status_error(StatusError::bad_request());
                res.render(Json(ErrorResponse {
                    error: e.to_string(),
                }));
            }

            Error::HyperError(e) => {
                res.set_status_error(StatusError::bad_request());
                res.render(Json(ErrorResponse {
                    error: e.to_string(),
                }));
            }
        }
    }
}
