use anyhow::{anyhow, bail, Context, Result};
use cms::builder::{SignedDataBuilder, SignerInfoBuilder};
use cms::cert::CertificateChoices;
use cms::signed_data::{EncapsulatedContentInfo, SignerIdentifier};
use const_oid::db::rfc5912::ID_SHA_256;
use der::asn1::{OctetStringRef, SetOfVec};
use der::{Any, Decode, Encode};
use rsa::pkcs1v15::SigningKey;
use rsa::sha2::Sha256;
use sha2::Digest;
use std::path::Path;
use x509_cert::attr::{Attribute, AttributeValue};
use x509_cert::Certificate;

const CONTENTS_CAPACITY: usize = 16384;
const PLACEHOLDER_OFFSET: i64 = 0;
const PLACEHOLDER_LEN: i64 = 9_999_999;

pub struct Signer {
    pub key: rsa::RsaPrivateKey,
    pub cert: Certificate,
}

impl Signer {
    pub fn from_pkcs12(path: &Path, password: &str) -> Result<Self> {
        let raw = std::fs::read(path).with_context(|| format!("leyendo {}", path.display()))?;
        let pfx = p12::PFX::parse(&raw).map_err(|e| anyhow!("PKCS#12 inválido: {e:?}"))?;

        let key_der = pfx
            .key_bags(password)
            .map_err(|e| anyhow!("descifrando claves: {e:?}"))?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("el PKCS#12 no contiene clave privada"))?;
        let key = parse_private_key(&key_der)?;

        let cert_der = pfx
            .cert_bags(password)
            .map_err(|e| anyhow!("extrayendo certificados: {e:?}"))?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("el PKCS#12 no contiene certificado"))?;
        let cert =
            Certificate::from_der(&cert_der).map_err(|e| anyhow!("certificado inválido: {e}"))?;

        Ok(Self { key, cert })
    }
}

fn parse_private_key(der: &[u8]) -> Result<rsa::RsaPrivateKey> {
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs8::DecodePrivateKey;
    if let Ok(k) = rsa::RsaPrivateKey::from_pkcs8_der(der) {
        return Ok(k);
    }
    rsa::RsaPrivateKey::from_pkcs1_der(der)
        .map_err(|e| anyhow!("clave RSA no reconocida (PKCS#8 ni PKCS#1): {e}"))
}

pub fn sign_b(pdf: &[u8], signer: &Signer) -> Result<Vec<u8>> {
    let (prepared, byte_range, contents_span) = prepare_incremental(pdf)?;
    let digest = digest_byte_range(&prepared, &byte_range);
    let cms = build_cms_detached(&digest, signer)?;
    embed_contents(prepared, contents_span, &cms)
}

struct ByteRange {
    a: usize,
    b: usize,
    c: usize,
    d: usize,
}

struct ContentsSpan {
    hex_start: usize,
    hex_len: usize,
}

fn prepare_incremental(pdf: &[u8]) -> Result<(Vec<u8>, ByteRange, ContentsSpan)> {
    let mut prev_startxref = find_startxref(pdf)?;
    let root_ref = find_root_ref(pdf)?;
    let mut next_obj = max_obj_number(pdf)? + 1;

    let sig_obj = next_obj;
    next_obj += 1;
    let annot_obj = next_obj;
    next_obj += 1;
    let acroform_obj = next_obj;
    next_obj += 1;
    let catalog_obj = next_obj;

    let page_ref = find_first_page_ref(pdf)?;

    let mut out = Vec::with_capacity(pdf.len() + CONTENTS_CAPACITY * 3);
    out.extend_from_slice(pdf);
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }

    let sig_obj_offset = out.len();
    let byte_range_token = b"/ByteRange [0000000000 0000000000 0000000000 0000000000]";
    let contents_placeholder_hex = vec![b'0'; CONTENTS_CAPACITY];

    out.extend_from_slice(format!("{sig_obj} 0 obj\n<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /ETSI.CAdES.detached ").as_bytes());
    out.extend_from_slice(byte_range_token);
    out.extend_from_slice(b" /Contents <");
    let hex_start = out.len();
    out.extend_from_slice(&contents_placeholder_hex);
    let hex_len = out.len() - hex_start;
    out.extend_from_slice(b"> >>\nendobj\n");

    let annot_offset = out.len();
    out.extend_from_slice(format!("{annot_obj} 0 obj\n<< /Type /Annot /Subtype /Widget /FT /Sig /Rect [0 0 0 0] /T (Rubrica Signature) /V {sig_obj} 0 R /P {page_ref} 0 R /F 132 >>\nendobj\n").as_bytes());

    let acroform_offset = out.len();
    out.extend_from_slice(
        format!("{acroform_obj} 0 obj\n<< /Fields [{annot_obj} 0 R] /SigFlags 3 >>\nendobj\n")
            .as_bytes(),
    );

    let catalog_offset = out.len();
    out.extend_from_slice(format!("{catalog_obj} 0 obj\n<< /Type /Catalog /Pages {root_pages} 0 R /AcroForm {acroform_obj} 0 R >>\nendobj\n", root_pages = find_pages_ref(pdf)?).as_bytes());

    let new_xref_offset = out.len();
    let entries = [
        (sig_obj, sig_obj_offset),
        (annot_obj, annot_offset),
        (acroform_obj, acroform_offset),
        (catalog_obj, catalog_offset),
    ];
    out.extend_from_slice(b"xref\n");
    for (num, off) in entries.iter() {
        out.extend_from_slice(format!("{num} 1\n{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {size} /Root {catalog_obj} 0 R /Prev {prev_startxref} >>\nstartxref\n{new_xref_offset}\n%%EOF\n",
            size = catalog_obj + 1
        )
        .as_bytes(),
    );

    let _ = (&mut prev_startxref, root_ref);

    let after_contents = hex_start + hex_len + 1;
    let total = out.len();
    let byte_range = ByteRange {
        a: 0,
        b: hex_start - 1,
        c: after_contents,
        d: total - after_contents,
    };

    write_byte_range(&mut out, sig_obj_offset, &byte_range)?;

    Ok((out, byte_range, ContentsSpan { hex_start, hex_len }))
}

fn write_byte_range(out: &mut [u8], search_from: usize, br: &ByteRange) -> Result<()> {
    let needle = b"/ByteRange [";
    let pos = find_from(out, needle, search_from)
        .ok_or_else(|| anyhow!("no se encontró /ByteRange para reescribir"))?;
    let start = pos + needle.len();
    let value = format!("{:010} {:010} {:010} {:010}", br.a, br.b, br.c, br.d);
    let slot = &mut out[start..start + value.len()];
    slot.copy_from_slice(value.as_bytes());
    Ok(())
}

fn digest_byte_range(prepared: &[u8], br: &ByteRange) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&prepared[br.a..br.a + br.b]);
    hasher.update(&prepared[br.c..br.c + br.d]);
    hasher.finalize().to_vec()
}

fn build_cms_detached(message_digest: &[u8], signer: &Signer) -> Result<Vec<u8>> {
    let digest_algorithm = spki::AlgorithmIdentifierOwned {
        oid: ID_SHA_256,
        parameters: None,
    };

    let econtent = EncapsulatedContentInfo {
        econtent_type: const_oid::db::rfc5911::ID_DATA,
        econtent: None,
    };

    let signing_key = SigningKey::<Sha256>::new(signer.key.clone());

    let sid = SignerIdentifier::IssuerAndSerialNumber(cms::cert::IssuerAndSerialNumber {
        issuer: signer.cert.tbs_certificate.issuer.clone(),
        serial_number: signer.cert.tbs_certificate.serial_number.clone(),
    });

    let mut signer_info =
        SignerInfoBuilder::new(&signing_key, sid, digest_algorithm.clone(), &econtent, None)
            .map_err(|e| anyhow!("creando SignerInfoBuilder: {e:?}"))?;

    let md_attr = Attribute {
        oid: const_oid::db::rfc5911::ID_MESSAGE_DIGEST,
        values: set_of_one(AttributeValue::new(
            der::Tag::OctetString,
            OctetStringRef::new(message_digest)?.as_bytes(),
        )?)?,
    };
    signer_info
        .add_signed_attribute(md_attr)
        .map_err(|e| anyhow!("añadiendo message-digest: {e:?}"))?;

    signer_info
        .add_signed_attribute(signing_certificate_v2(&signer.cert)?)
        .map_err(|e| anyhow!("añadiendo signing-certificate-v2: {e:?}"))?;

    let content_info = SignedDataBuilder::new(&econtent)
        .add_digest_algorithm(digest_algorithm)
        .map_err(|e| anyhow!("digest alg: {e:?}"))?
        .add_certificate(CertificateChoices::Certificate(signer.cert.clone()))
        .map_err(|e| anyhow!("añadiendo certificado: {e:?}"))?
        .add_signer_info(signer_info)
        .map_err(|e| anyhow!("añadiendo signer info: {e:?}"))?
        .build()
        .map_err(|e| anyhow!("construyendo SignedData: {e:?}"))?;

    content_info
        .to_der()
        .map_err(|e| anyhow!("DER ContentInfo: {e:?}"))
}

fn signing_certificate_v2(cert: &Certificate) -> Result<Attribute> {
    use der::asn1::OctetString;

    let cert_der = cert.to_der()?;
    let hash = Sha256::digest(&cert_der);

    #[derive(der::Sequence)]
    struct EssCertIdV2 {
        cert_hash: OctetString,
    }
    #[derive(der::Sequence)]
    struct SigningCertificateV2 {
        certs: Vec<EssCertIdV2>,
    }

    let scv2 = SigningCertificateV2 {
        certs: vec![EssCertIdV2 {
            cert_hash: OctetString::new(hash.to_vec())?,
        }],
    };
    let value = Any::encode_from(&scv2)?;

    Ok(Attribute {
        oid: const_oid::ObjectIdentifier::new("1.2.840.113549.1.9.16.2.47")?,
        values: set_of_one(value)?,
    })
}

fn set_of_one<T: der::DerOrd>(v: T) -> Result<SetOfVec<T>> {
    let mut set = SetOfVec::new();
    set.insert(v).map_err(|e| anyhow!("SetOfVec: {e:?}"))?;
    Ok(set)
}

fn embed_contents(mut prepared: Vec<u8>, span: ContentsSpan, cms: &[u8]) -> Result<Vec<u8>> {
    let hex = hex::encode(cms);
    if hex.len() > span.hex_len {
        bail!(
            "el CMS no cabe en /Contents: {} hex > {} reservados (subir CONTENTS_CAPACITY)",
            hex.len(),
            span.hex_len
        );
    }
    let bytes = hex.as_bytes();
    prepared[span.hex_start..span.hex_start + bytes.len()].copy_from_slice(bytes);
    for b in prepared[span.hex_start + bytes.len()..span.hex_start + span.hex_len].iter_mut() {
        *b = b'0';
    }
    Ok(prepared)
}

fn find_from(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

fn rfind(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).rposition(|w| w == needle)
}

fn find_startxref(pdf: &[u8]) -> Result<usize> {
    let pos = rfind(pdf, b"startxref").ok_or_else(|| anyhow!("PDF sin startxref"))?;
    let tail = &pdf[pos + b"startxref".len()..];
    let num: String = tail
        .iter()
        .skip_while(|b| b.is_ascii_whitespace())
        .take_while(|b| b.is_ascii_digit())
        .map(|b| *b as char)
        .collect();
    num.trim().parse().context("startxref no numérico")
}

fn find_root_ref(pdf: &[u8]) -> Result<u32> {
    obj_ref_after(pdf, b"/Root").context("trailer sin /Root")
}

fn find_pages_ref(pdf: &[u8]) -> Result<u32> {
    obj_ref_after(pdf, b"/Pages").context("catálogo sin /Pages")
}

fn find_first_page_ref(pdf: &[u8]) -> Result<u32> {
    let pos = find_from(pdf, b"/Kids", 0).ok_or_else(|| anyhow!("Pages sin /Kids"))?;
    let tail = &pdf[pos..];
    let open = tail
        .iter()
        .position(|b| *b == b'[')
        .ok_or_else(|| anyhow!("/Kids sin ["))?;
    obj_ref_after(&tail[open..], b"[").context("/Kids vacío")
}

fn obj_ref_after(hay: &[u8], key: &[u8]) -> Result<u32> {
    let pos = find_from(hay, key, 0).ok_or_else(|| anyhow!("clave no encontrada"))?;
    let tail = &hay[pos + key.len()..];
    let num: String = tail
        .iter()
        .skip_while(|b| b.is_ascii_whitespace())
        .take_while(|b| b.is_ascii_digit())
        .map(|b| *b as char)
        .collect();
    num.trim()
        .parse()
        .context("referencia de objeto no numérica")
}

fn max_obj_number(pdf: &[u8]) -> Result<u32> {
    let mut max = 0u32;
    let needle = b" 0 obj";
    let mut i = 0;
    while let Some(p) = find_from(pdf, needle, i) {
        let mut j = p;
        while j > 0 && pdf[j - 1].is_ascii_digit() {
            j -= 1;
        }
        if let Ok(n) = std::str::from_utf8(&pdf[j..p])
            .unwrap_or("")
            .trim()
            .parse::<u32>()
        {
            max = max.max(n);
        }
        i = p + needle.len();
    }
    if max == 0 {
        bail!("no se encontró ningún objeto en el PDF");
    }
    Ok(max)
}

const _: () = {
    let _ = PLACEHOLDER_OFFSET;
    let _ = PLACEHOLDER_LEN;
};
