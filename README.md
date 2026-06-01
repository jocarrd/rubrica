<!-- Idioma / Language: **Español** · [English](README.en.md) -->

# Rúbrica

> Cliente de firma electrónica nativo, moderno y libre, compatible con las sedes
> electrónicas españolas (plataforma @firma). Pensado para funcionar bien donde
> AutoFirma falla — empezando por Linux.

La *rúbrica* es el trazo final de una firma manuscrita. Este proyecto aspira a ser
ese trazo: firmar y validar documentos con DNIe, certificado FNMT y certificados
software, sin Java, con buena experiencia de uso y un empaquetado que simplemente
funciona.

## Características

- Firma de documentos en los formatos estándar de la administración española:
  **PAdES** (PDF), **CAdES** (binario) y **XAdES** (XML).
- Perfiles ETSI desde firma básica (**-B**) hasta sello de tiempo (**-T**) y
  niveles de conservación a largo plazo.
- Acceso a claves mediante **DNIe** y tarjetas criptográficas (PKCS#11),
  certificados **PKCS#12** (`.p12`/`.pfx`) y el almacén **NSS** del sistema.
- Validación de firmas contra las herramientas oficiales.
- Binario nativo de 64 bits, sin dependencias de Java.

## Componentes

| Componente | Descripción |
|------------|-------------|
| `rubrica-core` | Núcleo de firma y validación. Toda la lógica, sin interfaz. |
| `rubrica-cli` | Interfaz de línea de comandos sobre el núcleo. |
| `rubrica` (app) | Aplicación de escritorio con interfaz gráfica nativa. |
| `afirma-bridge` | Puente del protocolo `afirma://` para integrarse con las sedes. |

## Instalación

> Distribución mediante **AppImage** y **Flatpak** en Linux. Windows y macOS
> llegarán más adelante.

Para firmar con DNIe o tarjeta criptográfica se necesitan los módulos PKCS#11 del
sistema:

```bash
sudo apt install opensc pcscd
```

## Uso

```bash
rubrica sign --in documento.pdf --cert mi-certificado.p12 --out documento-firmado.pdf
rubrica verify documento-firmado.pdf
```

## Documentación

- [Arquitectura](docs/es/arquitectura.md)
- [Guía de contribución](CONTRIBUTING.md)
- English documentation: [README.en.md](README.en.md) · [architecture](docs/en/architecture.md)

## Licencia

Doble licencia a elección del usuario: [MIT](LICENSE-MIT) o
[Apache-2.0](LICENSE-APACHE). El motor criptográfico se construye con
dependencias de licencia permisiva o se integra como proceso separado; no deriva
de `clienteafirma` (GPL/EUPL).
