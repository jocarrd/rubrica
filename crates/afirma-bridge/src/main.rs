mod certificados;
mod larioja;
mod protocol;
mod proveedores;

use proveedores::ProveedorSede;
use std::net::TcpListener;
use tungstenite::accept;
use tungstenite::Message;

const ECHO_PREFIX: &str = "echo=";
const ECHO_OK: &str = "OK";
const PORTS: [u16; 5] = [63117, 63118, 63119, 63120, 63121];

fn main() {
    // Modo manejador de protocolo: el sistema nos invoca con la URL de la sede
    // (carfirma://, afirma:// o idazki://) al pulsar "Firmar".
    if let Some(url) = std::env::args().nth(1) {
        if es_invocacion(&url) {
            handle_url(&url);
            return;
        }
    }
    serve();
}

fn es_invocacion(url: &str) -> bool {
    url.starts_with("carfirma://") || url.starts_with("afirma://") || url.starts_with("idazki://")
}

fn handle_url(url: &str) {
    let invocation = protocol::parse(url);
    let proveedor = proveedores::para_esquema(url);
    let solicitud = proveedor.preparar(&invocation);

    let mut report = String::new();
    report.push_str("=== Invocación recibida desde la sede ===\n");
    report.push_str(&format!("URL completa:\n{url}\n\n"));
    report.push_str(&format!("proveedor: {}\n", proveedor.nombre()));
    report.push_str(&format!("operación: {}\n", invocation.operation));
    report.push_str(&format!("servidor: {}\n", solicitud.servidor));
    report.push_str(&format!("sesión: {}\n", solicitud.sesion));
    report.push_str(&format!(
        "documento: {}\n",
        match &solicitud.documento {
            Some(d) => format!("{} bytes descargados", d.len()),
            None => "no se pudo descargar (¿sesión caducada?)".to_string(),
        }
    ));

    print!("{report}");
    log_invocation(&report);
    mostrar_ventana(proveedor.as_ref(), &solicitud, &report);
}

fn mostrar_ventana(
    proveedor: &dyn ProveedorSede,
    solicitud: &proveedores::Solicitud,
    report: &str,
) {
    let page = ventana_html(
        proveedor.nombre(),
        &solicitud.servidor,
        &solicitud.sesion,
        report,
    );
    let addr = std::env::var("RUBRICA_VENTANA_ADDR").unwrap_or_else(|_| "127.0.0.1:0".into());
    let Ok(listener) = TcpListener::bind(addr) else {
        return;
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    abrir_navegador(&format!("http://127.0.0.1:{port}/"));

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let req = leer_peticion(&mut stream);
        if req.starts_with("POST /firmar") {
            let cuerpo = req.rsplit("\r\n\r\n").next().unwrap_or("");
            let resultado = firmar_solicitud(cuerpo, proveedor, solicitud);
            responder(&mut stream, "application/json", &resultado);
            break;
        } else {
            responder(&mut stream, "text/html; charset=utf-8", &page);
        }
    }
}

fn leer_peticion(stream: &mut std::net::TcpStream) -> String {
    use std::io::Read;
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

fn responder(stream: &mut std::net::TcpStream, tipo: &str, cuerpo: &str) {
    use std::io::Write;
    let _ = stream.write_all(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {tipo}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
            cuerpo.len()
        )
        .as_bytes(),
    );
}

fn firmar_solicitud(
    cuerpo_json: &str,
    proveedor: &dyn ProveedorSede,
    solicitud: &proveedores::Solicitud,
) -> String {
    let cert = protocol::json_campo(cuerpo_json, "certificado").unwrap_or_default();
    let pass = protocol::json_campo(cuerpo_json, "password").unwrap_or_default();

    let identity = match rubrica_core::load_pkcs12(std::path::Path::new(&cert), &pass) {
        Ok(id) => id,
        Err(e) => {
            return format!(
                "{{\"ok\":false,\"error\":\"{}\"}}",
                escape_json(&e.to_string())
            )
        }
    };

    let resultado = proveedor.firmar(solicitud, &identity);
    format!(
        "{{\"ok\":{},\"mensaje\":\"{}\"}}",
        resultado.ok,
        escape_json(&resultado.mensaje)
    )
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn abrir_navegador(url: &str) {
    if std::env::var("RUBRICA_NO_ABRIR").is_ok() {
        return;
    }
    // Abrimos una ventana de aplicación dedicada (sin pestañas ni barra de
    // direcciones), de modo que parezca una ventana nativa de Rúbrica.
    let navegadores = [
        "chromium",
        "google-chrome",
        "chromium-browser",
        "brave-browser",
    ];
    let geometria = "--app=";
    for nav in navegadores {
        let lanzado = std::process::Command::new(nav)
            .arg(format!("{geometria}{url}"))
            .arg("--window-size=560,640")
            .spawn()
            .is_ok();
        if lanzado {
            return;
        }
    }
    // Si no hay navegador con modo app, recurrimos al manejador del sistema.
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn ventana_html(operacion: &str, servidor: &str, sesion: &str, report: &str) -> String {
    let plantilla = include_str!("ventana.html");
    plantilla
        .replace("{{OPERACION}}", &escape(operacion))
        .replace("{{SERVIDOR}}", &escape(servidor))
        .replace("{{SESION}}", &escape(sesion))
        .replace("{{OPCIONES_CERT}}", &opciones_certificados())
        .replace("{{DETALLE}}", &escape(report))
}

fn opciones_certificados() -> String {
    let certs = certificados::disponibles();
    if certs.is_empty() {
        return "<div class=\"empty\">No se encontró ningún certificado (.p12/.pfx) en tu carpeta personal ni en Descargas.</div>".into();
    }
    certs
        .iter()
        .map(|c| {
            format!(
                "<div class=\"cert\" data-ruta=\"{}\">\
<span class=\"ico\">🔑</span>\
<span class=\"nm\">{}</span>\
<span class=\"ck\">✓</span>\
</div>",
                escape(&c.ruta.to_string_lossy()),
                escape(&nombre_legible(&c.nombre))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn nombre_legible(archivo: &str) -> String {
    archivo
        .trim_end_matches(".p12")
        .trim_end_matches(".pfx")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn log_invocation(report: &str) {
    let path = format!("{}/.rubrica-invocacion.log", env_home());
    let _ = std::fs::write(&path, report);
}

fn env_home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn serve() {
    let listener = bind().expect("no se pudo abrir ningún puerto del protocolo afirma");
    let port = listener.local_addr().unwrap().port();
    println!("Puente afirma:// escuchando en 127.0.0.1:{port}");

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        std::thread::spawn(move || {
            if let Ok(mut ws) = accept(stream) {
                while let Ok(msg) = ws.read() {
                    if let Message::Text(text) = msg {
                        let response = handle(&text);
                        let _ = ws.send(Message::Text(response));
                    }
                }
            }
        });
    }
}

fn bind() -> Option<TcpListener> {
    PORTS
        .iter()
        .find_map(|p| TcpListener::bind(("127.0.0.1", *p)).ok())
}

fn handle(message: &str) -> String {
    if let Some(rest) = message.strip_prefix(ECHO_PREFIX) {
        let _ = rest;
        return ECHO_OK.to_string();
    }
    process_operation(message)
}

fn process_operation(message: &str) -> String {
    let invocation = protocol::parse(message);
    format!("RUBRICA_RECEIVED:{}", invocation.operation)
}
