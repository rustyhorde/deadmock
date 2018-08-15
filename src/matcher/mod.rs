// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` configuration
use http_types::Request as HttpRequest;
use std::collections::HashMap;
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
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, Hash, PartialEq, Serialize)]
pub struct Matcher {
    priority: u8,
    request: Option<Request>,
    response: Option<Response>,
}

impl Matcher {
    pub fn has_match(&self, _request: &HttpRequest<()>) -> bool {
        false
    }
}
