// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` 0.1.0
#![deny(missing_docs)]
#![feature(duration_as_u128, try_from)]
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate getset;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_try;

extern crate bytes;
extern crate chrono;
extern crate clap;
extern crate colored;
extern crate curl;
extern crate http as http_types;
extern crate httparse;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate slog_async;
extern crate slog_term;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate toml;
extern crate tomlenv;
extern crate uuid;

mod client;
mod environment;
mod error;
mod handler;
mod header;
mod http;
mod matcher;
mod run;

use std::io::{self, Write};
use std::process;

/// CLI Entry Point
fn main() {
    match run::run() {
        Ok(i) => process::exit(i),
        Err(e) => {
            writeln!(io::stderr(), "{}", e).expect("Unable to write to stderr!");
            process::exit(1)
        }
    }
}
