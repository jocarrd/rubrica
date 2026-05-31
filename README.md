<!-- Idioma / Language: **Español** · [English](README.en.md) -->

# Rúbrica

> Cliente de firma electrónica nativo, moderno y libre, compatible con las sedes
> electrónicas españolas (plataforma @firma). Pensado para funcionar bien donde
> AutoFirma falla — empezando por Linux.

La *rúbrica* es el trazo final de una firma manuscrita. Este proyecto aspira a ser
ese trazo: firmar y validar documentos con DNIe, certificado FNMT y certificados
software, sin Java, con buena experiencia de uso y un empaquetado que simplemente
funciona.

## Por qué

AutoFirma es el cliente oficial de firma en España. Funciona, pero arrastra un
runtime de Java, instaladores frágiles y una integración con el navegador que se
rompe a menudo, especialmente en Linux. Rúbrica no reinventa la criptografía de
firma —que son estándares abiertos (ETSI XAdES, CAdES, PAdES, ASiC)— sino la
experiencia, el empaquetado y la integración.

## Estado

Sprint 0 — *de-risk*. Antes de construir el producto, validamos lo único que
puede tumbar el proyecto: **¿puede Rust generar firmas que los validadores
oficiales acepten?**

Ver [`spike/`](spike/): firmador PAdES-B de prueba y criterio de continuación
(go/no-go). Si el spike pasa [VALIDe](https://valide.redsara.es) o el
[demo DSS](https://ec.europa.eu/digital-building-blocks/DSS/webapp-demo/validation)
de la Comisión Europea, levantamos el workspace completo descrito en la
[arquitectura](docs/es/arquitectura.md).

## Hoja de ruta

| Fase | Objetivo |
|------|----------|
| 0 | **Spike**: firmar PAdES-B y validar contra herramientas oficiales |
| 1 | **MVP local**: firmar/validar PAdES + CAdES (-B/-T) con PKCS#12 y DNIe; AppImage Linux |
| 2 | XAdES y validación avanzada (-LT) |
| 3 | Puente del protocolo `afirma://` para integrarse con sedes reales |
| 4 | Windows/macOS, Flatpak, comunidad |

## Documentación

- [Arquitectura](docs/es/arquitectura.md)
- [Guía de contribución](CONTRIBUTING.md)
- English documentation: [README.en.md](README.en.md) · [architecture](docs/en/architecture.md)

## Licencia

Doble licencia a elección del usuario: [MIT](LICENSE-MIT) o
[Apache-2.0](LICENSE-APACHE). El motor criptográfico se construye con
dependencias de licencia permisiva o se integra como proceso separado; no deriva
de `clienteafirma` (GPL/EUPL).
