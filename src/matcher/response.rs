// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` response templating configuration
#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Response {
    proxy_base_url: Option<String>,
    additional_proxy_request_headers: Option<Vec<Header>>,
}

#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Header {
    key: String,
    value: String,
}
