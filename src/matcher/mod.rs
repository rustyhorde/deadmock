// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` configuration

use crate::error::Result;
use crate::mapping::{Mapping, Mappings, Request};
use http::header::{HeaderName, HeaderValue};
use http::Request as HttpRequest;

#[derive(Debug)]
pub struct Matcher {}

impl Matcher {
    pub fn get_match(&self, request: &HttpRequest<()>, mappings: &Mappings) -> Result<Mapping> {
        mappings
            .mappings()
            .iter()
            .filter_map(|(_uuid, mapping)| self.is_match(request, mapping))
            .max()
            .ok_or_else(|| "No mapping found".into())
    }

    fn is_match(&self, request: &HttpRequest<()>, mapping: &Mapping) -> Option<Mapping> {
        let mut matches: Vec<bool> = Vec::new();
        let request_config = mapping.request();

        self.is_exact_match_method(&mut matches, request, request_config);
        self.is_exact_match_url(&mut matches, request, request_config);
        self.is_exact_match_all_headers(&mut matches, request, request_config);

        if matches.iter().all(|v| *v) {
            Some(mapping.clone())
        } else {
            None
        }
    }

    fn is_exact_match_method(
        &self,
        matches: &mut Vec<bool>,
        request: &HttpRequest<()>,
        request_config: &Request,
    ) {
        if let Some(method) = request_config.method() {
            matches.push(request.method().as_str() == &method[..]);
        }
    }

    fn is_exact_match_url(
        &self,
        matches: &mut Vec<bool>,
        request: &HttpRequest<()>,
        request_config: &Request,
    ) {
        if let Some(url) = request_config.url() {
            matches.push(request.uri().path() == url);
        }
    }

    fn is_exact_match_all_headers(
        &self,
        matches: &mut Vec<bool>,
        request: &HttpRequest<()>,
        request_config: &Request,
    ) {
        if let Some(headers) = request_config.headers() {
            let mut found = false;
            'outer: for header in headers {
                if let Ok(match_name) = HeaderName::from_bytes(header.key().as_bytes()) {
                    if let Ok(match_value) = HeaderValue::from_bytes(header.value().as_bytes()) {
                        for (k, v) in request.headers() {
                            if match_name == k && match_value == v {
                                found = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
            matches.push(found);
        }
    }
}
