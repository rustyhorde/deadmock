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
use crate::http_types::header::{CONTENT_LENGTH, HeaderName, HeaderValue};
use crate::util;
use futures::{future, Future, Stream};
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
        request: &HttpRequest<()>,
    ) -> Box<Future<Item = HttpResponse<String>, Error = String> + Send> {
        if let Some(proxy_base_url) = self.response.proxy_base_url() {
            let _full_url = format!("{}/{}", proxy_base_url, request.uri());
            // let mut addrs = ("espn.go.com", 80).to_socket_addrs().expect("Unable to generate SocketAddr");
            let mut addrs = ("ecsb-test.kroger.com", 80).to_socket_addrs().expect("Unable to generate SocketAddr");
            let (tx, rx) = futures::sync::mpsc::unbounded();

            if let Some(addr) = addrs.next() {
                tokio::spawn_async(async move {
                    tx.unbounded_send(await!(run_client(&addr))).expect("Unable to send upstream response!");
                });
            }

            Box::new(rx.fold(String::new(), |mut buffer, res| {
                match res {
                    Ok(val) => buffer.push_str(&val),
                    Err(e) => buffer.push_str(&e.to_string()),
                }
                futures::future::ok(buffer)
            }).map_err(|_| "Error processing upstream response".to_string())
            .map(|res| HttpResponse::new(res)))
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
                Err(e) => util::error_response_fut(format!("{}", e), StatusCode::INTERNAL_SERVER_ERROR),
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
    // "GET / HTTP/1.1\r\nHost: www.espn.com\r\n\r\n",
];

async fn run_client(addr: &SocketAddr) -> std::io::Result<String> {
    let mut stream = await!(TcpStream::connect(addr))?;

    // Buffer to read into
    let mut buf = [0; 200_000];

    let mut total = Vec::new();
    for msg in MESSAGES {
        // Write the message to the server
        await!(stream.write_all_async(msg.as_bytes()))?;

        // Read the message back from the server
        'outer: loop {
            let read = await!(stream.read_async(&mut buf))?;

            if read == 0 {
                break
            }
            let mut headers = [None; 16];
            let (version, amt) = {
                let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
                let mut r = httparse::Response::new(&mut parsed_headers);
                let status = r.parse(&buf).map_err(|e| {
                    let msg = format!("failed to parse http response: {:?}", e);
                    std::io::Error::new(std::io::ErrorKind::Other, msg)
                })?;

                let amt = match status {
                    httparse::Status::Complete(amt) => amt,
                    httparse::Status::Partial => continue 'outer,
                };

                let toslice = |a: &[u8]| {
                    let start = a.as_ptr() as usize - buf.as_ptr() as usize;
                    assert!(start < buf.len());
                    (start, start + a.len())
                };

                for (i, header) in r.headers.iter().enumerate() {
                    let k = toslice(header.name.as_bytes());
                    let v = toslice(header.value);
                    headers[i] = Some((k, v));
                }

                (r.version.unwrap(), amt)
            };
            if version != 1 {
                return Err(std::io::Error::new( std::io::ErrorKind::Other, "only HTTP/1.1 accepted"));
            }

            let data = &buf[..amt];

            let mut content_length: usize = 0;
            for header in &headers {
                let (k, v) = match *header {
                    Some((ref k, ref v)) => (k, v),
                    None => break,
                };
                if let Ok(value) = HeaderValue::from_bytes(&data[v.0..v.1]) {
                    if let Ok(name) = HeaderName::from_bytes(&data[k.0..k.1]) {
                        if name == CONTENT_LENGTH {
                            content_length = value.to_str().unwrap().parse::<usize>().expect("unable to parse header value");
                        }
                    }
                }
            }

            let mut bytes_so_far = read - amt;
            total.extend_from_slice(&buf[amt..read]);

            while bytes_so_far < content_length {
                let read = await!(stream.read_async(&mut buf))?;

                bytes_so_far += read;
                total.extend_from_slice(&buf[..read]);
            }

            break 'outer
        }
    }

    let response = String::from_utf8_lossy(&total).into_owned();
    Ok(response)
}