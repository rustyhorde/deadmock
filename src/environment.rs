// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` environment config
use std::env;
use std::fmt;

/// The runtime environment for deadmock.
#[derive(Clone, Copy, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Env<'a> {
    /// The IP address to listen on.
    #[get = "pub"]
    ip: Option<&'a str>,
    /// The port to listen on.
    #[get = "pub"]
    port: Option<u32>,
    /// Log level filter.
    #[get = "pub"]
    level: Option<&'a str>,
}

fn write_opt<T: fmt::Display + fmt::Debug>(
    f: &mut fmt::Formatter,
    key: &str,
    opt: Option<T>,
) -> fmt::Result {
    if let Some(val) = opt {
        write!(f, "{}: {}, ", key, val)?
    };
    Ok(())
}

impl<'a> fmt::Display for Env<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Env {{ ")?;
        write_opt(f, "ip", self.ip).map_err(|_| fmt::Error)?;
        write_opt(f, "port", self.port).map_err(|_| fmt::Error)?;
        write_opt(f, "level", self.level).map_err(|_| fmt::Error)?;
        write!(f, "}}")
    }
}

impl<'a> Env<'a> {
    pub fn get_env_var() -> String {
        env::var("env").unwrap_or_else(|_| {
            env::set_var("env", "local");
            "local".to_string()
        })
    }
}
