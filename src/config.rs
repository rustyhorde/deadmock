// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` proxy configuration
use clap::ArgMatches;

#[derive(Clone, Debug, Default, Getters, Hash, Eq, PartialEq)]
pub struct ProxyConfig {
    #[get = "pub"]
    proxy_url: Option<String>,
    #[get = "pub"]
    use_proxy: bool,
    #[get = "pub"]
    proxy_username: Option<String>,
    #[get = "pub"]
    proxy_password: Option<String>,
}

impl <'a> From<&'a ArgMatches<'a>> for ProxyConfig {
    fn from(matches: &'a ArgMatches<'a>) -> Self {
        let proxy_url = matches.value_of("proxy-url").map(|v| v.to_string());
        let use_proxy = matches.is_present("proxy");
        let proxy_username = matches.value_of("proxy-username").map(|v| v.to_string());
        let proxy_password = matches.value_of("proxy-password").map(|v| v.to_string());

        Self {
            proxy_url,
            use_proxy,
            proxy_username,
            proxy_password,
        }
    }
}