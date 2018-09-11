// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` runtime
use clap::{App, Arg, ArgMatches};
use crate::error::Result;
use crate::handler::Handler;
use crate::header;
use crate::mapping::{Mapping, Mappings};
use crate::util;
use libdeadmock::{ProxyConfig, RuntimeConfig};
use slog::{Drain, Level, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::prelude::Stream;
use tomlenv::{Environment, Environments};
use uuid::Uuid;

/// CLI Runtime
pub fn run() -> Result<i32> {
    header::header();

    let matches: ArgMatches = App::new(env!("CARGO_PKG_NAME"))
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
                .long("env_path")
                .takes_value(true)
                .value_name("ENV_PATH")
                .help("Specify the full path to the 'env' directory"),
        ).arg(
            Arg::with_name("files_path")
                .short("f")
                .long("files_path")
                .takes_value(true)
                .value_name("FILES_PATH")
                .help("Specify the full path to the 'files' directory"),
        ).arg(
            Arg::with_name("mappings_path")
                .short("m")
                .long("mappings_path")
                .takes_value(true)
                .value_name("MAPPINGS_PATH")
                .help("Specify the full path to the 'mappings' directory"),
        ).arg(
            Arg::with_name("proxy")
                .short("p")
                .long("proxy")
                .requires("proxy-url")
                .help("Use a proxy"),
        ).arg(
            Arg::with_name("proxy-url")
                .long("proxy-url")
                .takes_value(true)
                .value_name("PROXY_URL")
                .help("Your proxy url, if applicable"),
        ).arg(
            Arg::with_name("proxy-username")
                .long("proxy-username")
                .takes_value(true)
                .value_name("PROXY_USER")
                .help("Your proxy username, if applicable"),
        ).arg(
            Arg::with_name("proxy-password")
                .long("proxy-password")
                .takes_value(true)
                .value_name("PROXY_PASS")
                .help("Your proxy password, if applicable"),
        ).get_matches();

    // Setup the environment.
    let dm_env = RuntimeConfig::env();
    let envs: Environments<Environment, RuntimeConfig> = Environments::try_from(&matches)?;
    let current = envs.current()?;

    // Setup the proxy config.
    let proxy_config = match ProxyConfig::try_from(&matches) {
        Ok(pc) => pc,
        Err(e) => return Err(e.as_fail().to_string().into()),
    };

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

    // Setup logging clones to move into handlers.
    let map_err_stderr = stderr.clone();
    let process_stderr = stderr.clone();
    let process_stdout = stdout.clone();

    // Load up the static mappings.
    let mut mappings = Mappings::new();
    let mappings_path = if let Some(mappings_path) = matches.value_of("mappings_path") {
        PathBuf::from(mappings_path)
    } else if let Some(config_path) = dirs::config_dir() {
        config_path.join("deadmock").join("mappings")
    } else {
        PathBuf::from("mappings")
    };

    util::visit_dirs(&mappings_path, &mut |entry| -> Result<()> {
        trace!(stdout, "Loading Mapping: {}", entry.path().display());
        let f = File::open(entry.path())?;
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let mapping: Mapping = toml::from_slice(&buffer)?;
        trace!(stdout, "{}", mapping);
        mappings.insert(Uuid::new_v4(), mapping);
        Ok(())
    })?;

    // Setup the files_path.
    let files_path = if let Some(files_path) = matches.value_of("files_path") {
        PathBuf::from(files_path)
    } else if let Some(config_path) = dirs::config_dir() {
        config_path.join("deadmock").join("files")
    } else {
        PathBuf::from("files")
    };

    // Setup the listener.
    let ip = if let Some(ip) = current.ip() {
        ip.clone()
    } else {
        "127.0.0.1".to_string()
    };
    let port = current.port().unwrap_or(32276);
    let addr = format!("{}:{}", ip, port);
    let socket_addr = addr.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&socket_addr)?;

    // Run the server.
    trace!(stdout, "{:?}", current);
    info!(stdout, "Listening on '{}'", socket_addr);

    tokio::run({
        listener
            .incoming()
            .map_err(move |e| error!(map_err_stderr, "Failed to accept socket: {}", e))
            .for_each(move |socket| {
                header::socket_info(&socket, &process_stdout);

                Handler::new(
                    socket,
                    mappings.clone(),
                    proxy_config.clone(),
                    files_path.clone(),
                ).stdout(process_stdout.clone())
                .stderr(process_stderr.clone())
                .handle();
                Ok(())
            })
    });

    Ok(0)
}
