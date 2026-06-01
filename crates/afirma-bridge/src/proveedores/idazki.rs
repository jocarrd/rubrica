//! Proveedor del País Vasco (Idazki Desktop, de Izenpe, protocolo `idazki://`).
//!
//! Idazki Desktop intercepta URIs `idazki://`, recibe el PDF a firmar en base64 y
//! firma localmente (XAdES/CAdES/PAdES). Sigue el mismo patrón que carFirma. El
//! formato exacto de la invocación debe confirmarse contra una sede real de
//! Euskadi; mientras tanto se resuelve el caso de datos embebidos en base64.

use super::{ProveedorSede, Solicitud};
use crate::protocol::Invocation;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub struct IdazkiProvider;

impl ProveedorSede for IdazkiProvider {
    fn nombre(&self) -> &'static str {
        "Idazki (País Vasco)"
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
