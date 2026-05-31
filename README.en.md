<!-- Language / Idioma: **English** · [Español](README.md) -->

# Rúbrica

> A native, modern, free electronic-signature client compatible with Spanish
> public-administration portals (the @firma platform). Built to work well where
> AutoFirma struggles — starting with Linux.

A *rúbrica* is the final flourish of a handwritten signature. This project aims to
be that stroke: signing and validating documents with the Spanish national eID
(DNIe), FNMT certificates and software certificates — no Java, good UX, and
packaging that just works.

## Why

AutoFirma is the official signing client in Spain. It works, but it ships a Java
runtime, brittle installers and browser integration that breaks often, especially
on Linux. Rúbrica does not reinvent signature cryptography — those are open
standards (ETSI XAdES, CAdES, PAdES, ASiC) — it reinvents the experience, the
packaging and the integration.

## Status

Sprint 0 — de-risk. Before building the product we validate the one thing that
can kill it: **can Rust produce signatures that official validators accept?**

See [`spike/`](spike/): a proof-of-concept PAdES-B signer and the go/no-go
criterion. If the spike passes [VALIDe](https://valide.redsara.es) or the European
Commission's
[DSS demo](https://ec.europa.eu/digital-building-blocks/DSS/webapp-demo/validation),
we stand up the full workspace described in the
[architecture](docs/en/architecture.md).

## Roadmap

| Phase | Goal |
|-------|------|
| 0 | **Spike**: sign PAdES-B and validate against official tools |
| 1 | **Local MVP**: sign/validate PAdES + CAdES (-B/-T) with PKCS#12 and DNIe; Linux AppImage |
| 2 | XAdES and advanced validation (-LT) |
| 3 | `afirma://` protocol bridge to integrate with real portals |
| 4 | Windows/macOS, Flatpak, community |

## Documentation

- [Architecture](docs/en/architecture.md)
- [Contributing](CONTRIBUTING.md)
- Documentación en español: [README.md](README.md) · [arquitectura](docs/es/arquitectura.md)

## License

Dual-licensed at your option: [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
The cryptographic engine is built from permissively licensed dependencies or
integrated as a separate process; it does not derive from `clienteafirma`
(GPL/EUPL).
