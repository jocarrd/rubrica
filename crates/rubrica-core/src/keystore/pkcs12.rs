use super::Identity;
use crate::error::{Error, Result};
use der::Decode;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;
use std::path::Path;
use x509_cert::Certificate;

pub fn load_pkcs12(path: &Path, password: &str) -> Result<Identity> {
    let raw = std::fs::read(path)?;
    parse_pkcs12(&raw, password)
}

pub fn parse_pkcs12(raw: &[u8], password: &str) -> Result<Identity> {
    modern(raw, password).or_else(|_| legacy(raw, password))
}

fn modern(raw: &[u8], password: &str) -> Result<Identity> {
    let store = p12_keystore::KeyStore::from_pkcs12(raw, password).map_err(|_| Error::Pkcs12)?;
    let (_, chain) = store
        .private_key_chain()
        .ok_or(Error::Missing("clave privada"))?;

    let key = parse_private_key(chain.key())?;
    let mut certs = chain
        .chain()
        .iter()
        .map(|c| Certificate::from_der(c.as_der()).map_err(|e| Error::Certificate(e.to_string())))
        .collect::<Result<Vec<_>>>()?;

    if certs.is_empty() {
        return Err(Error::Missing("certificado"));
    }
    let cert = certs.remove(0);
    Ok(Identity {
        key,
        cert,
        chain: certs,
    })
}

fn legacy(raw: &[u8], password: &str) -> Result<Identity> {
    let pfx = p12::PFX::parse(raw).map_err(|_| Error::Pkcs12)?;

    let key_der = pfx
        .key_bags(password)
        .map_err(|_| Error::Pkcs12)?
        .into_iter()
        .next()
        .ok_or(Error::Missing("clave privada"))?;
    let key = parse_private_key(&key_der)?;

    let mut certs = pfx
        .cert_bags(password)
        .map_err(|_| Error::Pkcs12)?
        .into_iter()
        .map(|der| Certificate::from_der(&der).map_err(|e| Error::Certificate(e.to_string())))
        .collect::<Result<Vec<_>>>()?;

    if certs.is_empty() {
        return Err(Error::Missing("certificado"));
    }
    let cert = certs.remove(0);
    Ok(Identity {
        key,
        cert,
        chain: certs,
    })
}

fn parse_private_key(der: &[u8]) -> Result<RsaPrivateKey> {
    if let Ok(k) = RsaPrivateKey::from_pkcs8_der(der) {
        return Ok(k);
    }
    RsaPrivateKey::from_pkcs1_der(der).map_err(|_| Error::UnsupportedKey)
}
