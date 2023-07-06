/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use salvo::catcher::Catcher;
use salvo::prelude::{Depot, Request, Response, StatusCode};

pub fn catchers() -> Vec<Box<dyn Catcher>> {
    vec![
        Box::new(Handle400),
        Box::new(Handle404),
        Box::new(Handle500),
    ]
}

struct Handle400;
impl Catcher for Handle400 {
    fn catch(&self, _req: &Request, _depot: &Depot, res: &mut Response) -> bool {
        if let Some(StatusCode::BAD_REQUEST) = res.status_code() {
            res.render("400 - Bad request");
            true
        } else {
            false
        }
    }
}

struct Handle404;
impl Catcher for Handle404 {
    fn catch(&self, _req: &Request, _depot: &Depot, res: &mut Response) -> bool {
        if let Some(StatusCode::NOT_FOUND) = res.status_code() {
            res.render("404 - Not found");
            true
        } else {
            false
        }
    }
}

struct Handle500;
impl Catcher for Handle500 {
    fn catch(&self, _req: &Request, _depot: &Depot, res: &mut Response) -> bool {
        if let Some(StatusCode::INTERNAL_SERVER_ERROR) = res.status_code() {
            res.render("500 - Internal error");
            true
        } else {
            false
        }
    }
}
