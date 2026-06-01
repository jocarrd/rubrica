<!-- Idioma / Language: **Español** · [English](../en/architecture.md) -->

# Arquitectura

> Cliente moderno de firma electrónica compatible con las sedes electrónicas
> españolas (plataforma @firma), pensado para funcionar bien donde AutoFirma falla.
>
> - **Stack:** Rust nativo, sin runtime. Interfaz web local servida por el binario.
> - **Plataforma:** Linux, distribuido como AppImage.
> - **Licencia:** AGPL-3.0-or-later.

## Idea en una frase

Rúbrica no reinventa la criptografía de firma —que son estándares ETSI abiertos—
sino la experiencia, el empaquetado y la integración con el navegador, que es donde
está el dolor real.

## Motor de firma

El motor está escrito en Rust puro sobre RustCrypto, sin dependencias de Java.
Formatos soportados:

| Formato | Uso en sedes | Descripción |
|---------|--------------|-------------|
| CAdES (CMS/PKCS#7) | Alto | Firma binaria detached |
| PAdES (PDF) | Muy alto | CAdES detached embebido en el PDF |
| XAdES (XML-DSig) | Muy alto | Firma XML enveloping con canonicalización C14N |

Niveles de firma (perfiles ETSI): **-B** (básica) y **-T** (con sello de tiempo
RFC 3161 obtenido de una autoridad TSA).

## Acceso a certificados y claves

Los certificados se cargan desde ficheros **PKCS#12** (`.p12`/`.pfx`), como el
certificado de la FNMT exportado. Se admiten tanto el cifrado moderno PBES2/AES
(el que generan OpenSSL 3 y la FNMT) como el cifrado 3DES heredado.

## Estructura del workspace

```
rubrica/
├─ crates/
│  ├─ rubrica-core/       núcleo de firma y validación
│  │  ├─ keystore/        carga de certificados PKCS#12
│  │  ├─ formats/         cades, pades, xades, validación
│  │  └─ tsa/             sellado de tiempo RFC 3161
│  ├─ rubrica-cli/        interfaz de línea de comandos
│  └─ rubrica-gui/        interfaz web local
└─ ...
```

Toda la lógica vive en `rubrica-core`, un crate independiente con tests. La CLI y
la interfaz web son capas finas que reutilizan el mismo núcleo.

## Verificación

Cada formato se contrasta con las herramientas de referencia: las firmas PAdES con
`pdfsig` (poppler), las XAdES con `xmlsec1`, y los sellos de tiempo con `openssl
ts`. La verificación propia recalcula el hash del contenido firmado y comprueba la
firma RSA contra la clave pública del certificado, detectando cualquier
manipulación del documento.

## Integración con las sedes

Las sedes no llaman a la aplicación directamente: su JavaScript (`autoscript.js`)
invoca el protocolo de AutoFirma, que tiene dos mecanismos:

- registra el esquema `afirma://` (en Linux mediante un `.desktop` con
  `MimeType=x-scheme-handler/afirma`), y
- levanta un servidor local en `127.0.0.1` que habla un protocolo JSON con comandos
  (`sign`, `cosign`, `batch`; parámetros en base64).

El crate `afirma-bridge` implementa ese servidor WebSocket local: responde al
handshake de detección (`echo=` → `OK`) que usan las sedes para localizar el
cliente de firma. La compatibilidad con ese protocolo se construye a partir de su
especificación pública; no se porta código de `clienteafirma`.

## Empaquetado

Rúbrica se distribuye como **AppImage**: un único fichero ejecutable que incluye el
binario y todo lo necesario, sin instalación ni dependencias del sistema. Binario
de 64 bits nativo.
