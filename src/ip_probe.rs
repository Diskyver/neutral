//! # Ip probe module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/ip-probe):
//! Execute a realtime network probe against an IPv4 or IPv6 address.
//!
//! This API will run a series of live network scans and service probes to extract useful details about the host provider.

use http::Method;
use hyper::Body;
use neutral_types::ip_probe::IpProbeResponse;
use std::net::IpAddr;

use crate::{Error, Neutral};

#[cfg(test)]
use mockito;

pub struct IpProbe<'a> {
    pub(crate) neutral: &'a Neutral,
}

impl<'a> IpProbe<'a> {
    /// Send an ip probe request to neutrinoapi.com
    pub async fn send(&self, ip_addr: IpAddr) -> Result<IpProbeResponse, Error> {
        let path_and_query = format!("/ip-probe?output-case=snake&ip={}", ip_addr.to_string());
        let request = self
            .neutral
            .request_builder(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())?;

        let body = self.neutral.request(request).await?;
        let response: IpProbeResponse = serde_json::from_slice(&body)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ApiAuth;
    use mockito::{mock, Matcher};
    use neutral_types::NeutrinoProviderKind;
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

        let neutral = Neutral::try_new(
            &mockito::server_url(),
            ApiAuth::new("User".to_string(), "test".to_string()),
        )
        .unwrap();

        for test in &tests {
            let Args { ip_addr } = test.args;
            let ip_probe_res = neutral.ip_probe().send(ip_addr).await;
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
