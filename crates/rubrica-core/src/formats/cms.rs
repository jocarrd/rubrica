use crate::error::{Error, Result};
use crate::keystore::Identity;
use cms::builder::{SignedDataBuilder, SignerInfoBuilder};
use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::signed_data::{EncapsulatedContentInfo, SignerIdentifier};
use const_oid::db::rfc5912::ID_SHA_256;
use der::asn1::{OctetString, OctetStringRef, SetOfVec};
use der::{Any, Encode};
use rsa::pkcs1v15::SigningKey;
use rsa::sha2::Sha256;
use sha2::Digest;
use x509_cert::attr::{Attribute, AttributeValue};
use x509_cert::Certificate;

const OID_SIGNING_CERTIFICATE_V2: &str = "1.2.840.113549.1.9.16.2.47";

pub fn signed_data_detached(message_digest: &[u8], identity: &Identity) -> Result<Vec<u8>> {
    let digest_algorithm = spki::AlgorithmIdentifierOwned {
        oid: ID_SHA_256,
        parameters: None,
    };

    let econtent = EncapsulatedContentInfo {
        econtent_type: const_oid::db::rfc5911::ID_DATA,
        econtent: None,
    };

    let signing_key = SigningKey::<Sha256>::new(identity.key.clone());

    let sid = SignerIdentifier::IssuerAndSerialNumber(IssuerAndSerialNumber {
        issuer: identity.cert.tbs_certificate.issuer.clone(),
        serial_number: identity.cert.tbs_certificate.serial_number.clone(),
    });

    let mut signer_info =
        SignerInfoBuilder::new(&signing_key, sid, digest_algorithm.clone(), &econtent, None)
            .map_err(|e| Error::Crypto(format!("signer info: {e:?}")))?;

    signer_info
        .add_signed_attribute(message_digest_attribute(message_digest)?)
        .map_err(|e| Error::Crypto(format!("message-digest: {e:?}")))?;
    signer_info
        .add_signed_attribute(signing_certificate_v2(&identity.cert)?)
        .map_err(|e| Error::Crypto(format!("signing-certificate-v2: {e:?}")))?;

    let mut builder = SignedDataBuilder::new(&econtent);
    builder
        .add_digest_algorithm(digest_algorithm)
        .map_err(|e| Error::Crypto(format!("digest alg: {e:?}")))?;
    builder
        .add_certificate(CertificateChoices::Certificate(identity.cert.clone()))
        .map_err(|e| Error::Crypto(format!("certificado: {e:?}")))?;
    for ca in &identity.chain {
        builder
            .add_certificate(CertificateChoices::Certificate(ca.clone()))
            .map_err(|e| Error::Crypto(format!("cadena: {e:?}")))?;
    }
    builder
        .add_signer_info(signer_info)
        .map_err(|e| Error::Crypto(format!("añadir signer info: {e:?}")))?;

    let content_info = builder
        .build()
        .map_err(|e| Error::Crypto(format!("build SignedData: {e:?}")))?;

    content_info
        .to_der()
        .map_err(|e| Error::Crypto(format!("DER: {e:?}")))
}

fn message_digest_attribute(digest: &[u8]) -> Result<Attribute> {
    let value = AttributeValue::new(
        der::Tag::OctetString,
        OctetStringRef::new(digest)
            .map_err(|e| Error::Crypto(e.to_string()))?
            .as_bytes(),
    )
    .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(Attribute {
        oid: const_oid::db::rfc5911::ID_MESSAGE_DIGEST,
        values: set_of_one(value)?,
    })
}

fn signing_certificate_v2(cert: &Certificate) -> Result<Attribute> {
    #[derive(der::Sequence)]
    struct EssCertIdV2 {
        cert_hash: OctetString,
    }
    #[derive(der::Sequence)]
    struct SigningCertificateV2 {
        certs: Vec<EssCertIdV2>,
    }

    let cert_der = cert.to_der().map_err(|e| Error::Crypto(e.to_string()))?;
    let hash = Sha256::digest(&cert_der);

    let scv2 = SigningCertificateV2 {
        certs: vec![EssCertIdV2 {
            cert_hash: OctetString::new(hash.to_vec()).map_err(|e| Error::Crypto(e.to_string()))?,
        }],
    };
    let value = Any::encode_from(&scv2).map_err(|e| Error::Crypto(e.to_string()))?;

    Ok(Attribute {
        oid: const_oid::ObjectIdentifier::new(OID_SIGNING_CERTIFICATE_V2)
            .map_err(|e| Error::Crypto(e.to_string()))?,
        values: set_of_one(value)?,
    })
}

fn set_of_one<T: der::DerOrd>(v: T) -> Result<SetOfVec<T>> {
    let mut set = SetOfVec::new();
    set.insert(v).map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(set)
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}
