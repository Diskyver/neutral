//! # Phone validate module
//! Quoted from [neutrinoapi.com](https://www.neutrinoapi.com/api/phone-validate):
//!
//! Parse, validate and get location information about a phone number.
//!
//! Use this API to validate local and international phone numbers in any country. You can determine the location of the number and also reformat the number into local and international dialing formats.

use http::Method;
use hyper::Body;
use neutral_types::phone_validate::PhoneValidateResponse;

use crate::{error::Error, Neutral};

#[cfg(test)]
use mockito;

pub struct PhoneValidate<'a> {
    pub(crate) neutral: &'a Neutral,
}

impl<'a> PhoneValidate<'a> {
    /// Send an phone validate request to neutrinoapi.com
    pub async fn send(&self, phone_number: String) -> Result<PhoneValidateResponse, Error> {
        let path_and_query = format!(
            "/phone-validate?output-case=snake&number={}",
            phone_number.replace('+', "")
        );

        let request = self
            .neutral
            .request_builder(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())?;

        let body = self.neutral.request(request).await?;
        let response: PhoneValidateResponse = serde_json::from_slice(&body)?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ApiAuth;
    use mockito::{mock, Matcher};
    use neutral_types::PhoneInfoKind;

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

        let neutral = Neutral::try_new(
            &mockito::server_url(),
            ApiAuth::new("User".to_string(), "test".to_string()),
        )
        .unwrap();

        for test in &tests {
            let Args { phone_number } = &test.args;
            let validate_phone_result =
                neutral.phone_validate().send(phone_number.to_owned()).await;
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
