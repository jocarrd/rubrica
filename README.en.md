<!-- Language / Idioma: **English** · [Español](README.md) -->

# Rúbrica

> A native, modern, free electronic-signature client compatible with Spanish
> public-administration portals (the @firma platform). Built to work well where
> AutoFirma struggles — starting with Linux.

A *rúbrica* is the final flourish of a handwritten signature. This project aims to
be that stroke: signing and validating documents with the Spanish national eID
(DNIe), FNMT certificates and software certificates — no Java, good UX, and
packaging that just works.

## Features

- Signs documents in the standard formats of the Spanish administration:
  **PAdES** (PDF), **CAdES** (binary) and **XAdES** (XML).
- ETSI profiles from baseline (**-B**) to timestamped (**-T**) and long-term
  preservation levels.
- Key access via **DNIe** and smart cards (PKCS#11), **PKCS#12** certificates
  (`.p12`/`.pfx`) and the system **NSS** store.
- Signature validation against the official tools.
- Native 64-bit binary, no Java dependency.

## Components

| Component | Description |
|-----------|-------------|
| `rubrica-core` | Signing and validation core. All logic, no UI. |
| `rubrica-cli` | Command-line interface over the core. |
| `rubrica` (app) | Desktop application with a native GUI. |
| `afirma-bridge` | `afirma://` protocol bridge to integrate with the portals. |

## Installation

> Distributed as an **AppImage** and **Flatpak** on Linux. Windows and macOS will
> follow.

Signing with a DNIe or smart card requires the system PKCS#11 modules:

```bash
sudo apt install opensc pcscd
```

## Usage

```bash
rubrica sign --in document.pdf --cert my-certificate.p12 --out document-signed.pdf
rubrica sign --in document.pdf --cert my-certificate.p12 --out signed.pdf --timestamp
rubrica sign --in document.xml --cert my-certificate.p12 --out signed.xml --format xades
rubrica verify document-signed.pdf
```

## Documentation

- [Architecture](docs/en/architecture.md)
- [Contributing](CONTRIBUTING.md)
- Documentación en español: [README.md](README.md) · [arquitectura](docs/es/arquitectura.md)

## License

Dual-licensed at your option: [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
The cryptographic engine is built from permissively licensed dependencies or
integrated as a separate process; it does not derive from `clienteafirma`
(GPL/EUPL).
