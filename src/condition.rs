/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use salvo::prelude::Request;
use salvo::routing::{Filter, PathState};
use std::fmt::{self, Formatter};

pub struct ConditionHeader {
    name: String,
    value: Option<String>,
}

impl ConditionHeader {
    pub fn new(name: &str, value: Option<&str>) -> ConditionHeader {
        tracing::info!(target: "ConditionHeader", name=name, value=value, "condition header created");
        ConditionHeader {
            name: name.to_string(),
            value: value.map(|s| s.into()),
        }
    }
}

impl Filter for ConditionHeader {
    fn filter(&self, req: &mut Request, _state: &mut PathState) -> bool {
        let header_value: Option<String> = req.header(&self.name);
        match header_value.as_ref() {
            None => false,
            Some(value) => match self.value.as_ref() {
                None => true,
                Some(v) => value == v,
            },
        }
    }
}

impl fmt::Debug for ConditionHeader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "header {} - value {:?}", self.name, self.value)
    }
}
