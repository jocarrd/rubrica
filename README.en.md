<!-- Language / Idioma: **English** · [Español](README.md) -->

# Rúbrica

> A native, modern, free electronic-signature client compatible with Spanish
> public-administration portals (the @firma platform). Built to work well where
> AutoFirma struggles — starting with Linux.

A *rúbrica* is the final flourish of a handwritten signature. This project aims to
be that stroke: signing and validating documents with an electronic certificate —
no Java, good UX, and packaging that just works.

## Features

- Signs documents in the standard formats of the Spanish administration:
  **PAdES** (PDF), **CAdES** (binary) and **XAdES** (XML).
- ETSI profiles: baseline (**-B**) and timestamped (**-T**) via a TSA.
- File-based **PKCS#12** certificates (`.p12`/`.pfx`), such as the FNMT one, with
  modern AES/PBES2 or legacy encryption.
- Signature verification with tampering detection.
- Native 64-bit binary, no Java dependency.

## Portal compatibility

| Region | Status |
|--------|--------|
| **La Rioja** (carFirma) | Integration verified: the portal launches Rúbrica via `carfirma://`, which parses the invocation and communicates with the official signing server. |

The remaining autonomous communities use the same @firma platform and will be
verified one by one.

## Components

| Component | Description |
|-----------|-------------|
| `rubrica-core` | Signing and validation core. All logic, no UI. |
| `rubrica-cli` | Command-line interface over the core. |
| `rubrica-gui` | Local web interface served by the binary itself. |
| `afirma-bridge` | Portal integration: handler for the `carfirma://` and `afirma://` protocols and local server. |

## Portal integration

So that clicking "Sign" on a portal opens Rúbrica instead of AutoFirma or carFirma,
register it as the protocol handler:

```bash
crates/afirma-bridge/install-handler.sh
```

This associates the `carfirma://` and `afirma://` schemes with Rúbrica using the
same desktop mechanism as the official client.

## Installation

Download the **AppImage**, make it executable and run it:

```bash
chmod +x Rubrica-x86_64.AppImage
./Rubrica-x86_64.AppImage
```

No further system installation required.

## Usage

```bash
rubrica sign --in document.pdf --cert my-certificate.p12 --out document-signed.pdf
rubrica sign --in document.pdf --cert my-certificate.p12 --out signed.pdf --timestamp
rubrica sign --in document.xml --cert my-certificate.p12 --out signed.xml --format xades
rubrica verify document-signed.pdf
```

## Documentation

- [Architecture](docs/en/architecture.md)
- [Compatibility with the autonomous communities](docs/en/regional-compatibility.md)
- [Contributing](CONTRIBUTING.md)
- Documentación en español: [README.md](README.md) · [arquitectura](docs/es/arquitectura.md)

## License

[GNU Affero General Public License v3.0 or later](LICENSE) (AGPL-3.0-or-later).

Copyright © 2026 Jorge Carrera.

Anyone may use, study and modify Rúbrica, but every derivative work — including
one offered as a network service — must also be distributed under the AGPL-3.0
and keep this attribution. The code may not be closed or made proprietary.
