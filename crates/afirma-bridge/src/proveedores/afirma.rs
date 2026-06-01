use super::{ProveedorSede, ResultadoFirma, Solicitud};
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

    fn firmar(&self, solicitud: &Solicitud, identity: &rubrica_core::Identity) -> ResultadoFirma {
        firmar_local(solicitud, identity)
    }
}

pub(super) fn firmar_local(
    solicitud: &Solicitud,
    identity: &rubrica_core::Identity,
) -> ResultadoFirma {
    let Some(datos) = &solicitud.documento else {
        return ResultadoFirma::error("no se recibió el documento a firmar");
    };
    let firmado = if datos.starts_with(b"%PDF-") {
        rubrica_core::formats::pades::sign(datos, identity)
    } else {
        rubrica_core::formats::cades::sign(datos, identity)
    };
    match firmado {
        Ok(bytes) => {
            let salida = format!(
                "{}/.rubrica-firmado",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())
            );
            let _ = std::fs::write(&salida, &bytes);
            ResultadoFirma::ok(format!("firmado ({} bytes)", bytes.len()))
        }
        Err(e) => ResultadoFirma::error(e.to_string()),
    }
}
