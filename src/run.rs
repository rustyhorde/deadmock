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
use slog::{Drain, Level, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};
use std::net::SocketAddr;
use std::path::Path;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::Stream;
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
                .long_help(
                    "Set the logging verbosity
Note: This will override a log level defined in your environment config",
                ),
        ).arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Specify the config file location"),
        ).get_matches();

    // Setup the environment.
    let dm_env = Env::get_env_var();
    let mut buffer = String::new();
    let envs: Environments<Environment, Env> =
        Environments::from_path(Path::new("env.toml"), &mut buffer)?;
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

    // Parse the arguments, bind the TCP socket we'll be listening to, spin up
    // our worker threads, and start shipping sockets to those worker threads.
    let ip = current.ip().unwrap_or("127.0.0.1");
    let port = current.port().unwrap_or(32276);
    let addr = format!("{}:{}", ip, port);
    let socket_addr = addr.parse::<SocketAddr>()?;

    let listener = TcpListener::bind(&socket_addr)?;

    info!(stdout, "Runtime Environment: {}", dm_env);
    info!(stdout, "{}", current);
    info!(stdout, "Listening on '{}'", socket_addr);

    tokio::run({
        listener
            .incoming()
            .map_err(move |e| error!(stderr, "failed to accept socket: {:?}", e))
            .for_each(move |socket| {
                info!(stdout, "{:?}", socket);
                Ok(())
            })
    });

    Ok(0)
}
