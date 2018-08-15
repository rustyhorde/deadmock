// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` request/response handler.
use client::Proxy;
use error::Result;
use http::Http;
use http_types::{Request, Response, StatusCode};
use matcher::Mappings;
use serde_json;
use slog::Logger;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use tokio::prelude::{future, Future, Sink, Stream};
use tokio_codec::Decoder;

#[derive(Debug)]
pub struct Handler {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    stream: TcpStream,
    state: Arc<Mutex<Mappings>>,
    proxy_map: Arc<Mutex<HashMap<&'static str, Proxy>>>,
}

impl Handler {
    pub fn new(stream: TcpStream, state: Arc<Mutex<Mappings>>) -> Self {
        Self {
            stdout: None,
            stderr: None,
            stream,
            state,
            proxy_map: Arc::new(Mutex::new(HashMap::new())),
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

        // Map all requests into responses and send them back to the client.
        let response_stderr = self.stderr.clone();
        let response_state = self.state.clone();
        let proxy_map_clone = self.proxy_map.clone();
        let task =
            tx.send_all(rx.and_then(move |req| {
                respond(req, response_state.clone(), proxy_map_clone.clone())
            })).then(move |res| {
                if let Err(e) = res {
                    try_error!(
                        response_stderr,
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

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

/// "Server logic" is implemented in this function.
///
/// This function is a map from and HTTP request to a future of a response and
/// represents the various handling a server might do. Currently the contents
/// here are pretty uninteresting.
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn respond(
    req: Request<()>,
    state: Arc<Mutex<Mappings>>,
    proxy_map: Arc<Mutex<HashMap<&'static str, Proxy>>>,
) -> Box<Future<Item = Response<String>, Error = io::Error> + Send> {
    match try_respond(req, state, proxy_map) {
        Ok(response) => Box::new(future::ok(response)),
        Err(e) => {
            let response = Response::builder();
            response.status(503);
            Box::new(future::ok(response.body(e.to_string()).unwrap()))
        }
    }
}

fn try_respond(
    req: Request<()>,
    state: Arc<Mutex<Mappings>>,
    proxy_map: Arc<Mutex<HashMap<&'static str, Proxy>>>,
) -> Result<Response<String>> {
    let mut response = Response::builder();

    let mut locked_state = match state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    for (_, mapping) in locked_state.mappings_mut() {
        if mapping.has_match(&req) {
            ()
        }
    }

    let body = match req.uri().path() {
        "/healthcheck" => {
            let mut locked_map = proxy_map.lock().unwrap();
            let mut data = Vec::new();

            let mut proxy = locked_map.entry("healthcheck").or_insert_with(|| {
                Proxy::new("http://ecsb-test.kroger.com/mobilecheckout/healthcheck").unwrap()
            });
            {
                let mut transfer = proxy.transfer();
                transfer
                    .write_function(|incoming_data| {
                        data.extend_from_slice(incoming_data);
                        Ok(incoming_data.len())
                    }).unwrap();
                transfer.perform().unwrap();
            }

            response.header("Content-Type", "text/plain");
            String::from_utf8_lossy(&data).into_owned()
        }
        "/plaintext" => {
            response.header("Content-Type", "text/plain");
            "Hello, World!".to_string()
        }
        "/json" => {
            response.header("Content-Type", "application/json");
            serde_json::to_string(&Message {
                message: "Hello, World!",
            }).unwrap()
        }
        _ => {
            response.status(StatusCode::NOT_FOUND);
            String::new()
        }
    };

    Ok(response.body()?)
}
