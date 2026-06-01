//! Proveedor de AutoFirma estándar (protocolo `afirma://`), usado por la mayoría
//! de las comunidades y por la Administración del Estado.
//!
//! En el protocolo de AutoFirma los datos a firmar llegan por el parámetro `dat`
//! (en base64) o se descargan de un servidor de almacenamiento indicado por
//! `stservlet`. Aquí se resuelve el caso directo (`dat`); el caso de servidor de
//! almacenamiento queda pendiente de verificar contra una sede real.

use super::{ProveedorSede, Solicitud};
use crate::protocol::Invocation;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub struct AfirmaProvider;

impl ProveedorSede for AfirmaProvider {
    fn nombre(&self) -> &'static str {
        "AutoFirma (@firma)"
    }

    fn preparar(&self, invocation: &Invocation) -> Solicitud {
        let documento = invocation
            .data
            .as_deref()
            .and_then(|d| STANDARD.decode(d).ok());

        Solicitud {
            servidor: invocation.id.clone().unwrap_or_default(),
            sesion: invocation.id.clone().unwrap_or_default(),
            documento,
        }
    }
}
