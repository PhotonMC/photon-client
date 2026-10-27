//! The profile lookups of a head that names its owner by name or id only
//! (`ProfileResolver.fetchByName` and `fetchById`): the account services answer the name's id,
//! and the session server the profile with its signed properties.

use std::time::Duration;

use photon_core::binary::Uuid;
use photon_core::json;
use photon_protocol::play::ProfileProperty;

/// `YggdrasilEnvironment.PROD`'s account and session hosts.
const ACCOUNT_HOST: &str = "https://api.mojang.com";
const SESSION_HOST: &str = "https://sessionserver.mojang.com";

/// A profile as the session server tells of it.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchedProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into()
}

fn get(agent: &ureq::Agent, url: &str) -> Option<String> {
    agent.get(url).call().ok()?.body_mut().read_to_string().ok()
}

/// A UUID written as 32 hex digits, or as the dashed form.
pub fn parse_uuid(text: &str) -> Option<Uuid> {
    let digits: String = text.chars().filter(|c| *c != '-').collect();
    if digits.len() != 32 {
        return None;
    }
    Some(Uuid::new(
        u64::from_str_radix(&digits[..16], 16).ok()?,
        u64::from_str_radix(&digits[16..], 16).ok()?,
    ))
}

/// `GameProfile` of the session server's answer: the id, the name and the properties.
pub fn parse_profile(body: &str) -> Option<FetchedProfile> {
    let object = json::parse_object(body)?;
    let mut properties = Vec::new();
    for entry in object.array("properties").unwrap_or_default() {
        let json::JsonValue::Object(entry) = entry else {
            return None;
        };
        properties.push(ProfileProperty {
            name: entry.text("name")?,
            value: entry.text("value")?,
            signature: entry.text("signature"),
        });
    }
    Some(FetchedProfile {
        id: parse_uuid(&object.text("id")?)?,
        name: object.text("name")?,
        properties,
    })
}

/// The profile of the account with this id; none where it does not exist or the request fails.
pub fn fetch_by_id(id: Uuid, timeout: Duration) -> Option<FetchedProfile> {
    let dashless = format!("{:016x}{:016x}", id.most, id.least);
    let url = format!("{SESSION_HOST}/session/minecraft/profile/{dashless}?unsigned=false");
    parse_profile(&get(&agent(timeout), &url)?)
}

/// The profile of the account with this name.
pub fn fetch_by_name(name: &str, timeout: Duration) -> Option<FetchedProfile> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let found = get(
        &agent(timeout),
        &format!("{ACCOUNT_HOST}/users/profiles/minecraft/{name}"),
    )?;
    fetch_by_id(
        parse_uuid(&json::parse_object(&found)?.text("id")?)?,
        timeout,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_answer_gives_the_id_name_and_properties() {
        let body = r#"{"id":"853c80ef3c3749fdaa49938b674adae6","name":"jeb_",
            "properties":[{"name":"textures","value":"abc","signature":"sig"}]}"#;
        let profile = parse_profile(body).unwrap();
        assert_eq!(profile.name, "jeb_");
        assert_eq!(
            profile.id,
            Uuid::new(0x853c_80ef_3c37_49fd, 0xaa49_938b_674a_dae6)
        );
        assert_eq!(profile.properties[0].signature.as_deref(), Some("sig"));
    }

    #[test]
    fn a_uuid_is_read_with_or_without_dashes() {
        let dashed = parse_uuid("853c80ef-3c37-49fd-aa49-938b674adae6");
        assert_eq!(dashed, parse_uuid("853c80ef3c3749fdaa49938b674adae6"));
        assert!(dashed.is_some());
        assert_eq!(parse_uuid("zz"), None);
    }

    #[test]
    fn a_name_that_is_not_a_player_name_is_not_asked_for() {
        assert_eq!(fetch_by_name("a/b", Duration::from_millis(1)), None);
    }
}
