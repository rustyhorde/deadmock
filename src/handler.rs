// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` request/response handler.
use http::Http;
use http_types::{Request, Response, StatusCode};
use matcher::Mappings;
use serde_json;
use slog::Logger;
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
}

impl Handler {
    pub fn new(stream: TcpStream, state: Arc<Mutex<Mappings>>) -> Self {
        Self {
            stdout: None,
            stderr: None,
            stream,
            state,
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
        let task = tx
            .send_all(rx.and_then(move |req| respond(req, response_state.clone())))
            .then(move |res| {
                if let Err(e) = res {
                    try_error!(
                        response_stderr,
                        "failed to process connection; error = {:?}",
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
    _state: Arc<Mutex<Mappings>>,
) -> Box<Future<Item = Response<String>, Error = io::Error> + Send> {
    let mut response = Response::builder();
    let body = match req.uri().path() {
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
    Box::new(future::ok(response.body(body).unwrap()))
}
