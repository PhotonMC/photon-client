use photon_protocol::crypto::SystemRandom;
use rsa::pkcs8::EncodePublicKey;
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use sha1::{Digest, Sha1};

use super::*;

const SHA1_INFO: [u8; 15] = [
    0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04, 0x14,
];

fn payload(url: &str, model: &str) -> String {
    base64::encode(
        format!(
            r#"{{"timestamp":1,"profileId":"x","profileName":"A","textures":{{"SKIN":{{"url":"{url}","metadata":{{"model":"{model}"}}}},"CAPE":{{"url":"{url}"}}}}}}"#
        )
        .as_bytes(),
    )
}

fn signed(value: &str, private: &RsaPrivateKey) -> ProfileProperty {
    let mut info = SHA1_INFO.to_vec();
    info.extend(Sha1::digest(value.as_bytes()));
    let signature = private
        .sign(Pkcs1v15Sign::new_unprefixed(), &info)
        .expect("sign");
    ProfileProperty {
        name: "textures".into(),
        value: value.into(),
        signature: Some(base64::encode(&signature)),
    }
}

fn keypair() -> (RsaPrivateKey, Vec<u8>) {
    let private = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
    let der = private
        .to_public_key()
        .to_public_key_der()
        .unwrap()
        .to_vec();
    (private, der)
}

const GOOD: &str = "http://textures.minecraft.net/texture/abc123";

#[test]
fn a_signed_property_gives_secure_textures() {
    let (private, der) = keypair();
    let property = signed(&payload(GOOD, "slim"), &private);
    let textures = unpack_textures(&property, &[vec![1, 2], der]);
    assert_eq!(textures.signature, SignatureState::Signed);
    assert!(textures.secure());
    let skin = textures.skin.expect("a skin");
    assert_eq!(skin.hash, "abc123");
    assert!(skin.is_slim());
    assert_eq!(textures.cape.expect("a cape").model, None);
    assert!(textures.elytra.is_none());
}

#[test]
fn a_tampered_or_unsigned_property_is_not_secure() {
    let (private, der) = keypair();
    let mut property = signed(&payload(GOOD, "default"), &private);
    property.value = payload(GOOD, "slim");
    let tampered = unpack_textures(&property, std::slice::from_ref(&der));
    assert_eq!(tampered.signature, SignatureState::Invalid);
    property.signature = None;
    let unsigned = unpack_textures(&property, &[der]);
    assert_eq!(unsigned.signature, SignatureState::Unsigned);
    assert!(!unsigned.secure());
    assert!(unsigned.skin.is_some());
}

#[test]
fn no_known_key_leaves_a_signature_invalid() {
    let (private, _) = keypair();
    let property = signed(&payload(GOOD, "default"), &private);
    assert_eq!(
        unpack_textures(&property, &[]).signature,
        SignatureState::Invalid
    );
}

#[test]
fn a_url_outside_the_allowed_domain_empties_the_textures() {
    for url in [
        "http://evil.example/texture/abc",
        "ftp://textures.minecraft.net/texture/abc",
        "http://Textures.minecraft.net/texture/abc",
        "http://textures.minecraft.net.evil.example/texture/abc",
        "http://textures.minecraft.net@evil.example/texture/abc",
        "textures.minecraft.net/texture/abc",
    ] {
        let property = ProfileProperty {
            name: "textures".into(),
            value: payload(url, "default"),
            signature: None,
        };
        assert_eq!(
            unpack_textures(&property, &[]),
            ProfileTextures::EMPTY,
            "{url}"
        );
    }
}

#[test]
fn the_domain_check_follows_the_authority_not_the_path() {
    assert!(is_allowed_texture_domain(
        "https://textures.minecraft.net:443/t/a"
    ));
    assert!(is_allowed_texture_domain(
        "http://user@textures.minecraft.net/t/a"
    ));
    assert!(!is_allowed_texture_domain(
        "http://evil.example/textures.minecraft.net"
    ));
    assert!(!is_allowed_texture_domain(
        "http://evil.example?@textures.minecraft.net/"
    ));
}

#[test]
fn the_hash_is_the_last_path_segment_without_extension() {
    assert_eq!(
        texture_hash("http://textures.minecraft.net/texture/abc"),
        "abc"
    );
    assert_eq!(
        texture_hash("http://textures.minecraft.net/texture/abc.png?x=1"),
        "abc"
    );
    assert_eq!(texture_hash("http://textures.minecraft.net"), "");
}

#[test]
fn an_unreadable_payload_is_empty() {
    for value in ["!!!", &base64::encode(b"not json"), &base64::encode(b"{}")] {
        let property = ProfileProperty {
            name: "textures".into(),
            value: value.into(),
            signature: None,
        };
        assert_eq!(unpack_textures(&property, &[]), ProfileTextures::EMPTY);
    }
}
