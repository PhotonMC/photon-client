//! The login encryption: the session server hash, the CFB8 stream and the key packet.

use serde_json::{Value, json};

use super::codec::{utf16_hex, varint_hex};
use crate::bits::hex;
use crate::case::case;

/// A throwaway 1024-bit RSA public key (X.509 DER) made for these cases.
const PUBLIC_KEY: &str = "30819f300d06092a864886f70d010101050003818d0030818902818100e3d25a406073198b94db1ff7335ae03fcebc2077236d0e59701772bc20d5f86c17c27ae96c248d4abfb967e0d0d9dd3fe51367d39e3eb4a5264bef9028fb62dee9ecf4a16bfc071406c152f54ba6d57b3dbe64e4bd54d92e26b3cb60efb8d2b260c18a59bec8ee4c8e182660c95ae546b180e8a8bb3b63be5a3eb43c04779b7f0203010001";

fn wire(id: String, operation: &str, fields: Value) -> Value {
    case(id, "wire", operation, fields)
}

fn secret(seed: u8) -> String {
    let bytes: Vec<u8> = (0..16u8).map(|index| seed.wrapping_mul(31).wrapping_add(index * 17)).collect();
    hex(&bytes)
}

/// Digests of both signs come out of different secrets; the sweep keeps both in the set.
fn server_hash() -> Vec<Value> {
    let ids = [
        ("empty", String::new()),
        ("short", utf16_hex("abcd")),
        ("latin1", utf16_hex("\u{e9}\u{e8}\u{ff}")),
        ("bmp", utf16_hex("\u{670d}x")),
        ("pair", utf16_hex("\u{1f600}")),
        ("lone-surrogate", "d83d".to_owned()),
    ];
    let mut cases = Vec::new();
    for (label, id) in &ids {
        for seed in 0..64u8 {
            let fields = json!({
                "server_id_utf16_hex": id, "secret_hex": secret(seed), "public_key_hex": PUBLIC_KEY,
            });
            cases.push(wire(format!("crypto/server-hash/{label}/{seed}"), "crypto_server_hash", fields));
        }
    }
    cases
}

fn stream() -> Vec<Value> {
    let data = hex(&(0..=255u8).cycle().take(300).collect::<Vec<u8>>());
    let splits: [(&str, Vec<String>); 4] = [
        ("whole", vec![data.clone()]),
        ("bytes", data.as_bytes().chunks(2).take(40).map(|pair| String::from_utf8_lossy(pair).into_owned()).collect()),
        ("uneven", vec![data[..6].to_owned(), data[6..70].to_owned(), data[70..].to_owned()]),
        ("empty-first", vec![String::new(), data[..40].to_owned()]),
    ];
    let mut cases = Vec::new();
    for mode in ["encrypt", "decrypt"] {
        for (label, chunks) in &splits {
            let fields = json!({"secret_hex": secret(3), "mode": mode, "chunks_hex": chunks});
            cases.push(wire(format!("crypto/stream/{mode}/{label}"), "crypto_stream", fields));
        }
    }
    cases
}

fn array(bytes: &[u8]) -> String {
    format!("{}{}", varint_hex(bytes.len() as i64), hex(bytes))
}

fn key_packet() -> Vec<Value> {
    let pair = format!("{}{}", array(&[1, 2, 3]), array(&[9; 128]));
    let reads = [
        ("pair", pair.clone()),
        ("trailing", format!("{pair}aa")),
        ("empty", String::new()),
        ("second-missing", array(&[1, 2])),
        ("second-short", format!("{}05aabb", array(&[1, 2]))),
        ("negative-length", "ffffffff0f".to_owned()),
    ];
    reads
        .iter()
        .map(|(label, hex)| {
            wire(format!("crypto/key-packet/read/{label}"), "login_key", json!({"direction": "read", "hex": hex}))
        })
        .collect()
}

pub fn cases() -> Vec<Value> {
    let mut cases = server_hash();
    cases.extend(stream());
    cases.extend(key_packet());
    cases
}
