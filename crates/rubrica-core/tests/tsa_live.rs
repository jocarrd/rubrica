#[test]
#[ignore]
fn freetsa_devuelve_token_valido() {
    let token = rubrica_core::tsa::timestamp_token(b"hola rubrica", None)
        .expect("obtener sello de freetsa");
    assert!(token.len() > 100, "el token debe tener tamaño razonable");
    assert_eq!(token[0], 0x30, "el token es un ContentInfo DER");
    std::fs::write("/tmp/rubrica-tst.der", &token).unwrap();
}
