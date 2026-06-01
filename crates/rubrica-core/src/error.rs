use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("entrada/salida: {0}")]
    Io(#[from] std::io::Error),

    #[error("PKCS#12 inválido o contraseña incorrecta")]
    Pkcs12,

    #[error("el almacén no contiene {0}")]
    Missing(&'static str),

    #[error("clave privada no soportada")]
    UnsupportedKey,

    #[error("certificado inválido: {0}")]
    Certificate(String),

    #[error("estructura PDF no soportada: {0}")]
    Pdf(String),

    #[error("error criptográfico: {0}")]
    Crypto(String),

    #[error("la firma no cabe en el espacio reservado")]
    SignatureTooLarge,

    #[error("autoridad de sellado de tiempo: {0}")]
    Tsa(String),
}

pub type Result<T> = std::result::Result<T, Error>;
