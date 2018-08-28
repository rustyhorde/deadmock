// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` configuration
use cached::UnboundCache;
use error::Result;
use futures::{self, future, Future, Stream};
use http_types::{Request as HttpRequest, Response as HttpResponse, StatusCode};
use serde_json;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tokio;
use util;
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

cached_key_result!{
    STATIC_RESPONSE: UnboundCache<String, String> = UnboundCache::new();
    Key = { filename.to_string() };
    fn load(filename: &str) -> ::std::result::Result<String, &str> = {
        let files_path = Path::new("examples").join("files");
        let mut buffer = String::new();
        let mut found = false;

        util::visit_dirs(&files_path, &mut |entry| -> Result<()> {
            if let Some(fname) = entry.path().file_name() {
                if fname.to_string_lossy() == filename {
                    let f = File::open(entry.path())?;
                    let mut reader = BufReader::new(f);
                    reader.read_to_string(&mut buffer)?;
                    found = true;
                }
            }
            Ok(())
        }).map_err(|_| "Body file not found!")?;

        if found {
            Ok(buffer)
        } else {
            Err("Body file not found!")
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

    pub fn http_response(
        &self,
        _request: &HttpRequest<()>,
    ) -> Box<Future<Item = HttpResponse<String>, Error = String> + Send> {
        if let Some(proxy_base_url) = self.response.proxy_base_url() {
            let (sink, stream) = futures::sync::mpsc::unbounded();
            let pbu_clone = proxy_base_url.clone();

            thread::spawn(move || {
                let parts: Vec<&str> = pbu_clone.split("://").collect();
                if let Ok(addrs) = util::resolve(parts[0], parts[1]) {
                    for socket_addr in addrs {
                        println!("Addr: {}", socket_addr);
                    }
                }

                let delay = Duration::from_secs(2);
                thread::sleep(delay);

                let work = futures::future::ok::<String, ()>("output".to_string());
                let proxy = work.then(move |result| sink.unbounded_send(result).map_err(|_| ()));
                tokio::run(proxy);
            });

            let mut buffer = Vec::new();
            let response = stream
                .fold(buffer, |mut buffer, res| {
                    if let Ok(response) = res {
                        buffer.extend_from_slice(response.as_bytes());
                    }
                    future::ok(buffer)
                }).map_err(|_| "blah".to_string())
                .map(move |res| {
                    let val = String::from_utf8_lossy(&res).into_owned();
                    util::response(val, StatusCode::OK)
                });

            Box::new(response)
        } else {
            let mut response = HttpResponse::builder();
            if let Some(headers) = self.response.headers() {
                for header in headers {
                    response.header(&header.key()[..], &header.value()[..]);
                }
            }

            if let Some(status) = self.response.status() {
                response.status(if let Ok(status) = StatusCode::from_u16(*status) {
                    status
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                });
            } else {
                response.status(StatusCode::OK);
            }

            let body = if let Some(body_file_name) = self.response.body_file_name() {
                match load(body_file_name) {
                    Ok(body) => body,
                    Err(e) => e.to_string(),
                }
            } else {
                "Unable to process body".to_string()
            };

            match response.body(body) {
                Ok(response) => Box::new(future::ok(response)),
                Err(e) => {
                    util::error_response_fut(format!("{}", e), StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
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
