// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` proxy request handler.

use curl::easy::{Easy, Transfer};
use error::Result;

#[derive(Debug)]
pub struct Proxy {
    handle: Easy,
}

impl Proxy {
    pub fn new(url: &str) -> Result<Self> {
        let mut handle = Easy::new();
        handle.url(url)?;
        Ok(Self { handle })
    }

    pub fn transfer(&mut self) -> Transfer {
        self.handle.transfer()
    }
}
