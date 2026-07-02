use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use std::fs::{self, OpenOptions};
use std::io::{Error, ErrorKind, Write};
use std::path::Path;

const KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 24;

pub struct Cipher {
    inner: XChaCha20Poly1305,
}

impl Cipher {
    pub fn load(root: &Path) -> std::io::Result<Self> {
        let key = load_or_create_key(&root.join("store.key"))?;
        Ok(Self {
            inner: XChaCha20Poly1305::new((&key).into()),
        })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut nonce = [0_u8; NONCE_BYTES];
        getrandom::fill(&mut nonce).map_err(random_error)?;
        let ciphertext = self
            .inner
            .encrypt(XNonce::from_slice(&nonce), plaintext)
            .map_err(cipher_error)?;
        Ok([nonce.to_vec(), ciphertext].concat())
    }

    pub fn decrypt(&self, blob: &[u8]) -> std::io::Result<Vec<u8>> {
        if blob.len() < NONCE_BYTES {
            return Err(Error::new(ErrorKind::InvalidData, "invalid evidence blob"));
        }
        let (nonce, ciphertext) = blob.split_at(NONCE_BYTES);
        self.inner
            .decrypt(XNonce::from_slice(nonce), ciphertext)
            .map_err(cipher_error)
    }
}

fn load_or_create_key(path: &Path) -> std::io::Result<[u8; KEY_BYTES]> {
    if path.exists() {
        return read_key(path);
    }
    let mut key = [0_u8; KEY_BYTES];
    getrandom::fill(&mut key).map_err(random_error)?;
    write_private(path, &key)?;
    Ok(key)
}

fn read_key(path: &Path) -> std::io::Result<[u8; KEY_BYTES]> {
    let bytes = fs::read(path)?;
    bytes.try_into().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            "OpenTerm evidence key has an invalid length",
        )
    })
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)
}

fn cipher_error(_: chacha20poly1305::Error) -> Error {
    Error::new(ErrorKind::InvalidData, "evidence encryption failed")
}

fn random_error(error: getrandom::Error) -> Error {
    Error::other(format!("secure random generation failed: {error}"))
}
