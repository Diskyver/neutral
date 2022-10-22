//! # neutral - unofficial rust client for neutrinoapi.com
//! Provide an API to interact with some features provided by [neutrinoapi.com](https://www.neutrinoapi.com).

//! # What is neutrinoapi.com
//! A general-purpose tool that solves recurring problems encountered during the development of software systems. It is used across many industries by software developers, data scientists and systems operators.

//! # How to use the neutral crate ?
//! The API is describe the the [Neutral](./struct.Neutral.html) structure.
//! Features are represented by module, each module contains a `send` function which call neutrinoapi.com using an instance of [Neutral](./struct.Neutral.html) structure.
//!
//!
//! By example, ip info feature from neutrinoapi is implemented inside the [neutral:ip_info](./ip_info/index.html) module.
//!
//! ```ignore
//! let api_auth = ApiAuth::new("userid", "apikey");
//! let client = Neutral::try_new("https://neutrinoapi.net", api_auth).unwrap();
//! let ip_info_response = ip_info::send(&client, ip_addr).await.unwrap();
//! ```

use http::{
    uri::{Authority, Scheme},
    Uri,
};

use hyper::{client::HttpConnector, header::HeaderValue, Client, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Deserializer, Serialize};

pub mod error;
pub mod hlr_lookup;
pub mod ip_blocklist;
pub mod ip_info;
pub mod ip_probe;
pub mod phone_validate;

use crate::error::Error;

#[cfg(test)]
use mockito;

/// Describes a kind of phone number.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "kebab-case"))]
pub enum PhoneInfoKind {
    Mobile,
    FixedLine,
    PremiumRate,
    TollFree,
    Voip,
    Unknown,
}

/// Describes a timezone from neutrinoapi.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NeutrinoTimeZoneResponse {
    pub id: String,
    pub name: String,
    pub abbr: String,
    pub date: String,
    pub time: String,
    pub offset: String,
}

/// Describes a network provider.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all(deserialize = "snake_case", serialize = "snake_case"))]
pub enum NeutrinoProviderKind {
    Isp,
    Hosting,
    Vpn,
    Proxy,
    University,
    Government,
    Commercial,
    Unknown,
}

/// Desribe a neutrinoapi sensor.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NeutrinoSensor {
    pub id: usize,
    pub blocklist: String,
    pub description: String,
}

pub(crate) fn object_empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    for<'a> T: Deserialize<'a>,
{
    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    struct Empty {}

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Aux<T> {
        T(T),
        Empty(Empty),
        Null,
    }

    match Deserialize::deserialize(deserializer)? {
        Aux::T(t) => Ok(Some(t)),
        Aux::Empty(_) | Aux::Null => Ok(None),
    }
}

/// Provide authorization credentials for neutrinoapi.com
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
pub struct ApiAuth<'a> {
    user_id: &'a str,
    api_key: &'a str,
}

impl<'a> ApiAuth<'a> {
    /// Create a new instance of `ApiAuth` using your neutrinoapi.com credentials.
    pub fn new(user_id: &'a str, api_key: &'a str) -> Self {
        ApiAuth {
            user_id: user_id,
            api_key: api_key,
        }
    }
}

/// A client to consume features provided by neutrinoapi.com
#[derive(Debug, Clone)]
pub struct Neutral<'a> {
    pub(crate) uri: Uri,
    pub(crate) auth: ApiAuth<'a>,
    pub(crate) client: Client<HttpsConnector<HttpConnector>>,
}

impl<'a> Neutral<'a> {
    /// Create a new Neutral instance. Needs some credentials to be authorized.
    /// Provide your neutrinoapi.com userid and apikey with an instance of `ApiAuth` as argument.
    pub fn try_new(uri: &str, auth: ApiAuth<'a>) -> Result<Self, Error> {
        let mut https = HttpsConnector::new();

        #[cfg(test)]
        let uri = &mockito::server_url();

        let uri = uri.parse::<Uri>()?;

        https.https_only(uri.scheme() == Some(&Scheme::HTTPS));

        Ok(Self {
            uri: uri,
            auth: auth,
            client: Client::builder().build::<_, hyper::Body>(https),
        })
    }

    /// Returns the URI scheme.
    pub fn scheme(&self) -> Option<&Scheme> {
        self.uri.scheme()
    }

    /// Returns the URI authority.
    pub fn authority(&self) -> Option<&Authority> {
        self.uri.authority()
    }

    pub(crate) fn uri_builder(&self) -> http::uri::Builder {
        Uri::builder()
            .authority(self.authority().unwrap().as_str())
            .scheme(self.scheme().unwrap().as_str())
    }

    pub(crate) fn add_authentication_headers<B>(&self, request: Request<B>) -> Request<B> {
        let user_id = self.auth.user_id;
        let api_key = self.auth.api_key;
        let mut request = Request::from(request);
        request
            .headers_mut()
            .insert("user-id", HeaderValue::from_str(user_id).unwrap());
        request
            .headers_mut()
            .insert("api-key", HeaderValue::from_str(api_key).unwrap());
        request
    }

    pub(crate) fn request_builder(
        &self,
        path_and_query: String,
    ) -> Result<http::request::Builder, Error> {
        let uri = self.uri_builder().path_and_query(path_and_query).build()?;

        Ok(Request::builder().uri(uri))
    }
}
