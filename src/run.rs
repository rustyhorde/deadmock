// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` runtime
use clap::{App, Arg};
use crate::handler::Handler;
use crate::header;
use failure::Error;
use libdeadmock::{Loggers, MappingsConfig, ProxyConfig, RuntimeConfig};
use slog::{b, error, info, kv, log, record, record_static, trace};
use slog_try::{try_error, try_info, try_trace};
use std::convert::TryFrom;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::prelude::Stream;
use tomlenv::{Environment, Environments};

/// CLI Runtime
crate fn run() -> Result<i32, Error> {
    header::header();
    let default_config_path = if let Some(config_dir) = dirs::config_dir() {
        let prog_config_dir = config_dir.join(env!("CARGO_PKG_NAME"));
        format!("{}", prog_config_dir.display())
    } else if let Ok(current_dir) = env::current_dir() {
        format!("{}", current_dir.display())
    } else {
        ".".to_string()
    };

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
                .long("env_path")
                .takes_value(true)
                .value_name("ENV_PATH")
                .default_value(&default_config_path[..])
                .help("Specify the full path to the directory where 'env.toml' lives"),
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
                .default_value(&default_config_path[..])
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
    let envs: Environments<Environment, RuntimeConfig> = Environments::try_from(&matches)?;
    let current = envs.current()?;

    // Setup the proxy config.
    let proxy_config = ProxyConfig::try_from(&matches)?;

    // Load up the static mappings.
    let mappings = MappingsConfig::try_from(&matches)?;

    // Setup the logging.
    let loggers = Loggers::try_from(&matches)?;
    let (stdout, stderr) = loggers.split();

    // Setup logging clones to move into handlers.
    let map_err_stderr = stderr.clone();
    let process_stderr = stderr.clone();
    let process_stdout = stdout.clone();

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
    try_trace!(stdout, "{:?}", current);
    try_info!(stdout, "Listening on '{}'", socket_addr);

    tokio::run({
        listener
            .incoming()
            .map_err(move |e| try_error!(map_err_stderr, "Failed to accept socket: {}", e))
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
