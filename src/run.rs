// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` runtime
use clap::{App, Arg};
use environment::Env;
use error::Result;
use http::Http;
use http_types::{Request, Response, StatusCode};
use serde_json;
use slog::{Drain, Level, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};
use std::io;
use std::net::SocketAddr;
use std::path::Path;

use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::{future, Future, Sink, Stream};
use tokio_codec::Decoder;

use tomlenv::{Environment, Environments};

/// CLI Runtime
pub fn run() -> Result<i32> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Proxy server for hosting mocked responses on match criteria")
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Set the logging verbosity"),
        ).arg(
            Arg::with_name("env_path")
                .short("e")
                .long("envpath")
                .help("Specify the env file path"),
        ).get_matches();

    // Setup the environment.
    let dm_env = Env::get_env_var();
    let mut buffer = String::new();
    let env_path = matches.value_of("env_path").unwrap_or_else(|| "env.toml");
    let envs: Environments<Environment, Env> =
        Environments::from_path(Path::new(env_path), &mut buffer)?;
    let current = envs.current()?;

    // Setup the logging.
    let level = match matches.occurrences_of("v") {
        0 => Level::Warning,
        1 => Level::Info,
        2 => Level::Debug,
        3 | _ => Level::Trace,
    };

    let stdout_decorator = TermDecorator::new().stdout().build();
    let stdout_drain = CompactFormat::new(stdout_decorator).build().fuse();
    let stdout_async_drain = Async::new(stdout_drain).build().filter_level(level).fuse();
    let stdout = Logger::root(stdout_async_drain, o!());

    let stderr_decorator = TermDecorator::new().stdout().build();
    let stderr_drain = CompactFormat::new(stderr_decorator).build().fuse();
    let stderr_async_drain = Async::new(stderr_drain)
        .build()
        .filter_level(Level::Error)
        .fuse();
    let stderr = Logger::root(
        stderr_async_drain,
        o!(env!("CARGO_PKG_NAME") => env!("CARGO_PKG_VERSION")),
    );

    let ip = current.ip().unwrap_or("127.0.0.1");
    let port = current.port().unwrap_or(32276);
    let addr = format!("{}:{}", ip, port);
    let socket_addr = addr.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&socket_addr)?;

    info!(stdout, "Runtime Environment: {}", dm_env);
    info!(stdout, "{}", current);
    info!(stdout, "Listening on '{}'", socket_addr);
    info!(
        stdout,
        "Build Timestamp: {}",
        env!("VERGEN_BUILD_TIMESTAMP")
    );;
    info!(stdout, "Build Date: {}", env!("VERGEN_BUILD_DATE"));

    tokio::run({
        listener
            .incoming()
            .map_err(move |e| error!(stderr, "failed to accept socket: {}", e))
            .for_each(|socket| {
                process(socket);
                Ok(())
            })
    });

    Ok(0)
}

fn process(socket: TcpStream) {
    let (tx, rx) =
        // Frame the socket using the `Http` protocol. This maps the TCP socket
        // to a Stream + Sink of HTTP frames.
        Http.framed(socket)
        // This splits a single `Stream + Sink` value into two separate handles
        // that can be used independently (even on different tasks or threads).
        .split();

    // Map all requests into responses and send them back to the client.
    let task = tx.send_all(rx.and_then(respond)).then(|res| {
        if let Err(e) = res {
            println!("failed to process connection; error = {:?}", e);
        }

        Ok(())
    });

    // Spawn the task that handles the connection.
    tokio::spawn(task);
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
fn respond(req: Request<()>) -> Box<Future<Item = Response<String>, Error = io::Error> + Send> {
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
