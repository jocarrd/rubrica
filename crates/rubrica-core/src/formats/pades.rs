use super::cms;
use super::validate::{trim_der, verify_detached, Report};
use crate::error::{Error, Result};
use crate::keystore::Identity;

const CONTENTS_CAPACITY: usize = 16384;

pub fn sign(pdf: &[u8], identity: &Identity) -> Result<Vec<u8>> {
    let prepared = Prepared::build(pdf)?;
    let digest = cms::sha256(&prepared.signed_bytes());
    let signature = cms::signed_data_detached(&digest, identity)?;
    prepared.embed(&signature)
}

pub fn verify(pdf: &[u8]) -> Result<Report> {
    let (content, signature) = extract(pdf)?;
    verify_detached(&content, &signature)
}

fn extract(pdf: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let needle = b"/ByteRange [";
    let pos = find_from(pdf, needle, 0).ok_or_else(|| Error::Pdf("sin /ByteRange".into()))?;
    let start = pos + needle.len();
    let close = pdf[start..]
        .iter()
        .position(|b| *b == b']')
        .ok_or_else(|| Error::Pdf("/ByteRange sin cierre".into()))?
        + start;
    let nums = std::str::from_utf8(&pdf[start..close])
        .map_err(|_| Error::Pdf("/ByteRange no es texto".into()))?
        .split_whitespace()
        .map(|s| s.parse::<usize>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| Error::Pdf("/ByteRange no numérico".into()))?;
    let [a, b, c, d] = nums[..] else {
        return Err(Error::Pdf("/ByteRange debe tener 4 valores".into()));
    };
    if a + b > pdf.len() || c + d > pdf.len() {
        return Err(Error::Pdf("/ByteRange fuera de límites".into()));
    }

    let mut content = Vec::with_capacity(b + d);
    content.extend_from_slice(&pdf[a..a + b]);
    content.extend_from_slice(&pdf[c..c + d]);

    let hole = &pdf[a + b..c];
    let lt = hole
        .iter()
        .position(|x| *x == b'<')
        .ok_or_else(|| Error::Pdf("/Contents sin <".into()))?;
    let gt = hole
        .iter()
        .position(|x| *x == b'>')
        .ok_or_else(|| Error::Pdf("/Contents sin >".into()))?;
    let raw =
        hex::decode(&hole[lt + 1..gt]).map_err(|_| Error::Pdf("/Contents no es hex".into()))?;
    let signature = trim_der(&raw)?.to_vec();

    Ok((content, signature))
}

struct Prepared {
    bytes: Vec<u8>,
    hex_start: usize,
    hex_len: usize,
    range: [usize; 4],
}

impl Prepared {
    fn build(pdf: &[u8]) -> Result<Self> {
        let prev_xref = find_startxref(pdf)?;
        let pages_ref = obj_ref_after(pdf, b"/Pages")?;
        let page_ref = first_page_ref(pdf)?;
        let mut next = max_obj_number(pdf)? + 1;

        let sig = next;
        next += 1;
        let annot = next;
        next += 1;
        let acroform = next;
        next += 1;
        let catalog = next;

        let mut out = Vec::with_capacity(pdf.len() + CONTENTS_CAPACITY * 3);
        out.extend_from_slice(pdf);
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }

        let sig_offset = out.len();
        out.extend_from_slice(
            format!("{sig} 0 obj\n<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /ETSI.CAdES.detached ")
                .as_bytes(),
        );
        out.extend_from_slice(
            b"/ByteRange [0000000000 0000000000 0000000000 0000000000] /Contents <",
        );
        let hex_start = out.len();
        out.extend(std::iter::repeat(b'0').take(CONTENTS_CAPACITY));
        let hex_len = out.len() - hex_start;
        out.extend_from_slice(b"> >>\nendobj\n");

        let annot_offset = out.len();
        out.extend_from_slice(
            format!("{annot} 0 obj\n<< /Type /Annot /Subtype /Widget /FT /Sig /Rect [0 0 0 0] /T (Rubrica Signature) /V {sig} 0 R /P {page_ref} 0 R /F 132 >>\nendobj\n")
                .as_bytes(),
        );

        let acroform_offset = out.len();
        out.extend_from_slice(
            format!("{acroform} 0 obj\n<< /Fields [{annot} 0 R] /SigFlags 3 >>\nendobj\n")
                .as_bytes(),
        );

        let catalog_offset = out.len();
        out.extend_from_slice(
            format!("{catalog} 0 obj\n<< /Type /Catalog /Pages {pages_ref} 0 R /AcroForm {acroform} 0 R >>\nendobj\n")
                .as_bytes(),
        );

        let xref_offset = out.len();
        out.extend_from_slice(b"xref\n");
        for (num, off) in [
            (sig, sig_offset),
            (annot, annot_offset),
            (acroform, acroform_offset),
            (catalog, catalog_offset),
        ] {
            out.extend_from_slice(format!("{num} 1\n{off:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {size} /Root {catalog} 0 R /Prev {prev_xref} >>\nstartxref\n{xref_offset}\n%%EOF\n",
                size = catalog + 1
            )
            .as_bytes(),
        );

        let after = hex_start + hex_len + 1;
        let total = out.len();
        let range = [0, hex_start - 1, after, total - after];

        let mut prepared = Self {
            bytes: out,
            hex_start,
            hex_len,
            range,
        };
        prepared.write_byte_range(sig_offset)?;
        Ok(prepared)
    }

    fn write_byte_range(&mut self, from: usize) -> Result<()> {
        let needle = b"/ByteRange [";
        let pos = find_from(&self.bytes, needle, from)
            .ok_or_else(|| Error::Pdf("no se encontró /ByteRange".into()))?;
        let start = pos + needle.len();
        let value = format!(
            "{:010} {:010} {:010} {:010}",
            self.range[0], self.range[1], self.range[2], self.range[3]
        );
        self.bytes[start..start + value.len()].copy_from_slice(value.as_bytes());
        Ok(())
    }

    fn signed_bytes(&self) -> Vec<u8> {
        let [a, b, c, d] = self.range;
        let mut v = Vec::with_capacity(b + d);
        v.extend_from_slice(&self.bytes[a..a + b]);
        v.extend_from_slice(&self.bytes[c..c + d]);
        v
    }

    fn embed(mut self, signature: &[u8]) -> Result<Vec<u8>> {
        let hex = hex::encode(signature);
        if hex.len() > self.hex_len {
            return Err(Error::SignatureTooLarge);
        }
        let bytes = hex.as_bytes();
        self.bytes[self.hex_start..self.hex_start + bytes.len()].copy_from_slice(bytes);
        Ok(self.bytes)
    }
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

fn parse_uint_after(tail: &[u8]) -> Option<u32> {
    let s: String = tail
        .iter()
        .skip_while(|b| b.is_ascii_whitespace())
        .take_while(|b| b.is_ascii_digit())
        .map(|b| *b as char)
        .collect();
    s.parse().ok()
}

fn find_startxref(pdf: &[u8]) -> Result<u32> {
    let pos = rfind(pdf, b"startxref").ok_or_else(|| Error::Pdf("sin startxref".into()))?;
    parse_uint_after(&pdf[pos + b"startxref".len()..])
        .ok_or_else(|| Error::Pdf("startxref no numérico".into()))
}

fn obj_ref_after(hay: &[u8], key: &[u8]) -> Result<u32> {
    let pos = find_from(hay, key, 0).ok_or_else(|| {
        Error::Pdf(format!(
            "clave {:?} no encontrada",
            String::from_utf8_lossy(key)
        ))
    })?;
    parse_uint_after(&hay[pos + key.len()..])
        .ok_or_else(|| Error::Pdf("referencia no numérica".into()))
}

fn first_page_ref(pdf: &[u8]) -> Result<u32> {
    let kids = find_from(pdf, b"/Kids", 0).ok_or_else(|| Error::Pdf("Pages sin /Kids".into()))?;
    let open = pdf[kids..]
        .iter()
        .position(|b| *b == b'[')
        .ok_or_else(|| Error::Pdf("/Kids sin [".into()))?;
    parse_uint_after(&pdf[kids + open + 1..]).ok_or_else(|| Error::Pdf("/Kids vacío".into()))
}

fn max_obj_number(pdf: &[u8]) -> Result<u32> {
    let needle = b" 0 obj";
    let mut max = 0;
    let mut i = 0;
    while let Some(p) = find_from(pdf, needle, i) {
        let mut j = p;
        while j > 0 && pdf[j - 1].is_ascii_digit() {
            j -= 1;
        }
        if let Ok(n) = std::str::from_utf8(&pdf[j..p]).unwrap_or("").parse::<u32>() {
            max = max.max(n);
        }
        i = p + needle.len();
    }
    if max == 0 {
        return Err(Error::Pdf("sin objetos".into()));
    }
    Ok(max)
}
