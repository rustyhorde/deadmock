// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` utils.
use crate::error::Result;
use crate::http_types::header::{HeaderValue, CONTENT_TYPE};
use crate::http_types::{Response, StatusCode};
use futures::{future, Future};
use std::fmt;
use std::fs::{self, DirEntry};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;

#[allow(dead_code)]
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

pub fn visit_dirs<F>(dir: &Path, cb: &mut F) -> Result<()>
where
    F: FnMut(&DirEntry) -> Result<()>,
{
    if fs::metadata(dir)?.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if fs::metadata(entry.path())?.is_dir() {
                visit_dirs(&entry.path(), cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn response(body: String, status_code: StatusCode) -> Response<String> {
    let mut response = Response::builder();
    response
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .status(status_code);

    if let Ok(response) = response.body(body) {
        response
    } else {
        error_response(
            "Unable to process body".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

pub fn error_response_fut(
    body: String,
    status_code: StatusCode,
) -> Box<Future<Item = Response<String>, Error = String> + Send> {
    Box::new(future::ok(error_response(body, status_code)))
}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

pub fn error_response(message: String, status_code: StatusCode) -> Response<String> {
    let mut response = Response::builder();
    response
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .status(status_code);

    if let Ok(message) = serde_json::to_string(&ErrorMessage { message }) {
        if let Ok(response) = response.body(message) {
            return response;
        }
    }

    Response::new(r#"{ "message": "Unable to process body" }"#.to_string())
}

#[allow(dead_code)]
pub fn resolve(protocol: &str, host: &str) -> Result<Vec<SocketAddr>> {
    let port = match protocol {
        "http" => 80,
        "https" => 443,
        _ => 0,
    };
    Ok((host, port).to_socket_addrs()?.collect())
}
