//! # Hlr lookup module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/hlr-lookup):
//!
//! Connect to the global mobile cellular network and retrieve the status of a mobile device.
//!
//! The home location register (HLR) is a central database that contains details of each mobile phone subscriber connected to the global mobile network. You can use this API to validate that a mobile number is live and registered on a mobile network in real-time. Find out the carrier name, ported number status and fetch up-to-date device status information.

use crate::{error::NeutrinoError, Neutral, PhoneInfoKind};
use http::{Method, StatusCode};
use hyper::Body;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[cfg(test)]
use mockito;

/// Describes Hlr Status from Hrl lookup API
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "kebab-case"))]
pub enum HlrStatus {
    Ok,
    Absent,
    Unknown,
    Invalid,
    FixedLine,
    Voip,
    Failed,
}

/// Response of hlr lookup neutrinoapi.com endpoint
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HlrLookupResponse {
    #[serde(rename(deserialize = "number_valid"))]
    pub is_valid: bool,
    #[serde(rename(deserialize = "hlr_valid"))]
    pub is_hlr_valid: bool,
    pub hlr_status: HlrStatus,
    pub is_mobile: bool,
    pub is_ported: bool,
    pub is_roaming: bool,
    pub imsi: String,
    pub mcc: String,
    pub mnc: String,
    pub msin: String,
    pub msc: String,
    pub current_network: String,
    pub origin_network: String,
    pub ported_network: String,
    #[serde(rename(deserialize = "number_type"))]
    pub kind: PhoneInfoKind,
    pub location: String,
    pub country: String,
    pub country_code: String,
    pub country_code3: String,
    pub currency_code: String,
    pub roaming_country_code: String,
    pub international_calling_code: String,
    pub international_number: String,
    pub local_number: String,
}

/// Send an hlr lookup request to neutrinoapi.com
pub async fn send(neutral: &Neutral<'_>, number: String) -> Result<HlrLookupResponse, Error> {
    let path_and_query = format!(
        "/hlr-lookup?output-case=snake&number={}",
        number.replace('+', "")
    );
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
            let response: HlrLookupResponse = serde_json::from_slice(&body)?;
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

        let neutral =
            Neutral::try_new(&mockito::server_url(), ApiAuth::new("User", "test")).unwrap();

        for test in &tests {
            let Args { phone_number } = &test.args;
            let hlr_lookup_result = send(&neutral, phone_number.to_owned()).await;
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
