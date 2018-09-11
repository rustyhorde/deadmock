// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` request/response handler.
use cached::UnboundCache;
use crate::codec::inbound::Http;
use crate::error::Result;
use crate::http_types::{Request as HttpRequest, Response as HttpResponse, StatusCode};
use crate::mapping::{Header, Mappings, Response};
use crate::matcher::Matcher;
use crate::util;
use futures::{future, Future, Sink, Stream};
use hyper::client::HttpConnector;
use hyper::{Client, Request as HyperRequest};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_tls::HttpsConnector;
use libdeadmock::ProxyConfig;
use slog::Logger;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::prelude::FutureExt;
use tokio_codec::Decoder;
use typed_headers::Credentials;

#[derive(Debug)]
pub struct Handler {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    proxy_config: ProxyConfig,
    files_path: PathBuf,
    stream: TcpStream,
    static_mappings: Mappings,
    dynamic_mappings: Arc<Mutex<Mappings>>,
}

impl Handler {
    pub fn new(
        stream: TcpStream,
        static_mappings: Mappings,
        proxy_config: ProxyConfig,
        files_path: PathBuf,
    ) -> Self {
        Self {
            stdout: None,
            stderr: None,
            proxy_config,
            files_path,
            stream,
            static_mappings,
            dynamic_mappings: Arc::new(Mutex::new(Mappings::new())),
        }
    }

    pub fn stdout(mut self, stdout: Logger) -> Self {
        self.stdout = Some(stdout);
        self
    }

    pub fn stderr(mut self, stderr: Logger) -> Self {
        self.stderr = Some(stderr);
        self
    }

    pub fn handle(self) {
        // Frame the socket using the `Http` protocol. This maps the TCP socket
        // to a Stream + Sink of HTTP frames.
        // This splits a single `Stream + Sink` value into two separate handles
        // that can be used independently (even on different tasks or threads).
        let (tx, rx) = Http.framed(self.stream).split();

        // Clone all the things....
        let response_stdout = self.stdout.clone();
        let response_stderr = self.stderr.clone();
        let response_stderr_1 = self.stderr.clone();
        let static_mappings = self.static_mappings.clone();
        let dynamic_mappings = self.dynamic_mappings.clone();
        let proxy_config = self.proxy_config.clone();
        let files_path = self.files_path.clone();

        // Map all requests into responses and send them back to the client.
        let task = tx
            .send_all(rx.and_then(move |req| {
                respond(
                    &req,
                    proxy_config.clone(),
                    files_path.clone(),
                    response_stdout.clone(),
                    response_stderr.clone(),
                    &static_mappings,
                    &dynamic_mappings,
                ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            })).then(move |res| {
                if let Err(e) = res {
                    try_error!(
                        response_stderr_1,
                        "failed to process connection; error = {}",
                        e
                    );
                }

                Ok(())
            });

        // Spawn the task that handles the connection.
        tokio::spawn(task);
    }
}

/// "Server logic" is implemented in this function.
///
/// This function is a map from and HTTP request to a future of a response and
/// represents the various handling a server might do.
fn respond(
    request: &HttpRequest<()>,
    proxy_config: ProxyConfig,
    files_path: PathBuf,
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    static_mappings: &Mappings,
    dynamic_mappings: &Arc<Mutex<Mappings>>,
) -> Box<Future<Item = HttpResponse<String>, Error = String> + Send> {
    let matcher = Matcher {};
    if let Ok(mapping) = matcher.get_match(&request, &static_mappings) {
        try_trace!(stdout, "{}", mapping);
        http_response(
            &request,
            mapping.response(),
            stdout,
            stderr,
            proxy_config,
            files_path,
        )
    } else {
        let locked_dynamic_mappings = match dynamic_mappings.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Ok(mapping) = matcher.get_match(&request, &locked_dynamic_mappings) {
            try_trace!(stdout, "{}", mapping);
            http_response(
                &request,
                mapping.response(),
                stdout,
                stderr,
                proxy_config,
                files_path,
            )
        } else {
            try_error!(stderr, "No mapping found");
            util::error_response_fut("No mapping found".to_string(), StatusCode::NOT_FOUND)
        }
    }
}

fn http_response(
    request: &HttpRequest<()>,
    response_config: &Response,
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    proxy_config: ProxyConfig,
    files_path: PathBuf,
) -> Box<Future<Item = HttpResponse<String>, Error = String> + Send> {
    if let Some(proxy_base_url) = response_config.proxy_base_url() {
        let full_url = format!("{}{}", proxy_base_url, request.uri());
        let (tx, rx) = futures::sync::mpsc::unbounded();
        let headers = response_config.additional_proxy_request_headers().clone();

        tokio::spawn_async(
            async move {
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
                        let proxy_connector = ProxyConnector::from_proxy(connector, proxy)
                            .expect("Unable to create proxy connector!");
                        let client = Client::builder()
                            .set_host(true)
                            .build::<_, hyper::Body>(proxy_connector);
                        await!(run_request(client, tx, full_url, stdout, stderr, headers));
                    } else {
                        panic!("Unable to determine proxy url!");
                    }
                } else if full_url.starts_with("https") {
                    let https_connector =
                        HttpsConnector::new(4).expect("TLS initialization failed");
                    let client = Client::builder()
                        .set_host(true)
                        .build::<_, hyper::Body>(https_connector);
                    await!(run_request(client, tx, full_url, stdout, stderr, headers));
                } else {
                    let http_connector = HttpConnector::new(4);
                    let client = Client::builder()
                        .set_host(true)
                        .build::<_, hyper::Body>(http_connector);
                    await!(run_request(client, tx, full_url, stdout, stderr, headers));
                }
            },
        );

        Box::new(
            rx.fold(String::new(), |mut buffer, res| {
                match res {
                    Ok(val) => buffer.push_str(&val),
                    Err(e) => buffer.push_str(&e.to_string()),
                }
                futures::future::ok(buffer)
            }).map_err(|_| "Error processing upstream response".to_string())
            .map(HttpResponse::new),
        )
    } else {
        let mut response_builder = HttpResponse::builder();
        if let Some(headers) = response_config.headers() {
            for header in headers {
                response_builder.header(&header.key()[..], &header.value()[..]);
            }
        }

        if let Some(status) = response_config.status() {
            response_builder.status(if let Ok(status) = StatusCode::from_u16(*status) {
                status
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            });
        } else {
            response_builder.status(StatusCode::OK);
        }

        let body = if let Some(body_file_name) = response_config.body_file_name() {
            match load(files_path, body_file_name) {
                Ok(body) => body,
                Err(e) => e.to_string(),
            }
        } else {
            "Unable to process body".to_string()
        };

        match response_builder.body(body) {
            Ok(response) => Box::new(future::ok(response)),
            Err(e) => util::error_response_fut(format!("{}", e), StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

async fn run_request<C>(
    client: Client<C, hyper::Body>,
    tx: futures::sync::mpsc::UnboundedSender<std::result::Result<String, String>>,
    url: String,
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    headers: Option<Vec<Header>>,
) where
    C: hyper::client::connect::Connect + Sync + 'static,
{
    match await!({
        try_trace!(stdout, "Making request to {}", url);
        let mut request_builder = HyperRequest::get(url);

        if let Some(headers) = headers {
            for header in headers {
                request_builder.header(&header.key()[..], &header.value()[..]);
            }
        }
        let body = request_builder
            .body(hyper::Body::empty())
            .expect("Unable to create upstream request");
        client
            .request(body)
            .timeout(std::time::Duration::from_secs(10))
    }) {
        Ok(response) => {
            let body = await!({
                response
                    .into_body()
                    .map_err(|_| ())
                    .fold(Vec::new(), |mut v, chunk| {
                        v.extend_from_slice(&chunk);
                        futures::future::ok(v)
                    })
            });

            if let Ok(body) = body {
                let body_str = String::from_utf8_lossy(&body).into_owned();
                tx.unbounded_send(Ok(body_str))
                    .expect("Unable to send upstream response!");
            } else {
                try_error!(stderr, "Unable to process upstream response!");
                tx.unbounded_send(Err("Unable to process upstream response!".to_string()))
                    .expect("Unable to send upstream response!");
            }
        }
        Err(e) => {
            try_error!(stderr, "Unable to process upstream response! {}", e);
            tx.unbounded_send(Err(format!("Unable to process upstream response! {}", e)))
                .expect("Unable to send upstream response!");
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
