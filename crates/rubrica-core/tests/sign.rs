use rubrica_core::{formats, parse_pkcs12};

const P12: &[u8] = include_bytes!("fixtures/test.p12");
const P12_MODERN: &[u8] = include_bytes!("fixtures/modern.p12");
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

#[test]
fn pades_firma_y_verifica_round_trip() {
    let signed = formats::pades::sign(PDF, &identity()).expect("firmar PDF");
    let report = formats::pades::verify(&signed).expect("verificar PDF");
    assert!(report.digest_matches, "el documento no debe estar alterado");
    assert!(report.signature_valid, "la firma debe ser válida");
    assert!(report.is_valid());
    assert_eq!(report.signer_common_name.as_deref(), Some("Rubrica Test"));
}

#[test]
fn pades_detecta_documento_manipulado() {
    let mut signed = formats::pades::sign(PDF, &identity()).expect("firmar PDF");
    signed[20] ^= 0xff;
    let report = formats::pades::verify(&signed).expect("verificar PDF manipulado");
    assert!(
        !report.is_valid(),
        "una firma sobre contenido alterado no es válida"
    );
}

#[test]
fn cades_firma_y_verifica_round_trip() {
    let data = b"contenido de prueba";
    let sig = formats::cades::sign(data, &identity()).expect("firmar CAdES");
    let report = formats::cades::verify(data, &sig).expect("verificar CAdES");
    assert!(report.is_valid());

    let report_malo = formats::cades::verify(b"otro contenido", &sig).expect("verificar alterado");
    assert!(!report_malo.is_valid());
}

#[test]
fn carga_pkcs12_moderno_aes_y_firma() {
    let id = parse_pkcs12(P12_MODERN, "test").expect("cargar PKCS#12 con cifrado AES/PBES2");
    assert_eq!(id.common_name().as_deref(), Some("Modern Test"));

    let signed = formats::pades::sign(PDF, &id).expect("firmar con certificado moderno");
    let report = formats::pades::verify(&signed).expect("verificar");
    assert!(report.is_valid());
}
