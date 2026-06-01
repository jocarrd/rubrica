use crate::error::{Error, Result};
use cms::content_info::ContentInfo;
use cms::signed_data::{SignedData, SignerInfo};
use const_oid::db::rfc5911::{ID_MESSAGE_DIGEST, ID_SIGNED_DATA};
use der::asn1::{OctetString, SetOfVec};
use der::{Decode, Encode};
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use sha2::{Digest, Sha256};
use x509_cert::attr::Attribute;
use x509_cert::Certificate;

pub struct Report {
    pub signer_common_name: Option<String>,
    pub digest_matches: bool,
    pub signature_valid: bool,
}

impl Report {
    pub fn is_valid(&self) -> bool {
        self.digest_matches && self.signature_valid
    }
}

pub fn verify_detached(content: &[u8], cms_der: &[u8]) -> Result<Report> {
    let info =
        ContentInfo::from_der(cms_der).map_err(|e| Error::Crypto(format!("ContentInfo: {e}")))?;
    if info.content_type != ID_SIGNED_DATA {
        return Err(Error::Crypto("el CMS no es SignedData".into()));
    }
    let signed_data = info
        .content
        .decode_as::<SignedData>()
        .map_err(|e| Error::Crypto(format!("SignedData: {e}")))?;

    let signer = signed_data
        .signer_infos
        .0
        .iter()
        .next()
        .ok_or_else(|| Error::Crypto("el CMS no contiene firmantes".into()))?;
    let cert = signer_certificate(&signed_data)?;

    let signed_attrs = signer
        .signed_attrs
        .as_ref()
        .ok_or_else(|| Error::Crypto("la firma no tiene atributos firmados".into()))?;

    let declared = message_digest(signed_attrs)?;
    let digest_matches = declared == Sha256::digest(content).as_slice();
    let signature_valid = verify_signature(signer, &cert, signed_attrs)?;

    Ok(Report {
        signer_common_name: subject_common_name(&cert),
        digest_matches,
        signature_valid,
    })
}

fn verify_signature(
    signer: &SignerInfo,
    cert: &Certificate,
    signed_attrs: &SetOfVec<Attribute>,
) -> Result<bool> {
    let attrs_der = signed_attrs
        .to_der()
        .map_err(|e| Error::Crypto(format!("codificando atributos: {e}")))?;

    let spki = &cert.tbs_certificate.subject_public_key_info;
    let public_key = RsaPublicKey::from_pkcs1_der(
        spki.subject_public_key
            .as_bytes()
            .ok_or_else(|| Error::Crypto("clave pública sin bytes".into()))?,
    )
    .map_err(|e| Error::Crypto(format!("clave pública RSA: {e}")))?;

    let verifying_key = VerifyingKey::<Sha256>::new(public_key);
    let signature = Signature::try_from(signer.signature.as_bytes())
        .map_err(|e| Error::Crypto(format!("firma: {e}")))?;

    Ok(verifying_key.verify(&attrs_der, &signature).is_ok())
}

fn message_digest(attrs: &SetOfVec<Attribute>) -> Result<Vec<u8>> {
    for attr in attrs.iter() {
        if attr.oid == ID_MESSAGE_DIGEST {
            let value = attr
                .values
                .iter()
                .next()
                .ok_or_else(|| Error::Crypto("message-digest vacío".into()))?;
            let octets = value
                .decode_as::<OctetString>()
                .map_err(|e| Error::Crypto(format!("message-digest: {e}")))?;
            return Ok(octets.as_bytes().to_vec());
        }
    }
    Err(Error::Crypto("falta el atributo message-digest".into()))
}

fn signer_certificate(signed_data: &SignedData) -> Result<Certificate> {
    use cms::cert::CertificateChoices;
    signed_data
        .certificates
        .as_ref()
        .and_then(|set| {
            set.0.iter().find_map(|choice| match choice {
                CertificateChoices::Certificate(cert) => Some(cert.clone()),
                _ => None,
            })
        })
        .ok_or_else(|| Error::Crypto("el CMS no incluye el certificado del firmante".into()))
}

fn subject_common_name(cert: &Certificate) -> Option<String> {
    cert.tbs_certificate
        .subject
        .0
        .iter()
        .flat_map(|rdn| rdn.0.iter())
        .find(|atv| atv.oid == const_oid::db::rfc4519::CN)
        .and_then(|atv| atv.value.decode_as::<der::asn1::Utf8StringRef>().ok())
        .map(|s| s.as_str().to_owned())
}

pub(crate) fn trim_der(bytes: &[u8]) -> Result<&[u8]> {
    if bytes.len() < 2 || bytes[0] != 0x30 {
        return Err(Error::Crypto("no es una estructura DER SEQUENCE".into()));
    }
    let first = bytes[1];
    let total = if first < 0x80 {
        2 + first as usize
    } else {
        let n = (first & 0x7f) as usize;
        if bytes.len() < 2 + n {
            return Err(Error::Crypto("longitud DER truncada".into()));
        }
        let mut len = 0usize;
        for &b in &bytes[2..2 + n] {
            len = (len << 8) | b as usize;
        }
        2 + n + len
    };
    bytes
        .get(..total)
        .ok_or_else(|| Error::Crypto("longitud DER mayor que los datos".into()))
}
