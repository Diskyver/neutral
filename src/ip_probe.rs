//! # Ip probe module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/ip-probe):
//! Execute a realtime network probe against an IPv4 or IPv6 address.
//!
//! This API will run a series of live network scans and service probes to extract useful details about the host provider.

use http::{Method, StatusCode};
use hyper::Body;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use crate::{
    error::{Error, NeutrinoError},
    Neutral, NeutrinoProviderKind,
};

#[cfg(test)]
use mockito;

/// Response of ip probe neutrinoapi.com endpoint
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IpProbeResponse {
    pub country: String,
    pub country_code: String,
    pub provider_domain: String,
    pub city: String,
    pub vpn_domain: String,
    pub is_vpn: bool,
    pub as_cidr: String,
    #[serde(alias = "valid", alias = "is_valid")]
    pub is_valid: bool,
    pub provider_type: NeutrinoProviderKind,
    pub hostname: String,
    pub as_age: i64,
    pub continent_code: String,
    pub is_bogon: bool,
    pub ip: IpAddr,
    pub as_country_code: String,
    pub provider_description: String,
    pub as_country_code3: String,
    pub is_v4_mapped: bool,
    pub is_isp: bool,
    pub provider_website: String,
    pub as_description: String,
    pub is_hosting: bool,
    pub as_domains: Vec<String>,
    pub host_domain: String,
    pub is_proxy: bool,
    pub currency_code: String,
    pub region: String,
    pub asn: String,
    pub country_code3: String,
    pub is_v6: bool,
}

/// Send an ip probe request to neutrinoapi.com
pub async fn send(neutral: &Neutral<'_>, ip_addr: IpAddr) -> Result<IpProbeResponse, Error> {
    let path_and_query = format!("/ip-probe?output-case=snake&ip={}", ip_addr.to_string());
    let request = neutral
        .request_builder(path_and_query)?
        .method(Method::GET)
        .body(Body::empty())?;

    let client = &neutral.client;
    let request = neutral.add_authentication_headers(request);

    let http_resp = client.request(request).await?;

    match http_resp.status() {
        StatusCode::OK => {
            let body = hyper::body::to_bytes(http_resp.into_body()).await?;
            let response: IpProbeResponse = serde_json::from_slice(&body)?;
            Ok(response)
        }
        _ => {
            let status_code = http_resp.status();
            let body = hyper::body::to_bytes(http_resp.into_body()).await?;
            let error = String::from_utf8_lossy(&body).into_owned();
            Err(Error::Neutrino(NeutrinoError { status_code, error }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ApiAuth;
    use mockito::{mock, Matcher};
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_ip_probe_with_good_ip() {
        let body_resp = r#"
        {
            "country": "ACountry",
            "country_code": "AC",
            "provider_domain": "networkoperator.com",
            "city": "Roubaix",
            "vpn_domain": "",
            "is_vpn": false,
            "as_cidr": "128.0.0.0/22",
            "valid": true,
            "provider_type": "isp",
            "hostname": "",
            "as_age": 8,
            "continent_code": "EU",
            "is_bogon": false,
            "ip": "128.0.0.1",
            "as_country_code": "AC",
            "provider_description": "A network operator description",
            "as_country_code3": "ACO",
            "is_v4_mapped": false,
            "is_isp": true,
            "provider_website": "https://www.networkoperator.com/",
            "as_description": "NETWORK-OPERATOR-AS,AC,Network Operator",
            "is_hosting": false,
            "as_domains": [
              "networkoperator.com"
            ],
            "host_domain": "",
            "is_proxy": false,
            "currency_code": "ABC",
            "region": "Hauts-de-ACountry",
            "asn": "12345",
            "country_code3": "ACO",
            "is_v6": false
          }
        "#;

        let _m = mock("GET", "/ip-probe")
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
            pub expected: &'a IpProbeResponse,
        }

        let expected_response = IpProbeResponse {
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
            hostname: "".to_owned(),
            host_domain: "".to_owned(),
            provider_description: "A network operator description".to_owned(),
            provider_website: "https://www.networkoperator.com/".to_owned(),
            provider_domain: "networkoperator.com".to_owned(),
            provider_type: NeutrinoProviderKind::Isp,
            is_hosting: false,
            is_isp: true,
            is_vpn: false,
            is_proxy: false,
            vpn_domain: "".to_owned(),
            asn: "12345".to_owned(),
            as_cidr: "128.0.0.0/22".to_owned(),
            as_domains: vec!["networkoperator.com".to_owned()],
            as_description: "NETWORK-OPERATOR-AS,AC,Network Operator".to_owned(),
            as_age: 8,
            as_country_code: "AC".to_owned(),
            as_country_code3: "ACO".to_owned(),
        };

        let tests = vec![TestingData {
            name: "Using an IP v4".to_owned(),
            args: Args {
                ip_addr: IpAddr::V4(Ipv4Addr::new(128, 0, 0, 1)),
            },
            expected: &expected_response,
        }];

        let neutral =
            Neutral::try_new(&mockito::server_url(), ApiAuth::new("User", "test")).unwrap();

        for test in &tests {
            let Args { ip_addr } = test.args;
            let ip_probe_res = send(&neutral, ip_addr).await;
            let ip_probe_result = ip_probe_res.map(|ip_probe| ip_probe.clone());
            let expected = test.expected;

            assert_eq!(
                expected.clone().to_owned(),
                ip_probe_result.unwrap(),
                "{}",
                test.name
            )
        }
    }
}
