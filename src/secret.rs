use crate::settings::Settings;
use crate::storage::{write_bytes};
use crate::utils::{hex_decode, hex_encode};
use aes::Aes256;
use eax::aead::{AeadCore, AeadInPlace, KeyInit};
use eax::Eax;
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::io;

pub struct SecretKey;

impl SecretKey {
    pub fn generate(keypass: &str, settings: &Settings) -> Result<String, Box<dyn Error>> {
        let mut hasher = Sha256::new();
        hasher.update(keypass.as_bytes());
        let digest = hasher.finalize();
        let hex = hex_encode(&digest);
        write_bytes(&settings.storage_key_path, hex.as_bytes())?;
        Ok(hex)
    }

    pub fn read(settings: &Settings) -> Result<Vec<u8>, Box<dyn Error>> {
        let content = fs::read_to_string(&settings.storage_key_path).map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Secret key not found at {}. Run genkey first.",
                    settings.storage_key_path.display()
                ),
            )
        })?;
        hex_decode(content.trim())
    }
}

pub struct Crypto {
    key: Vec<u8>,
}

impl Crypto {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    pub fn encrypt(&self, raw: &str) -> Result<String, Box<dyn Error>> {
        let cipher =
            Eax::<Aes256>::new_from_slice(&self.key).map_err(|e| format!("cipher init: {:?}", e))?;
        let nonce = Eax::<Aes256>::generate_nonce(&mut OsRng);
        let mut buffer = raw.as_bytes().to_vec();
        let tag = cipher
            .encrypt_in_place_detached(&nonce, b"", &mut buffer)
            .map_err(|e| format!("encrypt error: {:?}", e))?;
        Ok(format!(
            "{}\\{}\\{}",
            hex_encode(&nonce),
            hex_encode(tag.as_slice()),
            hex_encode(&buffer)
        ))
    }

    pub fn decrypt(&self, encrypt: &str) -> Result<String, Box<dyn Error>> {
        let mut parts = encrypt.split('\\');
        let nonce = parts
            .next()
            .ok_or("Encryption is corrupted: missing nonce")?;
        let tag = parts
            .next()
            .ok_or("Encryption is corrupted: missing tag")?;
        let body = parts
            .next()
            .ok_or("Encryption is corrupted: missing cipher")?;

        let nonce = hex_decode(nonce)?;
        let tag = hex_decode(tag)?;
        let mut data = hex_decode(body)?;
        let cipher =
            Eax::<Aes256>::new_from_slice(&self.key).map_err(|e| format!("cipher init: {:?}", e))?;
        cipher
            .decrypt_in_place_detached(
                nonce.as_slice().into(),
                b"",
                &mut data,
                tag.as_slice().into(),
            )
            .map_err(|e| format!("decrypt error: {:?}", e))?;
        let plaintext = String::from_utf8(data)?;
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = vec![1u8; 32];
        let crypto = Crypto::new(key);
        let plain = "secret-text";
        let enc = crypto.encrypt(plain).unwrap();
        let dec = crypto.decrypt(&enc).unwrap();
        assert_eq!(dec, plain);
    }
}
