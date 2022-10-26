//! # Ip blocklist module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/ip-blocklist):
//!
//! The IP Blocklist API will detect potentially malicious or dangerous IP addresses.
//!
//! Use this API for identifying malicious hosts, anonymous proxies, tor, botnets, spammers and more.
//! Block, filter or flag traffic to help reduce attacks on your networks and software stacks. IP addresses are automatically removed from the blocklist after 7 days provided no other malicious activity is detected.
//!
//! You can also download the complete IP data for direct use on your own systems using the Download API.
//!
//! IP blocklist will detect the following main categories of IP addresses:
//!
//! * Open proxies
//! * Tor nodes
//! * Public VPNs
//! * Spam hosts
//! * Phishing hosts
//! * Malware servers
//! * Attack sources
//! * Criminal netblocks
//! * Malicious spiders
//! * Bots and botnets
//! * Exploit scanners
//! * Brute-force crackers

use crate::Neutral;
use http::Method;
use hyper::Body;
use neutral_types::ip_blocklist::IpBlocklistResponse;
use std::net::IpAddr;

use crate::error::Error;

#[cfg(test)]
use mockito;

pub struct IpBlocklist<'a> {
    pub(crate) neutral: &'a Neutral,
}

impl<'a> IpBlocklist<'a> {
    /// Send an ip blocklist request to neutrinoapi.com
    pub async fn send(&self, ip_addr: IpAddr) -> Result<IpBlocklistResponse, Error> {
        let path_and_query = format!(
            "/ip-blocklist?output-case=snake&ip={}&vpn-lookup=true",
            ip_addr.to_string()
        );

        let request = self
            .neutral
            .request_builder(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())?;

        let body = self.neutral.request(request).await?;
        let response: IpBlocklistResponse = serde_json::from_slice(&body)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ApiAuth;
    use mockito::{mock, Matcher};
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_ip_blocking_with_good_ip() {
        let body_resp = r#"
            {
                "ip": "128.0.0.1",
                "is_listed": false,
                "last_seen": 0,
                "list_count": 0,
                "blocklists": [],
                "sensors": [],
                "is_proxy": false,
                "is_tor": false,
                "is_vpn": false,
                "is_malware": false,
                "is_spyware": false,
                "is_dshield": false,
                "is_hijacked": false,
                "is_spider": false,
                "is_bot": false,
                "is_spam_bot": false,
                "is_exploit_bot": false
            }
        "#;

        let _m = mock("GET", "/ip-blocklist")
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
            pub expected: &'a IpBlocklistResponse,
        }

        let expected_response = IpBlocklistResponse {
            ip: IpAddr::V4(Ipv4Addr::new(128, 0, 0, 1)),
            is_listed: false,
            last_seen: 0,
            list_count: 0,
            blocklists: vec![],
            sensors: vec![],
            is_proxy: false,
            is_tor: false,
            is_vpn: false,
            is_malware: false,
            is_spyware: false,
            is_dshield: false,
            is_hijacked: false,
            is_spider: false,
            is_bot: false,
            is_spam_bot: false,
            is_exploit_bot: false,
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
            let ip_blocklist_res = neutral.ip_blocklist().send(ip_addr).await;
            let ip_blocklist_result = ip_blocklist_res.map(|ip_blocklist| ip_blocklist.clone());
            let expected = test.expected;

            assert_eq!(
                expected.clone().to_owned(),
                ip_blocklist_result.unwrap(),
                "{}",
                test.name
            )
        }
    }
}
