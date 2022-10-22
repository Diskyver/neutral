//! # Phone validate module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/phone-validate):
//!
//! Parse, validate and get location information about a phone number.
//!
//! Use this API to validate local and international phone numbers in any country. You can determine the location of the number and also reformat the number into local and international dialing formats.

use http::{Method, StatusCode};
use hyper::Body;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, NeutrinoError},
    Neutral, PhoneInfoKind,
};

#[cfg(test)]
use mockito;

/// Response of phone validate neutrinoapi.com endpoint
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PhoneValidateResponse {
    #[serde(alias = "valid", alias = "is_valid")]
    pub is_valid: bool,
    #[serde(alias = "type", alias = "kind")]
    pub kind: PhoneInfoKind,
    pub international_calling_code: String,
    pub international_number: String,
    pub local_number: String,
    pub location: String,
    pub country: String,
    pub country_code: String,
    pub country_code3: String,
    pub currency_code: String,
    pub is_mobile: bool,
    pub prefix_network: String,
}

/// Send an phone validate request to neutrinoapi.com
pub async fn send(
    neutral: &Neutral<'_>,
    phone_number: String,
) -> Result<PhoneValidateResponse, Error> {
    let path_and_query = format!(
        "/phone-validate?output-case=snake&number={}",
        phone_number.replace('+', "")
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
            let response: PhoneValidateResponse = serde_json::from_slice(&body)?;
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
    async fn test_phone_validate_with_good_phone_number() {
        let body_resp = r#"
            {
                "valid":true,
                "type":"mobile",
                "international_calling_code":"33",
                "international_number":"+12345678901",
                "local_number":"01 23 45 67 89",
                "location":"ACountry",
                "country":"ACountry",
                "country_code":"AC",
                "country_code3":"ACO",
                "currency_code":"ABC",
                "is_mobile":true,
                "prefix_network":"Phone operator"
            }"#;

        let _m = mock("GET", "/phone-validate")
            .match_query(Matcher::Regex("number=12345678901".into()))
            .match_query(Matcher::Regex("number=+12345678901".into()))
            .with_status(200)
            .with_body(body_resp)
            .create();

        struct Args {
            pub phone_number: String,
        }

        struct TestingData<'a> {
            pub name: String,
            pub args: Args,
            pub expected: &'a PhoneValidateResponse,
        }

        let expected_response = PhoneValidateResponse {
            is_valid: true,
            kind: PhoneInfoKind::Mobile,
            international_calling_code: "33".to_owned(),
            international_number: "+12345678901".to_owned(),
            local_number: "01 23 45 67 89".to_owned(),
            location: "ACountry".to_owned(),
            country: "ACountry".to_owned(),
            country_code: "AC".to_owned(),
            country_code3: "ACO".to_owned(),
            currency_code: "ABC".to_owned(),
            is_mobile: true,
            prefix_network: "Phone operator".to_owned(),
        };

        let tests = vec![
            TestingData {
                name: "Using a phone number starting with +".to_owned(),
                args: Args {
                    phone_number: "+12345678901".to_owned(),
                },
                expected: &expected_response,
            },
            TestingData {
                name: "Using a phone number without + sign at start position".to_owned(),
                args: Args {
                    phone_number: "12345678901".to_owned(),
                },
                expected: &expected_response,
            },
        ];

        let neutral =
            Neutral::try_new(&mockito::server_url(), ApiAuth::new("user", "test")).unwrap();

        for test in &tests {
            let Args { phone_number } = &test.args;
            let validate_phone_result = send(&neutral, phone_number.to_owned()).await;
            let phone_info_result = validate_phone_result.map(|phone_info| phone_info.clone());
            let expected = test.expected;

            assert_eq!(
                expected.clone().to_owned(),
                phone_info_result.unwrap(),
                "{}",
                test.name
            )
        }
    }
}
