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
}
