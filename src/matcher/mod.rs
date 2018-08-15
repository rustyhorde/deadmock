// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` configuration
use error::Result;
use http_types::Request as HttpRequest;
use serde_json;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use uuid::Uuid;

mod request;
mod response;

pub use self::request::Request;
pub use self::response::Response;

#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, MutGetters, PartialEq, Serialize)]
pub struct Mappings {
    #[get_mut = "pub"]
    mappings: HashMap<Uuid, Matcher>,
}

impl Mappings {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn add(&mut self, uuid: Uuid, matcher: Matcher) {
        self.mappings.insert(uuid, matcher);
    }

    pub fn get_match(&self, request: &HttpRequest<()>) -> Result<Matcher> {
        let mut matches = BTreeMap::new();

        for mapping in self.mappings.values() {
            if mapping.is_match(request) {
                matches.insert(mapping.priority(), mapping);
            }
        }

        if let Some((_k, v)) = matches.iter().next() {
            Ok((*v).clone())
        } else {
            Err("Not found!".into())
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, Hash, PartialEq, Serialize)]
pub struct Matcher {
    #[get = "pub"]
    priority: u8,
    #[get = "pub"]
    request: Request,
    #[get = "pub"]
    response: Response,
}

impl Matcher {
    pub fn is_match(&self, request: &HttpRequest<()>) -> bool {
        if let Some(url) = self.request.url() {
            request.uri().path() == url
        } else {
            false
        }
    }
}

impl fmt::Display for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = serde_json::to_string_pretty(self).map_err(|_| fmt::Error)?;
        writeln!(f);
        write!(f, "{}", out)
    }
}
