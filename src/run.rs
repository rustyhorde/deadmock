// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` runtime
use clap::{App, Arg};
use failure::Error;
use libdeadmock::matcher::Enabled;
use libdeadmock::{config, logging, server};
use std::convert::TryFrom;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use tomlenv::{Environment, Environments};

/// CLI Runtime
crate fn run() -> Result<i32, Error> {
    server::header();

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
        )
        .arg(
            Arg::with_name("env_path")
                .short("e")
                .long("env_path")
                .takes_value(true)
                .value_name("ENV_PATH")
                .default_value(&default_config_path[..])
                .help("Specify the full path to the directory where 'env.toml' lives"),
        )
        .arg(
            Arg::with_name("mappings_path")
                .short("m")
                .long("mappings_path")
                .takes_value(true)
                .value_name("MAPPINGS_PATH")
                .default_value(&default_config_path[..])
                .help("Specify the full path to the parent directory of your mappings"),
        )
        .arg(
            Arg::with_name("files_path")
                .short("f")
                .long("files_path")
                .takes_value(true)
                .value_name("FILES_PATH")
                .help("Specify the full path to the 'files' directory"),
        )
        .arg(
            Arg::with_name("proxy")
                .short("p")
                .long("proxy")
                .requires("proxy-url")
                .help("Use a proxy"),
        )
        .arg(
            Arg::with_name("proxy-url")
                .long("proxy-url")
                .takes_value(true)
                .value_name("PROXY_URL")
                .help("Your proxy url, if applicable"),
        )
        .arg(
            Arg::with_name("proxy-username")
                .long("proxy-username")
                .takes_value(true)
                .value_name("PROXY_USER")
                .help("Your proxy username, if applicable"),
        )
        .arg(
            Arg::with_name("proxy-password")
                .long("proxy-password")
                .takes_value(true)
                .value_name("PROXY_PASS")
                .help("Your proxy password, if applicable"),
        )
        .get_matches();

    // Setup the environment.
    let envs: Environments<Environment, config::Runtime> = Environments::try_from(&matches)?;
    let current = envs.current()?;

    // Setup the proxy config.
    let proxy_config = config::Proxy::try_from(&matches)?;

    // Load up the static mappings.
    let mappings = config::Mappings::try_from(&matches)?;

    // Setup the logging.
    let loggers = logging::Loggers::try_from(&matches)?;
    let (stdout, stderr) = loggers.split();

    // Setup logging clones to move into handlers.
    let process_stderr = stderr.clone();
    let process_stdout = stdout.clone();

    // Setup the files_path.
    let files_path = if let Some(files_path) = matches.value_of("files_path") {
        PathBuf::from(files_path)
    } else if let Some(config_path) = dirs::config_dir() {
        config_path.join(env!("CARGO_PKG_NAME")).join("files")
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

    let enabled = Enabled::EXACT_URL
        | Enabled::PATTERN_URL
        | Enabled::EXACT_METHOD
        | Enabled::EXACT_HEADER
        | Enabled::EXACT_HEADERS;
    let handler = server::Handler::new(
        enabled,
        mappings.clone(),
        proxy_config.clone(),
        files_path.clone(),
    )
    .stdout(process_stdout.clone())
    .stderr(process_stderr.clone());

    // Run the server.
    let _ = server::run(&socket_addr, handler);

    Ok(0)
}
