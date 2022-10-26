//! # Hlr lookup module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/hlr-lookup):
//!
//! Connect to the global mobile cellular network and retrieve the status of a mobile device.
//!
//! The home location register (HLR) is a central database that contains details of each mobile phone subscriber connected to the global mobile network. You can use this API to validate that a mobile number is live and registered on a mobile network in real-time. Find out the carrier name, ported number status and fetch up-to-date device status information.

use crate::{Error, Neutral};
use http::Method;
use hyper::Body;
use neutral_types::hlr_lookup::HlrLookupResponse;

#[cfg(test)]
use mockito;

pub struct HlrLookup<'a> {
    pub(crate) neutral: &'a Neutral,
}

impl<'a> HlrLookup<'a> {
    /// Send an hlr lookup request to neutrinoapi.com
    pub async fn send(&self, phone_number: String) -> Result<HlrLookupResponse, Error> {
        let path_and_query = format!(
            "/hlr-lookup?output-case=snake&number={}",
            phone_number.replace('+', "")
        );

        let request = self
            .neutral
            .request_builder(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())?;

        let body = self.neutral.request(request).await?;
        let response: HlrLookupResponse = serde_json::from_slice(&body)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ApiAuth;
    use mockito::{mock, Matcher};
    use neutral_types::{hlr_lookup::HlrStatus, PhoneInfoKind};

    #[tokio::test]
    async fn test_hlr_lookup_with_good_phone_number() {
        let body_resp = r#"
        {
            "country": "ACountry",
            "country_code": "AC",
            "country_code3": "ACO",
            "currency_code": "ABC",
            "current_network": "Phone operator",
            "hlr_status": "ok",
            "hlr_valid": true,
            "imsi": "2081594584",
            "international_calling_code": "33",
            "international_number": "+12345678901",
            "is_mobile": true,
            "is_ported": false,
            "is_roaming": false,
            "local_number": "01 23 45 67 89",
            "location": "ACountry",
            "mcc": "208",
            "mnc": "15",
            "msc": "320433",
            "msin": "",
            "number_type": "mobile",
            "number_valid": true,
            "origin_network": "Phone operator",
            "ported_network": "",
            "roaming_country_code": ""
        }"#;

        let _m = mock("GET", "/hlr-lookup")
            .match_query(Matcher::Regex("number=12345678901".into()))
            .with_status(200)
            .with_body(body_resp)
            .create();

        struct Args {
            pub phone_number: String,
        }

        struct TestingData<'a> {
            pub name: String,
            pub args: Args,
            pub expected: &'a HlrLookupResponse,
        }

        let expected_response = HlrLookupResponse {
            country: "ACountry".to_owned(),
            country_code: "AC".to_owned(),
            country_code3: "ACO".to_owned(),
            currency_code: "ABC".to_owned(),
            current_network: "Phone operator".to_owned(),
            hlr_status: HlrStatus::Ok,
            is_hlr_valid: true,
            imsi: "2081594584".to_owned(),
            international_calling_code: "33".to_owned(),
            international_number: "+12345678901".to_owned(),
            is_mobile: true,
            is_ported: false,
            is_roaming: false,
            local_number: "01 23 45 67 89".to_owned(),
            location: "ACountry".to_owned(),
            mcc: "208".to_owned(),
            mnc: "15".to_owned(),
            msc: "320433".to_owned(),
            msin: "".to_owned(),
            kind: PhoneInfoKind::Mobile,
            is_valid: true,
            origin_network: "Phone operator".to_owned(),
            ported_network: "".to_owned(),
            roaming_country_code: "".to_owned(),
        };

        let tests = vec![TestingData {
            name: "Using a phone number without + sign at start position".to_owned(),
            args: Args {
                phone_number: "12345678901".to_owned(),
            },
            expected: &expected_response,
        }];

        let neutral = Neutral::try_new(
            &mockito::server_url(),
            ApiAuth::new("User".to_string(), "test".to_string().to_string()),
        )
        .unwrap();

        for test in &tests {
            let Args { phone_number } = &test.args;
            let hlr_lookup_result = neutral.hlr_lookup().send(phone_number.to_owned()).await;
            let result = hlr_lookup_result.map(|hlr_lookup| hlr_lookup.clone());
            let expected = test.expected;

            assert_eq!(
                expected.clone().to_owned(),
                result.unwrap(),
                "{}",
                test.name
            )
        }
    }
}
