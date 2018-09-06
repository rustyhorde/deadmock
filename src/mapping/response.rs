// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` response templating configuration
use crate::mapping::Header;

#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    headers: Option<Vec<Header>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    body_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    proxy_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    additional_proxy_request_headers: Option<Vec<Header>>,
}
