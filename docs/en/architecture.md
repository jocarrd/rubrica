<!-- Language / Idioma: **English** · [Español](../es/arquitectura.md) -->

# Architecture

> A modern electronic-signature client compatible with Spanish
> public-administration portals (the @firma platform), built to work well where
> AutoFirma struggles.
>
> - **Stack:** native Rust, no runtime. Local web interface served by the binary.
> - **Platform:** Linux, distributed as an AppImage.
> - **License:** AGPL-3.0-or-later.

## The idea in one sentence

Rúbrica does not reinvent signature cryptography — those are open ETSI standards —
it reinvents the experience, the packaging and the browser integration, which is
where the real pain is.

## Signing engine

The engine is written in pure Rust on top of RustCrypto, with no Java dependency.
Supported formats:

| Format | Portal usage | Description |
|--------|--------------|-------------|
| CAdES (CMS/PKCS#7) | High | Detached binary signature |
| PAdES (PDF) | Very high | Detached CAdES embedded in the PDF |
| XAdES (XML-DSig) | Very high | Enveloping XML signature with C14N canonicalization |

Signature levels (ETSI profiles): **-B** (baseline) and **-T** (with an RFC 3161
timestamp obtained from a TSA).

## Certificate and key access

Certificates are loaded from **PKCS#12** files (`.p12`/`.pfx`), such as an exported
FNMT certificate. Both modern PBES2/AES encryption (produced by OpenSSL 3 and the
FNMT) and legacy 3DES are supported.

## Workspace layout

```
rubrica/
├─ crates/
│  ├─ rubrica-core/       signing and validation core
│  │  ├─ keystore/        PKCS#12 certificate loading
│  │  ├─ formats/         cades, pades, xades, validation
│  │  └─ tsa/             RFC 3161 timestamping
│  ├─ rubrica-cli/        command-line interface
│  └─ rubrica-gui/        local web interface
└─ ...
```

All logic lives in `rubrica-core`, a standalone crate with tests. The CLI and the
web interface are thin layers reusing the same core.

## Verification

Each format is checked against the reference tools: PAdES signatures with `pdfsig`
(poppler), XAdES with `xmlsec1`, and timestamps with `openssl ts`. The built-in
verifier recomputes the hash of the signed content and checks the RSA signature
against the certificate's public key, detecting any tampering with the document.

## Portal integration

Portals do not call the application directly: their JavaScript (`autoscript.js`)
invokes the AutoFirma protocol, which has two mechanisms:

- it registers the `afirma://` scheme (on Linux through a `.desktop` file with
  `MimeType=x-scheme-handler/afirma`), and
- it starts a local server on `127.0.0.1` speaking a JSON protocol with commands
  (`sign`, `cosign`, `batch`; base64 parameters).

The `afirma-bridge` crate implements that local WebSocket server: it answers the
detection handshake (`echo=` → `OK`) that portals use to locate the signing client.
Compatibility with that protocol is built from its public specification; no code is
ported from `clienteafirma`.

## Packaging

Rúbrica is distributed as an **AppImage**: a single executable file bundling the
binary and everything it needs, with no installation or system dependencies. Native
64-bit binary.
