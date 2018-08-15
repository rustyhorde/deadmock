// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` utils.
use std::fmt;

pub fn write_opt<T: fmt::Display + fmt::Debug>(
    f: &mut fmt::Formatter,
    key: &str,
    opt: &Option<T>,
) -> fmt::Result {
    if let Some(val) = opt {
        write!(f, "{}: {}", key, val)?
    };
    Ok(())
}
