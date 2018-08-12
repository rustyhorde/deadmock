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
use handler::Handler;
use header;
use matcher::Mappings;
use slog::{Drain, Level, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpListener;
use tokio::prelude::Stream;
use tomlenv::{Environment, Environments};

/// CLI Runtime
pub fn run() -> Result<i32> {
    header::header();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("VERGEN_SEMVER"))
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
    let stdout = Logger::root(
        stdout_async_drain,
        o!(env!("CARGO_PKG_NAME") => env!("VERGEN_SEMVER"), "env" => dm_env.clone()),
    );

    let stderr_decorator = TermDecorator::new().stdout().build();
    let stderr_drain = CompactFormat::new(stderr_decorator).build().fuse();
    let stderr_async_drain = Async::new(stderr_drain)
        .build()
        .filter_level(Level::Error)
        .fuse();
    let stderr = Logger::root(
        stderr_async_drain,
        o!(env!("CARGO_PKG_NAME") => env!("VERGEN_SEMVER"), "env" => dm_env.clone()),
    );

    // Load up the static mappings.
    let mappings = Mappings::new();

    let state = Arc::new(Mutex::new(mappings));

    // Setup the listener.
    let ip = current.ip().unwrap_or("127.0.0.1");
    let port = current.port().unwrap_or(32276);
    let addr = format!("{}:{}", ip, port);
    let socket_addr = addr.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&socket_addr)?;

    trace!(stdout, "{}", current);
    info!(stdout, "Listening on '{}'", socket_addr);

    // Setup logging clones to move into handlers.
    let map_err_stderr = stderr.clone();
    let process_stderr = stderr.clone();
    let process_stdout = stdout.clone();

    tokio::run({
        listener
            .incoming()
            .map_err(move |e| error!(map_err_stderr, "failed to accept socket: {}", e))
            .for_each(move |socket| {
                header::socket_info(&socket, &process_stdout);

                Handler::new(socket, state.clone())
                    .stdout(process_stdout.clone())
                    .stderr(process_stderr.clone())
                    .handle();
                Ok(())
            })
    });

    Ok(0)
}
