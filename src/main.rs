// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` 0.1.0
#![deny(missing_docs)]
#![feature(
    await_macro,
    async_await,
    duration_as_u128,
    futures_api,
    try_from,
    uniform_paths
)]

#[macro_use]
extern crate cached;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate getset;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_try;
#[macro_use]
extern crate tokio;

extern crate http as http_types;

mod codec;
mod config;
mod environment;
mod error;
mod handler;
mod header;
mod matcher;
mod run;
mod util;

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
