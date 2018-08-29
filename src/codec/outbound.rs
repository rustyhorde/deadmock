// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` outbound codecs
use bytes::BytesMut;
use crate::http_types::{Request, Response, StatusCode, Version};
use std::fmt;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

pub struct Http;

// Right now `write!` on `Vec<u8>` goes through io::Write and is not
// super speedy, so inline a less-crufty implementation here which
// doesn't go through io::Error.
struct BytesWrite<'a>(&'a mut BytesMut);

impl<'a> fmt::Write for BytesWrite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::write(self, args)
    }
}

/// Implementation of encoding an HTTP response into a `BytesMut`, basically
/// just writing out an HTTP/1.1 response.
impl Encoder for Http {
    type Item = Request<()>;
    type Error = io::Error;

    fn encode(&mut self, item: Request<()>, dst: &mut BytesMut) -> io::Result<()> {
        use std::fmt::Write;

        println!("{} {} HTTP/1.1\r\n", item.method(), item.uri());

        write!(
            BytesWrite(dst),
            "{} {} HTTP/1.1\r\nHost: www.google.com\r\n",
            item.method(),
            item.uri()
        ).unwrap();

        // for (k, v) in item.headers() {
        //     dst.extend_from_slice(k.as_str().as_bytes());
        //     dst.extend_from_slice(b": ");
        //     dst.extend_from_slice(v.as_bytes());
        //     dst.extend_from_slice(b"\r\n");
        // }

        dst.extend_from_slice(b"\r\n");
        // dst.extend_from_slice(item.body().as_bytes());

        Ok(())
    }
}

/// Implementation of decoding an HTTP response from the bytes we've read so far.
/// This leverages the `httparse` crate to do the actual parsing and then we use
/// that information to construct an instance of a `http::Response` object,
/// trying to avoid allocations where possible.
impl Decoder for Http {
    type Item = Response<String>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Response<String>>> {
        // TODO: we should grow this headers array if parsing fails and asks
        //       for more headers
        let mut headers = [None; 16];
        let (_reason, version, amt) = {
            let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Response::new(&mut parsed_headers);
            let status = r.parse(src).map_err(|e| {
                let msg = format!("failed to parse http response: {:?}", e);
                io::Error::new(io::ErrorKind::Other, msg)
            })?;

            let amt = match status {
                httparse::Status::Complete(amt) => amt,
                httparse::Status::Partial => return Ok(None),
            };

            let toslice = |a: &[u8]| {
                let start = a.as_ptr() as usize - src.as_ptr() as usize;
                assert!(start < src.len());
                (start, start + a.len())
            };

            for (i, header) in r.headers.iter().enumerate() {
                let k = toslice(header.name.as_bytes());
                let v = toslice(header.value);
                headers[i] = Some((k, v));
            }

            (
                toslice(r.reason.unwrap().as_bytes()),
                r.version.unwrap(),
                amt,
            )
        };
        if version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "only HTTP/1.1 accepted",
            ));
        }
        println!("Amount: {}", amt);
        println!("Length: {}", src.len());
        let data = src.split_to(amt).freeze();
        let mut response = Response::builder();
        response.version(Version::HTTP_11);
        response.status(StatusCode::OK);
        // let mut request = Request::builder();
        // request.method(&data[method.0..method.1]);
        // request.uri(data.slice(path.0, path.1));
        // request.version(Version::HTTP_11);
        // for header in &headers {
        //     let (k, v) = match *header {
        //         Some((ref k, ref v)) => (k, v),
        //         None => break,
        //     };
        //     let value = unsafe { HeaderValue::from_shared_unchecked(data.slice(v.0, v.1)) };
        //     response.header(&data[k.0..k.1], value);
        // }

        let body = String::from_utf8_lossy(&src);
        println!("Data: {}", String::from_utf8_lossy(&data));
        println!("Body: {}", body);
        let res = response
            .body(body.to_string())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Some(res))
    }
}
