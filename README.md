<!-- Idioma / Language: **Español** · [English](README.en.md) -->

# Rúbrica

> Cliente de firma electrónica nativo, moderno y libre, compatible con las sedes
> electrónicas españolas (plataforma @firma). Pensado para funcionar bien donde
> AutoFirma falla — empezando por Linux.

La *rúbrica* es el trazo final de una firma manuscrita. Este proyecto aspira a ser
ese trazo: firmar y validar documentos con certificado electrónico, sin Java, con
buena experiencia de uso y un empaquetado que simplemente funciona.

## Características

- Firma de documentos en los formatos estándar de la administración española:
  **PAdES** (PDF), **CAdES** (binario) y **XAdES** (XML).
- Perfiles ETSI: firma básica (**-B**) y con sello de tiempo (**-T**) mediante una
  autoridad TSA.
- Certificados en fichero **PKCS#12** (`.p12`/`.pfx`), como el de la FNMT, con
  cifrado moderno AES/PBES2 o heredado.
- Verificación de firmas con detección de documentos manipulados.
- Binario nativo de 64 bits, sin dependencias de Java.

## Componentes

| Componente | Descripción |
|------------|-------------|
| `rubrica-core` | Núcleo de firma y validación. Toda la lógica, sin interfaz. |
| `rubrica-cli` | Interfaz de línea de comandos sobre el núcleo. |
| `rubrica-gui` | Interfaz web local servida por el propio binario. |

## Instalación

Descarga el **AppImage**, dale permiso de ejecución y ábrelo:

```bash
chmod +x Rubrica-x86_64.AppImage
./Rubrica-x86_64.AppImage
```

No requiere instalar nada más en el sistema.

## Uso

```bash
rubrica sign --in documento.pdf --cert mi-certificado.p12 --out documento-firmado.pdf
rubrica sign --in documento.pdf --cert mi-certificado.p12 --out firmado.pdf --timestamp
rubrica sign --in documento.xml --cert mi-certificado.p12 --out firmado.xml --format xades
rubrica verify documento-firmado.pdf
```

## Documentación

- [Arquitectura](docs/es/arquitectura.md)
- [Guía de contribución](CONTRIBUTING.md)
- English documentation: [README.en.md](README.en.md) · [architecture](docs/en/architecture.md)

## Licencia

[GNU Affero General Public License v3.0 o posterior](LICENSE) (AGPL-3.0-or-later).

Copyright © 2026 Jorge Carrera.

Esto significa que cualquiera puede usar, estudiar y modificar Rúbrica, pero todo
trabajo derivado —incluido el ofrecido como servicio en red— debe distribuirse
también bajo AGPL-3.0 y mantener esta atribución. El código no puede cerrarse ni
comercializarse de forma privativa.
