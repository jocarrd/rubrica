//! Localización de certificados del usuario para que pueda elegir con cuál firmar.

use std::path::PathBuf;

pub struct Certificado {
    pub ruta: PathBuf,
    pub nombre: String,
}

/// Busca certificados PKCS#12 (.p12/.pfx) en las ubicaciones habituales del usuario.
pub fn disponibles() -> Vec<Certificado> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".into());
    let dirs = [
        home.clone(),
        format!("{home}/Descargas"),
        format!("{home}/Documentos"),
        format!("{home}/.rubrica/certificados"),
    ];

    let mut encontrados = Vec::new();
    for dir in &dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let ruta = entry.path();
            let ext = ruta
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
            if matches!(ext.as_deref(), Some("p12") | Some("pfx")) {
                let nombre = ruta
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("certificado")
                    .to_string();
                encontrados.push(Certificado { ruta, nombre });
            }
        }
    }
    encontrados.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    encontrados
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_p12_en_el_home() {
        let home = std::env::temp_dir().join("rubrica-home-test");
        let _ = std::fs::create_dir_all(&home);
        let p12 = home.join("CERT_PRUEBA.p12");
        std::fs::write(&p12, b"dummy").unwrap();

        let previo = std::env::var("HOME").ok();
        std::env::set_var("HOME", &home);
        let encontrados = disponibles();
        if let Some(h) = previo {
            std::env::set_var("HOME", h);
        }

        assert!(encontrados.iter().any(|c| c.nombre == "CERT_PRUEBA.p12"));
        let _ = std::fs::remove_file(&p12);
    }
}
