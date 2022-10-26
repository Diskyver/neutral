//! # Ip info module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/ip-info):
//!
//! Get location information about an IP address and do reverse DNS (PTR) lookups.
//!
//! Identify the geolocation of an IP address down to the city level, including the geographic coordinates (latitude, longitude) and detailed locale information. Our geolocation database is continuously updated in realtime as Internet address allocation changes and as new IP ranges come online. The API supports both IPv4 and IPv6.
//!
//! Use this API for:
//!
//! * Application personalization
//! * Locale detection (timezone, currency)
//! * Geo-targeting
//! * Geo-fencing
//! * Ad targeting
//! * Fraud analysis
//! * Traffic analysis
//! * Access controls

use http::Method;
use hyper::Body;
use neutral_types::ip_info::IpInfoResponse;
use std::net::IpAddr;

use crate::{Error, Neutral};

#[cfg(test)]
use mockito;

pub struct IpInfo<'a> {
    pub(crate) neutral: &'a Neutral,
}

impl<'a> IpInfo<'a> {
    /// Send an ip info request to neutrinoapi.com
    pub async fn send(&self, ip_addr: IpAddr) -> Result<IpInfoResponse, Error> {
        let path_and_query = format!("/ip-info?output-case=snake&ip={}", ip_addr.to_string());
        let request = self
            .neutral
            .request_builder(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())?;

        let body = self.neutral.request(request).await?;
        let response: IpInfoResponse = serde_json::from_slice(&body)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ApiAuth;
    use mockito::{mock, Matcher};
    use neutral_types::NeutrinoTimeZoneResponse;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_ip_info_with_good_ip() {
        let body_resp = r#"
        {
            "ip": "128.0.0.1",
            "valid": true,
            "is_v6": false,
            "is_v4_mapped": false,
            "is_bogon": false,
            "country": "ACountry",
            "country_code": "AC",
            "country_code3": "ACO",
            "continent_code": "EU",
            "currency_code": "ABC",
            "city": "Roubaix",
            "region": "Hauts-de-ACountry",
            "longitude": 1.00000,
            "latitude": 1.00000,
            "hostname":"",
            "host_domain":"",
            "timezone": {
                "id": "Europe/Paris",
                "name": "Central European Standard Time",
                "abbr": "CET",
                "date": "2021-11-24",
                "time": "12:47:33.825588",
                "offset": "+01:00"

            }
          }
        "#;

        let _m = mock("GET", "/ip-info")
            .match_query(Matcher::Regex("ip=128.0.0.1".into()))
            .with_status(200)
            .with_body(body_resp)
            .create();

        struct Args {
            pub ip_addr: IpAddr,
        }

        struct TestingData<'a> {
            pub name: String,
            pub args: Args,
            pub expected: &'a IpInfoResponse,
        }

        let expected_response = IpInfoResponse {
            ip: IpAddr::V4(Ipv4Addr::new(128, 0, 0, 1)),
            is_valid: true,
            is_v6: false,
            is_v4_mapped: false,
            is_bogon: false,
            country: "ACountry".to_owned(),
            country_code: "AC".to_owned(),
            country_code3: "ACO".to_owned(),
            continent_code: "EU".to_owned(),
            currency_code: "ABC".to_owned(),
            city: "Roubaix".to_owned(),
            region: "Hauts-de-ACountry".to_owned(),
            longitude: 1.00000,
            latitude: 1.00000,
            hostname: "".to_owned(),
            host_domain: "".to_owned(),
            timezone: Some(NeutrinoTimeZoneResponse {
                id: "Europe/Paris".to_owned(),
                name: "Central European Standard Time".to_owned(),
                abbr: "CET".to_owned(),
                date: "2021-11-24".to_owned(),
                time: "12:47:33.825588".to_owned(),
                offset: "+01:00".to_owned(),
            }),
        };

        let tests = vec![TestingData {
            name: "Using an IP v4".to_owned(),
            args: Args {
                ip_addr: IpAddr::V4(Ipv4Addr::new(128, 0, 0, 1)),
            },
            expected: &expected_response,
        }];

        let neutral = Neutral::try_new(
            "http://localhost:1234",
            ApiAuth::new("User".to_string(), "test".to_string()),
        )
        .unwrap();

        for test in &tests {
            let Args { ip_addr } = test.args;
            let ip_info_res = neutral.ip_info().send(ip_addr).await;
            let ip_info_result = ip_info_res.map(|ip_info| ip_info.clone());
            let expected = test.expected;

            assert_eq!(
                expected.clone().to_owned(),
                ip_info_result.unwrap(),
                "{}",
                test.name
            )
        }
    }
}
