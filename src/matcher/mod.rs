// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` configuration
use cached::UnboundCache;
use crate::config::ProxyConfig;
use crate::error::Result;
use crate::util;
use futures::{future, Future, Stream};
use http::{Request as HttpRequest, Response as HttpResponse, StatusCode};
use hyper::{Client, Request as HyperRequest};
use hyper::client::HttpConnector;
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_tls::HttpsConnector;
use slog::Logger;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use tokio::prelude::FutureExt;
use typed_headers::Credentials;
use uuid::Uuid;

mod request;
mod response;

pub use self::request::Request;
pub use self::response::Response;

#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Header {
    #[get = "pub"]
    key: String,
    #[get = "pub"]
    value: String,
}

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
    fn load(files_path: PathBuf, filename: &str) -> ::std::result::Result<String, &str> = {
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
        let mut matches: Vec<bool> = Vec::new();

        if let Some(method) = self.request.method() {
            matches.push(request.method().as_str() == &method[..]);
        }

        if let Some(url) = self.request.url() {
            matches.push(request.uri().path() == url);
        }

        matches.iter().all(|v| *v)
    }

    pub fn http_response(
        &self,
        request: &HttpRequest<()>,
        stdout: Option<Logger>,
        stderr: Option<Logger>,
        proxy_config: ProxyConfig,
        files_path: PathBuf,
    ) -> Box<Future<Item = HttpResponse<String>, Error = String> + Send> {
        if let Some(proxy_base_url) = self.response.proxy_base_url() {
            let full_url = format!("{}{}", proxy_base_url, request.uri());
            let (tx, rx) = futures::sync::mpsc::unbounded();
            let headers = self.response.additional_proxy_request_headers().clone();

            tokio::spawn_async(async move {
                if *proxy_config.use_proxy() {
                    if let Some(url_str) = proxy_config.proxy_url() {
                        let proxy_uri = url_str.parse().expect("Unable to parse proxy URI");
                        let mut proxy = Proxy::new(Intercept::All, proxy_uri);
                        if let Some(username) = proxy_config.proxy_username() {
                            if let Some(password) = proxy_config.proxy_password() {
                                if let Ok(creds) = Credentials::basic(username, password) {
                                    proxy.set_authorization(creds);
                                }
                            }
                        }

                        let connector = HttpConnector::new(4);
                        let proxy_connector = ProxyConnector::from_proxy(connector, proxy).expect("Unable to create proxy connector!");
                        let client = Client::builder().set_host(true).build::<_, hyper::Body>(proxy_connector);
                        await!(run_request(client, tx, full_url, stdout, stderr, headers));
                    } else {
                        panic!("Unable to determine proxy url!");
                    }
                } else if full_url.starts_with("https") {
                    let https_connector = HttpsConnector::new(4).expect("TLS initialization failed");
                    let client = Client::builder().set_host(true).build::<_, hyper::Body>(https_connector);
                    await!(run_request(client, tx, full_url, stdout, stderr, headers));
                } else {
                    let http_connector = HttpConnector::new(4);
                    let client = Client::builder().set_host(true).build::<_, hyper::Body>(http_connector);
                    await!(run_request(client, tx, full_url, stdout, stderr, headers));
                }
            });

            Box::new(rx.fold(String::new(), |mut buffer, res| {
                match res {
                    Ok(val) => buffer.push_str(&val),
                    Err(e) => buffer.push_str(&e.to_string()),
                }
                futures::future::ok(buffer)
            }).map_err(|_| "Error processing upstream response".to_string())
            .map(HttpResponse::new))
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
                match load(files_path, body_file_name) {
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

async fn run_request<C>(
    client: Client<C, hyper::Body>,
    tx: futures::sync::mpsc::UnboundedSender<std::result::Result<String, String>>,
    url: String,
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    headers: Option<Vec<Header>>,
) where C: hyper::client::connect::Connect + Sync + 'static {
    let response = await!({
        try_trace!(stdout, "Making request to {}", url);
        let mut request_builder = HyperRequest::get(url);

        if let Some(headers) = headers {
            for header in headers {
                request_builder.header(&header.key()[..], &header.value()[..]);
            }
        }
        let body = request_builder.body(hyper::Body::empty()).expect("Unable to create upstream request");
        client.request(body).timeout(std::time::Duration::from_secs(10))
    }).expect("Unable to process upstream response");

    let body = await!({
        response.into_body().map_err(|_| ()).fold(Vec::new(), |mut v, chunk| {
            v.extend_from_slice(&chunk);
            futures::future::ok(v)
        })
    });

    if let Ok(body) = body {
        let body_str = String::from_utf8_lossy(&body).into_owned();
        tx.unbounded_send(Ok(body_str)).expect("Unable to send upstream response!");
    } else {
        try_error!(stderr, "Unable to process upstream response!");
        tx.unbounded_send(Err("Unable to process upstream response!".to_string())).expect("");
    }
}