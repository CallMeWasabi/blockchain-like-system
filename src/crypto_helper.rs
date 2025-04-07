use secp256k1::ecdsa::Signature;
use secp256k1::hashes::{Hash, sha256};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

fn _from_hex_to_secret_key(hex_str: &str) -> Result<SecretKey, secp256k1::Error> {
    let bytes = hex::decode(hex_str).map_err(|_| secp256k1::Error::InvalidSecretKey)?;
    return SecretKey::from_slice(&bytes);
}

fn from_hex_to_public_key(hex_str: &str) -> Result<PublicKey, secp256k1::Error> {
    let bytes = hex::decode(hex_str).map_err(|_| secp256k1::Error::InvalidSecretKey)?;
    return PublicKey::from_slice(&bytes);
}

fn from_hex_to_sig(hex_str: &str) -> Result<Signature, secp256k1::Error> {
    let bytes = hex::decode(hex_str).map_err(|_| secp256k1::Error::InvalidSignature)?;
    return Signature::from_der(&bytes);
}

pub fn verify_signature(
    message: &str,
    public_key: &str,
    signature: &str,
) -> Result<bool, secp256k1::Error> {
    let digest = sha256::Hash::hash(message.as_bytes());
    let message_from_digest = Message::from_digest(digest.to_byte_array());

    let secp = Secp256k1::new();
    let public_key_bytes = match from_hex_to_public_key(&public_key) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let signature_bytes = match from_hex_to_sig(&signature) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let result = secp
        .verify_ecdsa(&message_from_digest, &signature_bytes, &public_key_bytes)
        .is_ok();

    return Ok(result);
}
