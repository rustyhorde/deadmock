// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` request/response handler.
use crate::codec::inbound::Http;
use crate::config::ProxyConfig;
use crate::http_types::{Request, Response, StatusCode};
use crate::matcher::Mappings;
use crate::util;
use slog::Logger;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::prelude::{Future, Sink, Stream};
use tokio_codec::Decoder;

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
    pub fn new(stream: TcpStream, static_mappings: Mappings, proxy_config: ProxyConfig, files_path: PathBuf) -> Self {
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
                    req,
                    proxy_config.clone(),
                    files_path.clone(),
                    response_stdout.clone(),
                    response_stderr.clone(),
                    static_mappings.clone(),
                    dynamic_mappings.clone(),
                ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            })).then(move |res| {
                if let Err(e) = res {
                    try_error!(response_stderr_1, "failed to process connection; error = {}", e);
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
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn respond(
    request: Request<()>,
    proxy_config: ProxyConfig,
    files_path: PathBuf,
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    static_mappings: Mappings,
    dynamic_mappings: Arc<Mutex<Mappings>>,
) -> Box<Future<Item = Response<String>, Error = String> + Send> {
    if let Ok(matched) = static_mappings.get_match(&request) {
        try_trace!(stdout, "{}", matched);
        matched.http_response(&request, stdout, stderr, proxy_config, files_path)
    } else {
        let locked_dynamic_mappings = match dynamic_mappings.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Ok(matched) = locked_dynamic_mappings.get_match(&request) {
            try_trace!(stdout, "{}", matched);
            matched.http_response(&request, stdout, stderr, proxy_config, files_path)
        } else {
            try_error!(stderr, "No mapping found");
            util::error_response_fut("No mapping found".to_string(), StatusCode::NOT_FOUND)
        }
    }
}
