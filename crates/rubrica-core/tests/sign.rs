use rubrica_core::{formats, parse_pkcs12};

const P12: &[u8] = include_bytes!("fixtures/test.p12");
const PDF: &[u8] = include_bytes!("fixtures/sample.pdf");

fn identity() -> rubrica_core::Identity {
    parse_pkcs12(P12, "test").expect("cargar PKCS#12 de prueba")
}

#[test]
fn carga_pkcs12_y_lee_common_name() {
    let id = identity();
    assert_eq!(id.common_name().as_deref(), Some("Rubrica Test"));
}

#[test]
fn pades_produce_estructura_de_firma() {
    let signed = formats::pades::sign(PDF, &identity()).expect("firmar PDF");

    let contains = |needle: &[u8]| signed.windows(needle.len()).any(|w| w == needle);
    assert!(contains(b"/Type /Sig"));
    assert!(contains(b"ETSI.CAdES.detached"));
    assert!(contains(b"/AcroForm"));
    assert!(contains(b"/ByteRange ["));
    assert!(signed.len() > PDF.len());
}

#[test]
fn cades_produce_cms_signed_data() {
    let sig = formats::cades::sign(b"contenido de prueba", &identity()).expect("firmar CAdES");
    assert!(sig.len() > 100);
    assert_eq!(sig[0], 0x30, "el CMS debe empezar por SEQUENCE (DER)");
}
