// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` configuration
use cached::UnboundCache;
use crate::error::Result;
use crate::util;
use futures::{future, Future};
use http::{Request as HttpRequest, Response as HttpResponse, StatusCode};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use tokio::net::TcpStream;
use tokio::prelude::{AsyncReadExt, AsyncWriteExt};
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
        request: HttpRequest<()>,
    ) -> Box<Future<Item = HttpResponse<String>, Error = String> + Send> {
        if let Some(proxy_base_url) = self.response.proxy_base_url() {
            let _full_url = format!("{}/{}", proxy_base_url, request.uri());
            let mut addrs = ("ecsb-test.kroger.com", 80).to_socket_addrs().expect("Unable to generate SocketAddr");

            if let Some(addr) = addrs.next() {
                tokio::spawn_async(async move {
                    match await!(run_client(&addr)) {
                        Ok(_) => {},
                        Err(e) => eprintln!("grrrrr: {}", e),
                    }
                });
            }

            Box::new(futures::future::ok(HttpResponse::new("testing".to_string())))
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

const MESSAGES: &[&str] = &[
    "GET /mobilecheckout/healthcheck HTTP/1.1\r\nHost: ecsb-test.kroger.com\r\nProxy-Authorization: Basic a29uODExNjpLZW56aWUyMkVsbGll\r\n\r\n",
];

async fn run_client(addr: &SocketAddr) -> std::io::Result<()> {
    println!("Connecting to {}", addr.to_string());
    let mut stream = await!(TcpStream::connect(addr))?;

    // Buffer to read into
    let mut buf = [0; 100000];

    for msg in MESSAGES {
        println!(" > write = {:?}", msg);

        // Write the message to the server
        await!(stream.write_all_async(msg.as_bytes()))?;

        // Read the message back from the server
        let read = await!(stream.read_async(&mut buf))?;

        println!(" > read = {} bytes", read);
        let result = String::from_utf8_lossy(&buf[..read]);
        println!(" > result = {}", result);
    }

    Ok(())
}