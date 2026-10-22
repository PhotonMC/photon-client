//! The login encryption operations of the wire oracle: server hash, CFB8 stream, key packet.

use photon_protocol::crypto::{self, StreamCipher};
use photon_protocol::login::ServerboundKeyPacket;
use serde_json::{Value, json};

use crate::hex::{decode_hex, hex, login_read_error, string, utf16_hex_units};

fn secret_field(request: &Value) -> Result<[u8; crypto::SECRET_LENGTH], String> {
    decode_hex(string(request, "secret_hex")?)?
        .try_into()
        .map_err(|_| "secret_hex must be sixteen bytes".to_owned())
}

pub(crate) fn crypto_server_hash_result(request: &Value) -> Result<Value, String> {
    let server_id = utf16_hex_units(
        string(request, "server_id_utf16_hex")?,
        "server_id_utf16_hex",
    )?;
    let secret = secret_field(request)?;
    let key = decode_hex(string(request, "public_key_hex")?)?;
    Ok(json!({"hash": crypto::server_hash(&server_id, &secret, &key)}))
}

pub(crate) fn crypto_stream_result(request: &Value) -> Result<Value, String> {
    let secret = secret_field(request)?;
    let mut cipher = match string(request, "mode")? {
        "encrypt" => StreamCipher::encryptor(&secret),
        _ => StreamCipher::decryptor(&secret),
    };
    let mut out = String::new();
    for chunk in request["chunks_hex"].as_array().ok_or("chunks_hex")? {
        let mut bytes = decode_hex(chunk.as_str().ok_or("chunk")?)?;
        cipher.apply(&mut bytes);
        out.push_str(&hex(&bytes));
    }
    Ok(json!({"hex": out}))
}

pub(crate) fn login_key_result(request: &Value) -> Result<Value, String> {
    let bytes = decode_hex(string(request, "hex")?)?;
    let mut remaining = bytes.as_slice();
    let mut result = match ServerboundKeyPacket::decode(&mut remaining) {
        Ok(packet) => {
            let mut rewritten = Vec::new();
            packet
                .encode(&mut rewritten)
                .map_err(|_| "key packet encode".to_owned())?;
            json!({"rewritten_hex": hex(&rewritten), "outcome": "value"})
        }
        Err(error) => json!({"outcome": "error", "error": login_read_error(error)}),
    };
    result["consumed"] = json!(bytes.len() - remaining.len());
    result["remaining_hex"] = json!(hex(remaining));
    Ok(result)
}
