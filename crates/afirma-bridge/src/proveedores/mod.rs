//! Proveedores de sede: cada administración con protocolo propio implementa
//! `ProveedorSede`. El proveedor adecuado se elige según el esquema de la URL
//! de invocación (`carfirma://`, `idazki://`, `afirma://`).

mod afirma;
mod carfirma;
mod idazki;

pub use afirma::AfirmaProvider;
pub use carfirma::CarFirmaProvider;
pub use idazki::IdazkiProvider;

use crate::protocol::Invocation;

/// Resultado de preparar una operación de firma a partir de la invocación.
pub struct Solicitud {
    /// Servidor de la sede con el que se intercambian los datos.
    pub servidor: String,
    /// Identificador de sesión de la operación.
    pub sesion: String,
    /// Documento a firmar, descargado del servidor de la sede (si se logró).
    pub documento: Option<Vec<u8>>,
}

/// Comportamiento común a todas las sedes. Cada implementación sabe interpretar
/// su protocolo y hablar con el servidor de su administración.
pub trait ProveedorSede {
    /// Nombre legible del proveedor (para mostrar al usuario).
    fn nombre(&self) -> &'static str;

    /// Prepara la solicitud de firma a partir de la invocación recibida:
    /// extrae servidor y sesión y descarga el documento a firmar.
    fn preparar(&self, invocation: &Invocation) -> Solicitud;
}

/// Devuelve el proveedor adecuado para el esquema de la URL de invocación.
pub fn para_esquema(url: &str) -> Box<dyn ProveedorSede> {
    if url.starts_with("carfirma://") {
        Box::new(CarFirmaProvider)
    } else if url.starts_with("idazki://") {
        Box::new(IdazkiProvider)
    } else {
        Box::new(AfirmaProvider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elige_el_proveedor_segun_el_esquema() {
        assert_eq!(
            para_esquema("carfirma://firmar?id=x").nombre(),
            "carFirma (La Rioja)"
        );
        assert_eq!(
            para_esquema("idazki://sign?dat=x").nombre(),
            "Idazki (País Vasco)"
        );
        assert_eq!(
            para_esquema("afirma://sign?dat=x").nombre(),
            "AutoFirma (@firma)"
        );
    }

    #[test]
    fn afirma_descarga_datos_embebidos_en_base64() {
        let inv = crate::protocol::parse("afirma://sign?dat=SGVsbG8%3D");
        let sol = AfirmaProvider.preparar(&inv);
        assert_eq!(sol.documento.as_deref(), Some(b"Hello".as_slice()));
    }
}
