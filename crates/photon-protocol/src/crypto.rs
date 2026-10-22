//! The login encryption: the session server hash, the RSA key exchange and the AES/CFB8
//! stream cipher that wraps the connection once the server asked for it.

use aes::Aes128;
use cfb8::cipher::generic_array::GenericArray;
use cfb8::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rsa::pkcs8::DecodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use sha1::{Digest, Sha1};

/// The bytes of the shared secret.
pub const SECRET_LENGTH: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptError {
    /// The server's public key is not an X.509 RSA key.
    BadPublicKey,
    /// The RSA step failed (the data does not fit the key).
    Rsa,
    /// The system has no random bytes to give.
    NoRandom,
}

/// A fresh random shared secret (`Crypt.generateSecretKey`).
pub fn generate_secret() -> Result<[u8; SECRET_LENGTH], CryptError> {
    let mut secret = [0u8; SECRET_LENGTH];
    getrandom::getrandom(&mut secret).map_err(|_| CryptError::NoRandom)?;
    Ok(secret)
}

/// `Crypt.digestData` printed as `new BigInteger(digest).toString(16)`: the SHA-1 of the server
/// id (Latin-1), the secret and the key's DER bytes, read as a signed number, so a digest whose
/// first bit is set gets a minus sign and the two's complement magnitude.
pub fn server_hash(server_id: &[u16], secret: &[u8], public_key_der: &[u8]) -> String {
    let mut sha = Sha1::new();
    // Java's Latin-1 encoder gives one `?` to a surrogate pair, and one to a lone surrogate.
    let latin1: Vec<u8> = char::decode_utf16(server_id.iter().copied())
        .map(|unit| match unit {
            Ok(character) => u8::try_from(u32::from(character)).unwrap_or(b'?'),
            Err(_) => b'?',
        })
        .collect();
    sha.update(&latin1);
    sha.update(secret);
    sha.update(public_key_der);
    signed_hex(&sha.finalize())
}

/// `BigInteger.toString(16)` of big-endian two's complement bytes.
pub fn signed_hex(bytes: &[u8]) -> String {
    let negative = bytes.first().is_some_and(|&first| first & 0x80 != 0);
    let mut magnitude = bytes.to_vec();
    if negative {
        let mut carry = true;
        for byte in magnitude.iter_mut().rev() {
            *byte = !*byte;
            if carry {
                let (sum, overflow) = byte.overflowing_add(1);
                *byte = sum;
                carry = overflow;
            }
        }
    }
    let digits: String = magnitude.iter().map(|byte| format!("{byte:02x}")).collect();
    let trimmed = digits.trim_start_matches('0');
    match (negative, trimmed.is_empty()) {
        (_, true) => "0".to_owned(),
        (true, false) => format!("-{trimmed}"),
        (false, false) => trimmed.to_owned(),
    }
}

/// `Crypt.encryptUsingKey`: RSA with PKCS#1 v1.5 padding under the server's public key.
pub fn encrypt_with_public_key(public_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptError> {
    let key =
        RsaPublicKey::from_public_key_der(public_key_der).map_err(|_| CryptError::BadPublicKey)?;
    let mut random = RandomSource;
    key.encrypt(&mut random, Pkcs1v15Encrypt, data)
        .map_err(|_| CryptError::Rsa)
}

/// The random source the RSA padding draws from.
struct RandomSource;

impl rsa::rand_core::CryptoRng for RandomSource {}

impl rsa::rand_core::RngCore for RandomSource {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // A system without random bytes cannot do the login anyway: stop rather than pad with
        // something predictable.
        getrandom::getrandom(dest).expect("system random bytes");
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rsa::rand_core::Error> {
        getrandom::getrandom(dest).map_err(rsa::rand_core::Error::from)
    }
}

/// One direction of the connection's AES/CFB8 stream (`Crypt.getCipher`: the secret is the key
/// and, as in vanilla, also the IV).
pub struct StreamCipher {
    inner: Direction,
}

enum Direction {
    Encrypt(cfb8::Encryptor<Aes128>),
    Decrypt(cfb8::Decryptor<Aes128>),
}

impl StreamCipher {
    pub fn encryptor(secret: &[u8; SECRET_LENGTH]) -> Self {
        Self {
            inner: Direction::Encrypt(cfb8::Encryptor::new(secret.into(), secret.into())),
        }
    }

    pub fn decryptor(secret: &[u8; SECRET_LENGTH]) -> Self {
        Self {
            inner: Direction::Decrypt(cfb8::Decryptor::new(secret.into(), secret.into())),
        }
    }

    /// Run the next bytes of the stream through the cipher, in place.
    pub fn apply(&mut self, data: &mut [u8]) {
        // CFB8 has blocks of one byte; the cipher keeps the shift register between calls.
        for byte in data {
            let block = GenericArray::from_mut_slice(std::slice::from_mut(byte));
            match &mut self.inner {
                Direction::Encrypt(cipher) => cipher.encrypt_block_mut(block),
                Direction::Decrypt(cipher) => cipher.decrypt_block_mut(block),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::RsaPrivateKey;
    use rsa::pkcs8::EncodePublicKey;

    #[test]
    fn rsa_round_trip_through_der_key() {
        let private = RsaPrivateKey::new(&mut RandomSource, 1024).expect("key");
        let der = private
            .to_public_key()
            .to_public_key_der()
            .expect("der")
            .to_vec();
        let secret = generate_secret().expect("random");
        let sealed = encrypt_with_public_key(&der, &secret).expect("encrypt");
        let opened = private.decrypt(Pkcs1v15Encrypt, &sealed).expect("decrypt");
        assert_eq!(opened, secret);
    }

    #[test]
    fn bad_key_is_refused() {
        assert_eq!(
            encrypt_with_public_key(&[1, 2, 3], &[0]),
            Err(CryptError::BadPublicKey)
        );
    }

    #[test]
    fn signed_hex_of_edge_values() {
        assert_eq!(signed_hex(&[0, 0]), "0");
        assert_eq!(signed_hex(&[0x80]), "-80");
        assert_eq!(signed_hex(&[0xff]), "-1");
        assert_eq!(signed_hex(&[0x00, 0x0a]), "a");
    }
}
