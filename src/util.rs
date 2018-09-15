// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` utils.
#[cfg(test)]
use failure::Error;
#[cfg(test)]
use std::fmt;
#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::net::{SocketAddr, ToSocketAddrs};

#[cfg(test)]
crate fn write_opt<S: Write, T: fmt::Display + fmt::Debug>(
    s: &mut S,
    key: &str,
    opt: &Option<T>,
) -> fmt::Result {
    if let Some(val) = opt {
        write!(s, "{}: {}", key, val).map_err(|_e| fmt::Error)?;
    };
    Ok(())
}

#[cfg(test)]
crate fn resolve(protocol: &str, host: &str, port: Option<u16>) -> Result<Vec<SocketAddr>, Error> {
    let port = if let Some(port) = port {
        port
    } else {
        match protocol {
            "http" => 80,
            "https" => 443,
            _ => 0,
        }
    };
    Ok((host, port).to_socket_addrs()?.collect())
}

#[cfg(test)]
mod test {
    use super::{resolve, write_opt};
    use std::io::Cursor;

    #[test]
    fn resolve_url() {
        let socket_addrs =
            resolve("https", "jasonozias.com", Some(443)).expect("Unable to resolve host");
        assert_eq!(socket_addrs.len(), 1);
        assert_eq!(format!("{}", socket_addrs[0]), "65.185.106.237:443");
    }

    #[test]
    fn write_some() {
        let buf = vec![];
        let mut cursor = Cursor::new(buf);
        write_opt(&mut cursor, "foo", &Some("bar")).expect("Could not write optional value!");
        assert_eq!("foo: bar", String::from_utf8_lossy(&cursor.get_ref()));
    }
}
