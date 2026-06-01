mod pkcs12;

pub use pkcs12::{load_pkcs12, parse_pkcs12};

use rsa::RsaPrivateKey;
use x509_cert::Certificate;

pub struct Identity {
    pub(crate) key: RsaPrivateKey,
    pub cert: Certificate,
    pub chain: Vec<Certificate>,
}

impl Identity {
    pub fn common_name(&self) -> Option<String> {
        self.cert
            .tbs_certificate
            .subject
            .0
            .iter()
            .flat_map(|rdn| rdn.0.iter())
            .find(|atv| atv.oid == const_oid::db::rfc4519::CN)
            .and_then(|atv| atv.value.decode_as::<der::asn1::Utf8StringRef>().ok())
            .map(|s| s.as_str().to_owned())
    }

    /// El certificado del firmante en DER (para enviarlo a un servicio de firma).
    pub fn cert_der(&self) -> crate::Result<Vec<u8>> {
        use der::Encode;
        self.cert
            .to_der()
            .map_err(|e| crate::Error::Crypto(e.to_string()))
    }

    /// El último certificado de la cadena (raíz) en DER, si lo hay.
    pub fn root_der(&self) -> Option<Vec<u8>> {
        use der::Encode;
        self.chain.last().and_then(|c| c.to_der().ok())
    }

    /// Firma `datos` con RSA PKCS#1 v1.5 sobre SHA-256, devolviendo la firma
    /// cruda. Es la operación de la fase 2 de la firma trifásica de las sedes.
    pub fn firmar_sha256(&self, datos: &[u8]) -> crate::Result<Vec<u8>> {
        use rsa::pkcs1v15::SigningKey;
        use rsa::sha2::Sha256;
        use rsa::signature::{SignatureEncoding, Signer};
        let signing_key = SigningKey::<Sha256>::new(self.key.clone());
        Ok(signing_key.sign(datos).to_vec())
    }
}
