//! The keys that sign profile properties (`YggdrasilServicesKeyInfo`): the client asks the
//! services host for them, nothing is bundled.

use std::time::Duration;

use photon_core::{base64, json};

/// `YggdrasilEnvironment.PROD`'s services host, which serves `/publickeys`.
const SERVICES_HOST: &str = "https://api.minecraftservices.com";

/// The DER public keys that sign profile properties, from the services host; none where the
/// request or its answer fails (a profile then checks as invalid, as with an empty key set).
pub fn fetch_property_keys(timeout: Duration) -> Option<Vec<Vec<u8>>> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into();
    let mut response = agent
        .get(format!("{SERVICES_HOST}/publickeys"))
        .call()
        .ok()?;
    parse_property_keys(&response.body_mut().read_to_string().ok()?)
}

/// `KeySetResponse.profilePropertyKeys`: each entry's `publicKey` is base64 of the DER key.
pub fn parse_property_keys(body: &str) -> Option<Vec<Vec<u8>>> {
    let keys = json::parse_object(body)?;
    let mut parsed = Vec::new();
    for entry in keys.array("profilePropertyKeys")? {
        let json::JsonValue::Object(entry) = entry else {
            return None;
        };
        parsed.push(base64::decode(&entry.text("publicKey")?)?);
    }
    Some(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_property_keys_are_read_from_the_key_set() {
        let body = r#"{"profilePropertyKeys":[{"publicKey":"AQID"},{"publicKey":"BAUG"}],
            "playerCertificateKeys":[{"publicKey":"Bw=="}]}"#;
        assert_eq!(
            parse_property_keys(body),
            Some(vec![vec![1, 2, 3], vec![4, 5, 6]])
        );
        assert_eq!(parse_property_keys("{}"), None);
        assert_eq!(parse_property_keys("not json"), None);
    }
}
