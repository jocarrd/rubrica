<!-- Idioma / Language: **Español** · [English](../en/architecture.md) -->

# Arquitectura

> Cliente moderno de firma electrónica compatible con las sedes electrónicas
> españolas (plataforma @firma), pensado para funcionar bien donde AutoFirma falla.
>
> - **Stack:** Rust nativo, sin runtime. GUI nativa (egui / slint / gtk-rs).
> - **Plataforma inicial:** Linux.
> - **Licencia:** MIT o Apache-2.0. Permisiva porque no derivamos de
>   `clienteafirma` (GPL/EUPL); el motor se construye con piezas permisivas o se
>   integra como proceso separado.

## Idea en una frase

No reinventamos la criptografía de firma (estándares ETSI abiertos); reinventamos
la experiencia, el empaquetado y la integración con el navegador, que es donde está
el dolor real.

## Motor de firma

Al elegir Rust nativo renunciamos a DSS (que es Java), así que el motor de firma es
la parte difícil, no la interfaz. Plan realista por formato:

| Formato | Uso en sedes | Dificultad en Rust | Plan |
|---------|--------------|--------------------|------|
| CAdES (CMS/PKCS#7) | Alto | Media | MVP. `cms` + `x509-cert` (RustCrypto) |
| PAdES (PDF) | Muy alto | Media-alta | MVP. CAdES detached embebido vía `lopdf` |
| XAdES (XML-DSig) | Muy alto | Alta (canonicalización) | Fase 2. Bindings a `xmlsec`/`libxml2` o sidecar |
| ASiC | Medio | Media | Fase 3 (contenedor ZIP sobre los anteriores) |

XAdES es el talón de Aquiles de Rust: la canonicalización XML (C14N) y XML-DSig no
tienen librería madura. Es la única parte donde quizá haga falta un sidecar o
bindings a C, pero no bloquea el MVP.

Niveles de firma (perfiles ETSI): el MVP cubre **-B** (básica) y **-T** (con sello
de tiempo RFC 3161). Los niveles -LT y -LTA (OCSP/CRL embebidos, archivado a largo
plazo) quedan para fases posteriores.

## Acceso a certificados y claves

Un trait común `SignerProvider` con tres implementaciones:

1. **PKCS#11** — DNIe y tarjetas criptográficas, vía el crate `cryptoki` apuntando
   a `opensc-pkcs11.so`. Requiere `opensc` y `pcscd`. Gestión de PIN.
2. **PKCS#12** — ficheros `.p12`/`.pfx`, como el certificado FNMT exportado.
3. **Almacén NSS del sistema** — `~/.pki/nssdb`, el que usan Firefox y Chrome en
   Linux. (Nativo de 64 bits, sin el problema de arquitectura que tenía carFirma.)

## Estructura del workspace

```
rubrica/
├─ spike/                 de-risk: firmar PAdES-B y validar (fase actual)
├─ crates/
│  ├─ rubrica-core/       lógica pura, testeable en aislado
│  │  ├─ keystore/        SignerProvider: pkcs11, pkcs12, nss
│  │  ├─ formats/         cades, pades, (xades en fase 2)
│  │  ├─ tsa/             sellado de tiempo RFC 3161
│  │  └─ validation/      verificación de firmas
│  ├─ rubrica-cli/        interfaz de línea de comandos sobre el core
│  └─ afirma-bridge/      fase 3: servidor local + handler afirma://
├─ app/                   GUI nativa
└─ packaging/             AppImage + Flatpak
```

Regla de oro: toda la lógica vive en `rubrica-core`, un crate independiente con
tests. La GUI y la CLI son capas finas por encima. Esto permite tener una CLI
desde el principio reutilizando el mismo núcleo.

## Integración con las sedes (fase 3)

Las sedes no llaman a la app directamente: su JavaScript (`autoscript.js`) invoca
el protocolo de AutoFirma, que tiene dos mecanismos:

- registra el esquema `afirma://` (en Linux mediante un `.desktop` con
  `MimeType=x-scheme-handler/afirma`, el mismo mecanismo que usaba carFirma con
  `x-scheme-handler/carfirma`), y
- levanta un servidor local en `127.0.0.1` (puertos del orden de 63117) que habla
  un protocolo JSON con comandos (`sign`, `cosign`, `batch`; parámetros en base64).

Para ser compatible con todas las comunidades hay que implementar ese protocolo con
exactitud. Se entiende a partir del protocolo público y de capturas de red. Se
documenta desde fuentes públicas y observación; no se porta código GPL.

## Empaquetado

- **AppImage** (un único fichero ejecutable) y **Flatpak** (Software de GNOME y
  derivados).
- Binario de 64 bits nativo, sin dependencias de 32 bits. Los módulos PKCS#11 del
  DNIe se toman del sistema (`opensc`), documentando la dependencia.
- Registro del handler `afirma://` durante la instalación.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|--------|------------|
| XAdES/C14N en Rust es difícil | Diferir a fase 2; sidecar o bindings a xmlsec |
| El protocolo de sede cambia | Aislarlo en `afirma-bridge`; tests de integración |
| Confianza (validez legal) | Validar contra herramientas oficiales y vectores conocidos |
| Abandono a mitad | Entregar algo útil en la fase 1; no abarcarlo todo de golpe |
| Línea legal del clean-room | Documentar desde fuentes públicas, no portar código GPL |
