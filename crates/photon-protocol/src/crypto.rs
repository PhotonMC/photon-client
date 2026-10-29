//! The login encryption: the session server hash, the RSA key exchange and the AES/CFB8
//! stream cipher that wraps the connection once the server asked for it.

use aes::Aes128;
use cfb8::cipher::generic_array::GenericArray;
use cfb8::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Pkcs1v15Encrypt, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha1::{Digest, Sha1};
use sha2::Sha256;

/// The bytes of the shared secret.
pub const SECRET_LENGTH: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptError {
    /// The server's public key is not an X.509 RSA key.
    BadPublicKey,
    /// The private key is not a PKCS#8 RSA key.
    BadPrivateKey,
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

/// `Signature.getInstance("SHA1withRSA")` verifying `signature` over `message` under the key in
/// `public_key_der`. Any malformed key or signature is a failed check.
pub fn verify_sha1_rsa(public_key_der: &[u8], message: &[u8], signature: &[u8]) -> bool {
    /// The `DigestInfo` header of a SHA-1 digest (RFC 8017, note 1).
    const SHA1_PREFIX: [u8; 15] = [
        0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04, 0x14,
    ];
    let Ok(key) = RsaPublicKey::from_public_key_der(public_key_der) else {
        return false;
    };
    let scheme = rsa::Pkcs1v15Sign {
        hash_len: Some(20),
        prefix: Box::new(SHA1_PREFIX),
    };
    key.verify(scheme, &Sha1::digest(message), signature)
        .is_ok()
}

/// `Signature.getInstance("SHA256withRSA")` verifying `signature` over `message` under the key
/// in `public_key_der`. Any malformed key or signature is a failed check.
pub fn verify_sha256_rsa(public_key_der: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let Ok(key) = RsaPublicKey::from_public_key_der(public_key_der) else {
        return false;
    };
    key.verify(
        Pkcs1v15Sign::new::<Sha256>(),
        &Sha256::digest(message),
        signature,
    )
    .is_ok()
}

/// Whether `private_key_der` (PKCS#8) and `public_key_der` (X.509) are the two halves of one RSA
/// key, which is what a profile key pair has to be before anything is signed with it.
pub fn key_pair_matches(private_key_der: &[u8], public_key_der: &[u8]) -> bool {
    let (Ok(private), Ok(public)) = (
        RsaPrivateKey::from_pkcs8_der(private_key_der),
        RsaPublicKey::from_public_key_der(public_key_der),
    ) else {
        return false;
    };
    private.to_public_key() == public
}

/// `Signer.from(privateKey, "SHA256withRSA")`: the signature of the parts joined, PKCS#1 v1.5.
/// The private key stays with the caller; nothing here keeps it.
pub fn sign_sha256_rsa(private_key_der: &[u8], parts: &[&[u8]]) -> Result<Vec<u8>, CryptError> {
    let key =
        RsaPrivateKey::from_pkcs8_der(private_key_der).map_err(|_| CryptError::BadPrivateKey)?;
    let mut digest = Sha256::new();
    for part in parts {
        digest.update(part);
    }
    key.sign(Pkcs1v15Sign::new::<Sha256>(), &digest.finalize())
        .map_err(|_| CryptError::Rsa)
}

/// `Crypt.encryptUsingKey`: RSA with PKCS#1 v1.5 padding under the server's public key.
pub fn encrypt_with_public_key(public_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptError> {
    let key =
        RsaPublicKey::from_public_key_der(public_key_der).map_err(|_| CryptError::BadPublicKey)?;
    let mut random = SystemRandom;
    key.encrypt(&mut random, Pkcs1v15Encrypt, data)
        .map_err(|_| CryptError::Rsa)
}

/// The random source the RSA padding draws from (and the test keys, in the crates above).
pub struct SystemRandom;

impl rsa::rand_core::CryptoRng for SystemRandom {}

impl rsa::rand_core::RngCore for SystemRandom {
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
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};

    #[test]
    fn a_key_pair_matches_only_its_own_halves() {
        let one = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
        let other = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
        let private = one.to_pkcs8_der().expect("der");
        let own = one.to_public_key().to_public_key_der().expect("der");
        let foreign = other.to_public_key().to_public_key_der().expect("der");
        assert!(key_pair_matches(private.as_bytes(), own.as_bytes()));
        assert!(!key_pair_matches(private.as_bytes(), foreign.as_bytes()));
        assert!(!key_pair_matches(b"not a key", own.as_bytes()));
    }

    #[test]
    fn rsa_round_trip_through_der_key() {
        let private = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
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
    fn sha1_signatures_verify_only_for_their_message() {
        let private = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
        let der = private
            .to_public_key()
            .to_public_key_der()
            .expect("der")
            .to_vec();
        let message = b"textures property";
        // The signer pads a `DigestInfo` it builds here from the SHA-1 object id 1.3.14.3.2.26.
        let mut info = vec![
            0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a,
        ];
        info.extend([0x05, 0x00, 0x04, 0x14]);
        info.extend(Sha1::digest(message));
        let signature = private
            .sign(rsa::Pkcs1v15Sign::new_unprefixed(), &info)
            .expect("sign");
        assert!(verify_sha1_rsa(&der, message, &signature));
        assert!(!verify_sha1_rsa(&der, b"another", &signature));
        assert!(!verify_sha1_rsa(&der, message, &signature[1..]));
        assert!(!verify_sha1_rsa(&[1, 2], message, &signature));
    }

    #[test]
    fn a_chat_signature_verifies_over_the_parts_joined() {
        let private = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
        let der = private.to_pkcs8_der().expect("der");
        let signature = sign_sha256_rsa(der.as_bytes(), &[b"one ", b"two"]).expect("signed");
        let digest = Sha256::digest(b"one two");
        let public = private.to_public_key();
        assert!(
            public
                .verify(Pkcs1v15Sign::new::<Sha256>(), &digest, &signature)
                .is_ok()
        );
        assert_eq!(signature.len(), 128);
        let public_der = private.to_public_key().to_public_key_der().expect("der");
        assert!(verify_sha256_rsa(
            public_der.as_bytes(),
            b"one two",
            &signature
        ));
        assert!(!verify_sha256_rsa(
            public_der.as_bytes(),
            b"one",
            &signature
        ));
        assert!(!verify_sha256_rsa(b"nope", b"one two", &signature));
        assert_eq!(
            sign_sha256_rsa(b"nope", &[b"x"]),
            Err(CryptError::BadPrivateKey)
        );
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
