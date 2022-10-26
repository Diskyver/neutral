//! # neutral - unofficial rust client for neutrinoapi.com
//! Provide an API to interact with some features provided by [neutrinoapi.com](https://www.neutrinoapi.com).

//! # What is neutrinoapi.com
//! A general-purpose tool that solves recurring problems encountered during the development of software systems. It is used across many industries by software developers, data scientists and systems operators.

//! # How to use the neutral crate ?
//! The [Neutral](./struct.Neutral.html) structure act as an API client of neutrinoapi.
//! Features are represented by modules, each module contains a struct which implement a `send` method to call neutrinoapi.com. Use an instance of [Neutral](./struct.Neutral.html) to interact with neutrinoapi.
//!
//!
//! Example for ip_info endpoint:
//!
//! ```ignore
//! let api_auth = ApiAuth::new("userid".to_string(), "apikey".to_string());
//! let neutral = Neutral::try_new("https://neutrinoapi.net", api_auth).unwrap();
//! let ip_info_response = neutral.ip_info().send(ip_addr).await.unwrap();
//! ```

use error::NeutrinoError;
use hlr_lookup::HlrLookup;
use http::{
    uri::{Authority, Scheme},
    StatusCode, Uri,
};

use hyper::{body::Bytes, client::HttpConnector, Body, Client, Request};
use hyper_tls::HttpsConnector;
use ip_blocklist::IpBlocklist;
use ip_info::IpInfo;
use ip_probe::IpProbe;
use phone_validate::PhoneValidate;
use secrecy::{ExposeSecret, Secret};

pub mod error;
pub mod hlr_lookup;
pub mod ip_blocklist;
pub mod ip_info;
pub mod ip_probe;
pub mod phone_validate;

use crate::error::Error;

#[cfg(test)]
use mockito;

/// Provide authorization credentials for neutrinoapi.com
#[derive(Debug, Clone)]
pub struct ApiAuth {
    user_id: Secret<String>,
    api_key: Secret<String>,
}

impl ApiAuth {
    /// Create a new instance of `ApiAuth` using your neutrinoapi.com credentials.
    pub fn new(user_id: String, api_key: String) -> Self {
        ApiAuth {
            user_id: Secret::new(user_id),
            api_key: Secret::new(api_key),
        }
    }
}

/// A client to consume features provided by neutrinoapi.com
#[derive(Debug, Clone)]
pub struct Neutral {
    pub(crate) uri: Uri,
    pub(crate) auth: ApiAuth,
    pub(crate) client: Client<HttpsConnector<HttpConnector>>,
}

impl<'a> Neutral {
    /// Create a new Neutral instance. Needs some credentials to be authorized.
    /// Provide your neutrinoapi.com userid and apikey with an instance of `ApiAuth` as argument.
    pub fn try_new(uri: &str, auth: ApiAuth) -> Result<Self, Error> {
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

    pub(crate) fn request_builder(
        &self,
        path_and_query: String,
    ) -> Result<http::request::Builder, Error> {
        let uri = self.uri_builder().path_and_query(path_and_query).build()?;
        let request_builder = Request::builder()
            .uri(uri)
            .header("user-id", self.auth.user_id.expose_secret())
            .header("api-key", self.auth.api_key.expose_secret());
        Ok(request_builder)
    }

    pub(crate) async fn request(&self, req: Request<Body>) -> Result<Bytes, Error> {
        let http_resp = self.client.request(req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let body = hyper::body::to_bytes(http_resp.into_body()).await?;
                Ok(body)
            }
            _ => {
                let status_code = http_resp.status();
                let body = hyper::body::to_bytes(http_resp.into_body()).await?;
                let error = String::from_utf8_lossy(&body).into_owned();
                Err(Error::Neutrino(NeutrinoError { status_code, error }))
            }
        }
    }

    /// Returns an instance of PhoneValidate
    pub fn phone_validate(&'a self) -> PhoneValidate<'a> {
        PhoneValidate { neutral: self }
    }

    /// Returns an instance of IpInfo
    pub fn ip_info(&'a self) -> IpInfo<'a> {
        IpInfo { neutral: self }
    }

    /// Returns an instance of IpBlocklist
    pub fn ip_blocklist(&'a self) -> IpBlocklist<'a> {
        IpBlocklist { neutral: self }
    }

    /// Returns an instance of IpProbe
    pub fn ip_probe(&'a self) -> IpProbe<'a> {
        IpProbe { neutral: self }
    }

    /// Returns an instance of HlrLookup
    pub fn hlr_lookup(&'a self) -> HlrLookup<'a> {
        HlrLookup { neutral: self }
    }
}
