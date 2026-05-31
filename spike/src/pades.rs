use anyhow::Result;
use std::path::Path;

pub struct Signer {
    pub key: rsa::RsaPrivateKey,
    pub cert: x509_cert::Certificate,
    pub chain: Vec<x509_cert::Certificate>,
}

impl Signer {
    pub fn from_pkcs12(_path: &Path, _password: &str) -> Result<Self> {
        todo!("extraer clave, certificado y cadena del PKCS#12")
    }
}

pub fn sign_b(_pdf: &[u8], _signer: &Signer) -> Result<Vec<u8>> {
    let prepared = prepare_incremental(_pdf)?;
    let digest = digest_byte_range(&prepared)?;
    let cms = build_cms_detached(&digest, _signer)?;
    Ok(embed_contents(prepared, &cms))
}

fn prepare_incremental(_pdf: &[u8]) -> Result<Vec<u8>> {
    todo!("añadir el diccionario /Sig con SubFilter ETSI.CAdES.detached y el hueco /Contents")
}

fn digest_byte_range(_prepared: &[u8]) -> Result<Vec<u8>> {
    todo!("calcular el /ByteRange y su SHA-256")
}

fn build_cms_detached(_digest: &[u8], _signer: &Signer) -> Result<Vec<u8>> {
    todo!("SignedData detached con signedAttributes PAdES-B: content-type, message-digest, signing-certificate-v2")
}

fn embed_contents(prepared: Vec<u8>, _cms: &[u8]) -> Vec<u8> {
    todo!("rellenar /Contents con el DER del CMS")
}
